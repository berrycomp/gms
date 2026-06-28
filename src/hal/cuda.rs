use super::{Adapter, AdapterInfo, Backend, DeviceType, AdapterInternal};
use std::sync::Arc;
use std::ffi::CStr;
use cudarc::driver::{CudaDevice as CudarcDevice, CudaStream, CudaSlice, CudaFunction, LaunchAsync, LaunchConfig, sys};
use cudarc::nvrtc::Ptx;

pub struct CudaAdapter {
    pub ordinal: usize,
    pub name: String,
}

impl CudaAdapter {
    pub fn request_device(&self) -> CudaDevice {
        let device = CudarcDevice::new(self.ordinal).expect("Failed to create CUDA device context");
        let stream = device.fork_default_stream().expect("Failed to create stream");
        
        CudaDevice {
            device: device.clone(),
            queue: CudaQueue {
                device: device.clone(),
                stream: Arc::new(stream),
            },
        }
    }
}

#[derive(Clone)]
pub struct CudaDevice {
    pub device: Arc<CudarcDevice>,
    pub queue: CudaQueue,
}

impl CudaDevice {
    pub fn create_buffer(&self, size: u64) -> CudaBuffer {
        let buffer = self.device.alloc_zeros::<u8>(size as usize).expect("Failed to allocate CUDA buffer");
        CudaBuffer { buffer, size }
    }

    pub fn create_compute_pipeline(&self, desc: &crate::hal::ComputePipelineDescriptor) -> CudaComputePipeline {
        let ptx_payload = desc.ptx_payload.expect("CUDA backend requires PTX payload");
        let module_name = Box::leak(desc.label.unwrap_or("gms_cuda_module").to_string().into_boxed_str());
        let entry_point = Box::leak(desc.entry_point.unwrap_or("gms_main").to_string().into_boxed_str());

        let ptx = Ptx::from_src(ptx_payload);
        
        // Load the PTX module.
        self.device.load_ptx(ptx, module_name, &[entry_point]).expect("Failed to load PTX module into CUDA device");
        
        let func = self.device.get_func(module_name, entry_point).expect("Failed to get CUDA function from module");
        
        CudaComputePipeline {
            function: func,
            device: self.device.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CudaQueue {
    pub device: Arc<CudarcDevice>,
    pub stream: Arc<CudaStream>,
}

unsafe impl Send for CudaQueue {}
unsafe impl Sync for CudaQueue {}
unsafe impl Send for CudaDevice {}
unsafe impl Sync for CudaDevice {}

pub struct CudaBuffer {
    pub buffer: CudaSlice<u8>,
    pub size: u64,
}

pub struct CudaTexture {}

pub struct CudaCommandEncoder {
    pub device: Arc<CudarcDevice>,
}

impl CudaCommandEncoder {
    pub fn new(device: Arc<CudarcDevice>) -> Self {
        Self { device }
    }

    pub fn begin_compute_pass(&mut self, _desc: &crate::hal::ComputePassDescriptor) -> CudaComputePass {
        let stream = self.device.fork_default_stream().expect("Failed to create compute stream");
        CudaComputePass {
            stream: Arc::new(stream),
            pipeline: None,
        }
    }
}

#[derive(Clone)]
pub struct CudaComputePipeline {
    pub function: CudaFunction,
    pub device: Arc<CudarcDevice>,
}

pub struct CudaComputePass {
    pub stream: Arc<CudaStream>,
    pub pipeline: Option<CudaFunction>,
}

impl CudaComputePass {
    pub fn set_pipeline(&mut self, pipeline: &CudaComputePipeline) {
        self.pipeline = Some(pipeline.function.clone());
    }

    pub fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        if let Some(func) = &self.pipeline {
            let config = LaunchConfig {
                grid_dim: (x, y, z),
                block_dim: (64, 1, 1),
                shared_mem_bytes: 0,
            };
            // Note: Since this MVP does not yet bind actual buffers, we pass empty params
            unsafe {
                func.clone().launch_on_stream(&self.stream, config, (0i32,)).expect("CUDA dispatch failed");
            }
        }
    }
}

pub fn enumerate_adapters() -> Vec<Adapter> {
    let mut adapters = Vec::new();
    let count = CudarcDevice::count().unwrap_or(0) as usize;

    for ordinal in 0..count {
        let mut name_str = format!("CUDA Device {}", ordinal);
        let mut device_handle = 0;
        
        unsafe {
            if sys::cuDeviceGet(&mut device_handle, ordinal as i32) == sys::cudaError_enum::CUDA_SUCCESS {
                let mut name_buf = [0i8; 256];
                if sys::cuDeviceGetName(name_buf.as_mut_ptr(), name_buf.len() as i32, device_handle) == sys::cudaError_enum::CUDA_SUCCESS {
                    if let Ok(cstr) = CStr::from_ptr(name_buf.as_ptr()).to_str() {
                        name_str = cstr.to_string();
                    }
                }
            }
        }

        let info = AdapterInfo {
            name: name_str.clone(),
            vendor: 0x10DE, // NVIDIA vendor ID
            device: ordinal as u32,
            device_type: DeviceType::DiscreteGpu,
            backend: Backend::Cuda,
driver: String::new(),
driver_info: String::new(),
        };

        let internal = AdapterInternal::Cuda(CudaAdapter {
            ordinal,
            name: name_str,
        });

        adapters.push(Adapter { info, internal });
    }

    adapters
}
