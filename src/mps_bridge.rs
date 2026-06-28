use mps::{MpsScheduler, TaskPriority, CorePreference};
use std::time::Duration;

/// Bridge to connect GMS workloads to the MPS CPU scheduler.
/// This allows GPU command encoding to be distributed across CPU cores,
/// mapped roughly to the concept of Compute Units (SMs).
pub struct MpsBridge {
    scheduler: MpsScheduler,
}

impl Default for MpsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl MpsBridge {
    pub fn new() -> Self {
        Self {
            scheduler: MpsScheduler::new(),
        }
    }

    /// Submits a batch of GPU encoding tasks across MPS workers.
    /// 
    /// `total_workgroups` represents the total amount of GPU work for the current pass.
    /// `chunk_size` is the number of workgroups assigned to each CPU worker (e.g. 64).
    /// `encode_fn` is called with `(start_index, count)` to build a command buffer chunk.
    pub fn encode_command_buffers<F>(&self, total_workgroups: u32, chunk_size: u32, encode_fn: F)
    where
        F: Fn(u32, u32) + Send + Sync + 'static,
    {
        let chunks = (total_workgroups + chunk_size.max(1) - 1) / chunk_size.max(1);
        let encode_fn = std::sync::Arc::new(encode_fn);

        for i in 0..chunks {
            let start = i * chunk_size;
            let count = if start + chunk_size > total_workgroups {
                total_workgroups - start
            } else {
                chunk_size
            };
            
            let f = std::sync::Arc::clone(&encode_fn);
            self.scheduler.submit_native(
                TaskPriority::High,
                CorePreference::Performance,
                move || {
                    f(start, count);
                }
            );
        }

        // Wait for all encoding tasks to finish before returning
        self.scheduler.wait_for_idle(Duration::from_millis(1000));
    }

    pub fn metrics(&self) -> mps::SchedulerMetrics {
        self.scheduler.metrics()
    }
}
