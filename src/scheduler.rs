// Licensed under MPL-2.0 (Dual License)
//
// Tileline HPC / GMS Scheduler
// This file implements the GPU task scheduler.

use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::thread;
use wasmer::{imports, Instance, Module, Store, Value};
use crate::compiler::WasmGpuCompiler;

/// Task priorities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GmsTaskPriority {
    Low,
    Medium,
    High,
}

/// Task structure designed for GPU workloads.
pub type GmsTaskPayload = Box<dyn FnOnce() + Send + 'static>;

/// Task structure containing a Wasm module.
#[derive(Clone)]
pub struct WasmTask {
    pub module_bytes: Arc<[u8]>,
    pub entrypoint: String,
    pub args: Vec<i64>,
}

pub enum GmsTaskKind {
    Native(GmsTaskPayload),
    Wasm(WasmTask),
}

pub struct GmsTask {
    pub id: u64,
    pub priority: GmsTaskPriority,
    pub payload: GmsTaskKind,
    /// Indicates whether this task was stolen from another worker.
    pub is_stolen: bool,
}

/// Context for the GPU worker thread.
pub struct GmsWorkerContext {
    pub id: usize,
    /// The worker's own high-priority task queue.
    local_high_queue: Arc<Mutex<VecDeque<GmsTask>>>,
    /// References to other workers' high-priority queues (for stealing).
    other_high_queues: Vec<Arc<Mutex<VecDeque<GmsTask>>>>,
    /// Shared pool where stolen (or externally submitted) tasks are consumed cooperatively (sharing).
    shared_high_queue: Arc<Mutex<VecDeque<GmsTask>>>,
    /// Pool where low and medium priority tasks are shared using the task sharing method.
    shared_low_med_queue: Arc<Mutex<VecDeque<GmsTask>>>,
    /// Condition variable triggered upon task completion or arrival of a new Chunk.
    condvar: Arc<Condvar>,
    /// Dummy Mutex for the Condvar.
    cv_mutex: Arc<Mutex<bool>>,
}

impl GmsWorkerContext {
    /// Called when a new task is spawned by the worker itself or another source.
    pub fn spawn_high(&self, task: GmsTask) {
        if task.is_stolen {
            // Rule 2: Stolen tasks... will use task sharing among themselves.
            self.shared_high_queue.lock().unwrap().push_back(task);
        } else {
            // Push our normal task to our local queue.
            self.local_high_queue.lock().unwrap().push_back(task);
        }
        self.condvar.notify_one();
    }
    
    /// Fetches the next task.
    pub fn fetch_next(&self) -> Option<GmsTask> {
        // 1. First, check own local queue (High priority).
        if let Some(task) = self.local_high_queue.lock().unwrap().pop_front() {
            return Some(task);
        }
        
        // 2. Fetch shared high-priority tasks / stolen tasks (Task Sharing).
        if let Some(task) = self.shared_high_queue.lock().unwrap().pop_front() {
            return Some(task);
        }
        
        // 3. Task Stealing: Steal from other workers' High queues.
        for other_queue in &self.other_high_queues {
            if let Ok(mut guard) = other_queue.try_lock() {
                // Stealing from the back of the queue (pop_back) reduces conflicts with the original owner's pop_front.
                if let Some(mut task) = guard.pop_back() {
                    task.is_stolen = true;
                    return Some(task);
                }
            }
        }
        
        // 4. Fetch from the global Low/Medium Queue (Task Sharing).
        if let Some(task) = self.shared_low_med_queue.lock().unwrap().pop_front() {
            return Some(task);
        }
        
        None
    }
    
    /// Infinite listening loop running within the thread.
    pub fn run_loop(&self, completed_tasks: &std::sync::atomic::AtomicU64, active_time_ns: &std::sync::atomic::AtomicU64) {
        loop {
            if let Some(task) = self.fetch_next() {
                let start_time = std::time::Instant::now();
                // Execute the task.
                match task.payload {
                    GmsTaskKind::Native(payload) => payload(),
                    GmsTaskKind::Wasm(wasm_task) => {
                        let _ = Self::execute_wasm(wasm_task);
                    }
                }
                let elapsed_ns = start_time.elapsed().as_nanos() as u64;
                active_time_ns.fetch_add(elapsed_ns, std::sync::atomic::Ordering::Relaxed);
                
                completed_tasks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                // During idle periods, sleep via condvar to save CPU cycles.
                let mut guard = self.cv_mutex.lock().unwrap();
                if self.fetch_next().is_none() {
                    // We can check *guard to exit if a poison pill (shutdown) signal is received.
                    if *guard {
                        break;
                    }
                    guard = self.condvar.wait(guard).unwrap();
                    if *guard {
                        break;
                    }
                }
            }
        }
    }

    fn execute_wasm(wasm_task: WasmTask) -> Result<(), String> {
        let compiler = WasmGpuCompiler::new();
        let ptx = compiler.compile_to_ptx(wasm_task.module_bytes.as_ref())
            .map_err(|e| format!("AOT Compilation Failed: {}", e))?;
        
        // In the future, this PTX will be submitted to the GPU via multi_gpu_runtime.
        // For MVP, we simulate the GPU execution by running it on the CPU fallback (Wasmer)
        // so that the benchmark can measure the true FLOP throughput mathematically.

        let mut store = Store::default();
        let module = Module::new(&store, wasm_task.module_bytes.as_ref())
            .map_err(|err| err.to_string())?;

        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|err| err.to_string())?;

        let function = instance
            .exports
            .get_function(&wasm_task.entrypoint)
            .map_err(|_| format!("Missing export {}", wasm_task.entrypoint))?;

        let params: Vec<Value> = wasm_task.args.into_iter().map(Value::I64).collect();
        function
            .call(&mut store, &params)
            .map_err(|err| err.to_string())?;

        Ok(())
    }
}

/// GMS GPU Task Scheduler
pub struct GmsScheduler {
    pub worker_count: usize,
    shared_low_med_queue: Arc<Mutex<VecDeque<GmsTask>>>,
    shared_high_queue: Arc<Mutex<VecDeque<GmsTask>>>,
    next_task_id: std::sync::atomic::AtomicU64,
    pub completed_tasks: Arc<std::sync::atomic::AtomicU64>,
    pub active_time_ns: Arc<std::sync::atomic::AtomicU64>,
    condvar: Arc<Condvar>,
    cv_mutex: Arc<Mutex<bool>>, // Shutdown flag
    workers: Vec<thread::JoinHandle<()>>,
}

impl Default for GmsScheduler {
    fn default() -> Self {
        Self::new(8) // Default: 8 hardware threads
    }
}

impl GmsScheduler {
    /// Creates a context for the specified number of workers and returns the scheduler.
    pub fn new(worker_count: usize) -> Self {
        let shared_low_med_queue = Arc::new(Mutex::new(VecDeque::new()));
        let shared_high_queue = Arc::new(Mutex::new(VecDeque::new()));
        let condvar = Arc::new(Condvar::new());
        let cv_mutex = Arc::new(Mutex::new(false)); // false = running, true = shutdown
        let completed_tasks = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let active_time_ns = Arc::new(std::sync::atomic::AtomicU64::new(0));
        
        let mut local_queues = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            local_queues.push(Arc::new(Mutex::new(VecDeque::new())));
        }
        
        let mut worker_threads = Vec::with_capacity(worker_count);
        for (i, local_queue) in local_queues.iter().enumerate() {
            let mut other_queues = local_queues.clone();
            other_queues.remove(i);
            
            let context = GmsWorkerContext {
                id: i,
                local_high_queue: local_queue.clone(),
                other_high_queues: other_queues,
                shared_high_queue: shared_high_queue.clone(),
                shared_low_med_queue: shared_low_med_queue.clone(),
                condvar: condvar.clone(),
                cv_mutex: cv_mutex.clone(),
            };
            
            let completed_tasks_clone = completed_tasks.clone();
            let active_time_clone = active_time_ns.clone();
            
            // Spawn worker threads.
            worker_threads.push(thread::spawn(move || {
                context.run_loop(&completed_tasks_clone, &active_time_clone);
            }));
        }
        
        Self {
            worker_count,
            shared_low_med_queue,
            shared_high_queue,
            next_task_id: std::sync::atomic::AtomicU64::new(1),
            completed_tasks,
            active_time_ns,
            condvar,
            cv_mutex,
            workers: worker_threads,
        }
    }
    
    /// Submits a new task.
    pub fn submit_task<F>(&self, priority: GmsTaskPriority, f: F) 
    where 
        F: FnOnce() + Send + 'static 
    {
        self.submit_internal(priority, GmsTaskKind::Native(Box::new(f)));
    }
    
    pub fn submit_wasm(&self, priority: GmsTaskPriority, wasm_task: WasmTask) {
        self.submit_internal(priority, GmsTaskKind::Wasm(wasm_task));
    }
    
    fn submit_internal(&self, priority: GmsTaskPriority, payload: GmsTaskKind) {
        let id = self.next_task_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let task = GmsTask {
            id,
            priority,
            payload,
            is_stolen: false,
        };
        
        if task.priority == GmsTaskPriority::High {
            self.shared_high_queue.lock().unwrap().push_back(task);
        } else {
            self.shared_low_med_queue.lock().unwrap().push_back(task);
        }
        // Wake up idle workers.
        self.condvar.notify_one();
    }
    
    /// Submits the given function f with High priority.
    pub fn submit_native<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit_task(GmsTaskPriority::High, f);
    }
    
    /// Waits for pending tasks to finish.
    pub fn wait_for_idle(&self) {
        loop {
            let low_med_empty = self.shared_low_med_queue.lock().unwrap().is_empty();
            let high_empty = self.shared_high_queue.lock().unwrap().is_empty();
            if low_med_empty && high_empty {
                break;
            }
            thread::sleep(std::time::Duration::from_micros(100));
        }
    }
}

impl Drop for GmsScheduler {
    fn drop(&mut self) {
        // Trigger the shutdown flag
        {
            let mut guard = self.cv_mutex.lock().unwrap();
            *guard = true;
        }
        self.condvar.notify_all();
        
        // Wait for threads to finish
        while let Some(worker) = self.workers.pop() {
            let _ = worker.join();
        }
    }
}
