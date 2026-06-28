#[cfg(target_os = "macos")]
mod imp {
    use super::{Adapter, AdapterInfo, Backend, DeviceType, AdapterInternal};
    use metal::{
        Buffer, CommandBuffer, CommandQueue, ComputeCommandEncoder, Device,
        FunctionRef, MTLResourceOptions, Texture,
    };
    
    pub struct MetalAdapter {
        pub device: Device,
    }
    
    impl MetalAdapter {
        pub fn request_device(&self) -> MetalDevice {
            let queue = self.device.new_command_queue();
            MetalDevice {
                device: self.device.clone(),
                queue: MetalQueue { queue },
            }
        }
    }
    
    #[derive(Clone)]
pub struct MetalDevice {
        pub device: Device,
        pub queue: MetalQueue,
    }
    
    impl MetalDevice {
        pub fn create_buffer(&self, size: u64, options: MTLResourceOptions) -> MetalBuffer {
            let buffer = self.device.new_buffer(size, options);
            MetalBuffer { buffer }
        }
    
        pub fn create_buffer_with_data(&self, data: &[u8], options: MTLResourceOptions) -> MetalBuffer {
            let buffer = self.device.new_buffer_with_data(
                data.as_ptr() as *const _,
                data.len() as u64,
                options,
            );
            MetalBuffer { buffer }
        }
    
        pub fn create_compute_pipeline(&self, function: &FunctionRef) -> Result<metal::ComputePipelineState, String> {
            self.device.new_compute_pipeline_state_with_function(function)
                .map_err(|e| e.to_string())
        }
    }
    
    #[derive(Clone)]
    #[derive(Clone)]
pub struct MetalQueue {
        pub queue: CommandQueue,
    }
    
    #[derive(Debug)]
pub struct MetalBuffer {
        pub buffer: Buffer,
    }
    
    pub struct MetalTexture {
        pub texture: Texture,
    }
    
    pub struct MetalCommandEncoder {
        pub command_buffer: CommandBuffer,
    }
    
    impl MetalCommandEncoder {
        pub fn new(queue: &MetalQueue) -> Self {
            let command_buffer = queue.queue.new_command_buffer().to_owned();
            Self { command_buffer }
        }
    
        pub fn begin_compute_pass(&self) -> ComputeCommandEncoder {
            self.command_buffer.new_compute_command_encoder().to_owned()
        }
    
        pub fn commit(&self) {
            self.command_buffer.commit();
        }
    }
    
    pub fn enumerate_adapters() -> Vec<Adapter> {
        let mut adapters = Vec::new();
        
        // MTLCopyAllDevices()
        for device in Device::all() {
            let name = device.name().into();
            let device_type = if device.is_low_power() {
                DeviceType::IntegratedGpu
            } else {
                DeviceType::DiscreteGpu
            };
    
            let info = AdapterInfo {
                name,
                vendor: 0x106B, // Apple
                device: 0,
                device_type,
                backend: Backend::Metal,
            };
    
            let internal = AdapterInternal::Metal(MetalAdapter {
                device: device.clone(),
            });
            
            adapters.push(Adapter { info, internal });
        }
        
        adapters
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::super::{Adapter, AdapterInfo, Backend, DeviceType, AdapterInternal};
    
    pub struct MetalAdapter {}
    impl MetalAdapter {
        pub fn request_device(&self) -> MetalDevice { MetalDevice {} }
    }
    #[derive(Clone)]
pub struct MetalDevice {}
    #[derive(Clone)]
pub struct MetalQueue {}
    #[derive(Debug)]
pub struct MetalBuffer {}
    pub struct MetalTexture {}
    pub struct MetalCommandEncoder {}
    
    pub fn enumerate_adapters() -> Vec<Adapter> {
        vec![]
    }
}

pub use imp::*;
