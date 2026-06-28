pub mod cuda;
pub mod metal;
pub mod vulkan;

use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum Backend {
    #[default] Vulkan,
    Metal,
    Cuda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
    Other,
}

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub device_type: DeviceType,
    pub driver: String,
    pub driver_info: String,
    pub backend: Backend,
}

#[derive(Debug, Clone, Default)]
pub struct Limits {
    pub max_buffer_size: u64,
    pub max_storage_buffer_binding_size: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub max_compute_workgroup_storage_size: u32,
    pub max_compute_workgroups_per_dimension: u32,
    pub max_texture_dimension_1d: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_texture_array_layers: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub struct Features {
    bits: u64,
}

impl Features {
    pub const empty: Self = Self { bits: 0 };
    pub const MAPPABLE_PRIMARY_BUFFERS: Self = Self { bits: 1 };
    pub fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }
}

pub struct Adapter {
    pub info: AdapterInfo,
    pub internal: AdapterInternal,
}

pub enum AdapterInternal {
    Vulkan(vulkan::VulkanAdapter),
    Metal(metal::MetalAdapter),
    Cuda(cuda::CudaAdapter),
}

impl Adapter {
    pub fn get_info(&self) -> AdapterInfo {
        self.info.clone()
    }
    pub fn limits(&self) -> Limits {
        Limits::default() // TODO: route to internal
    }
    pub fn features(&self) -> Features {
        Features::empty // TODO: route to internal
    }
    pub fn request_device(&self) -> (Device, Queue) {
        (Device { internal: DeviceInternal::Vulkan(unsafe { std::mem::zeroed() }) }, Queue { internal: QueueInternal::Vulkan(unsafe { std::mem::zeroed() }) })
    }
}

pub mod dummies;
pub use dummies::*;

#[derive(Clone)]
pub struct Device {
    pub internal: DeviceInternal,
}
impl Device {

    pub fn create_command_encoder(&self, _desc: &CommandEncoderDescriptor) -> CommandEncoder { CommandEncoder { internal: CommandEncoderInternal::Vulkan(unsafe { std::mem::zeroed() }) } }
    pub fn create_texture(&self, _desc: &TextureDescriptor) -> Texture { Texture { internal: TextureInternal::Vulkan(unsafe { std::mem::zeroed() }) } }
    pub fn create_buffer(&self, _desc: &BufferDescriptor) -> Buffer { Buffer { internal: BufferInternal::Vulkan(unsafe { std::mem::zeroed() }) } }
    pub fn create_bind_group_layout(&self, _desc: &BindGroupLayoutDescriptor) -> BindGroupLayout { BindGroupLayout {} }
    pub fn create_pipeline_layout(&self, _desc: &PipelineLayoutDescriptor) -> PipelineLayout { PipelineLayout {} }
    pub fn create_shader_module(&self, _desc: ShaderModuleDescriptor) -> ShaderModule { ShaderModule {} }
    pub fn create_compute_pipeline(&self, desc: &ComputePipelineDescriptor) -> ComputePipeline {
        match &self.internal {
            DeviceInternal::Cuda(cuda_device) => ComputePipeline { internal: ComputePipelineInternal::Cuda(cuda_device.create_compute_pipeline(desc)) },
            _ => ComputePipeline { internal: ComputePipelineInternal::Dummy },
        }
    }
    pub fn create_bind_group(&self, _desc: &BindGroupDescriptor) -> BindGroup { BindGroup {} }
    

}

#[derive(Clone)]
pub enum DeviceInternal {
    Vulkan(vulkan::VulkanDevice),
    Metal(metal::MetalDevice),
    Cuda(cuda::CudaDevice),
}

impl Device {
    pub fn poll_internal(&self) {}
    pub fn poll(&self, _poll_type: PollType) -> Result<PollStatus, PollError> { Ok(PollStatus {}) }
}

pub struct Queue {
    pub internal: QueueInternal,
}
impl Queue {

    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(&self, _command_buffers: I) -> SubmissionIndex { 0 }
    pub fn write_buffer(&self, _buffer: &Buffer, _offset: u64, _data: &[u8]) {}

}

pub enum QueueInternal {
    Vulkan(vulkan::VulkanQueue),
    Metal(metal::MetalQueue),
    Cuda(cuda::CudaQueue),
}

pub struct Buffer {
    pub internal: BufferInternal,
}
impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer").finish()
    }
}
impl Buffer {
    pub fn as_entire_binding(&self) -> BindingResource<'static> { BindingResource::Buffer(BufferBinding { buffer: unsafe { std::mem::transmute(self) }, offset: 0, size: None }) }
}

pub enum BufferInternal {
    Vulkan(vulkan::VulkanBuffer),
    Metal(metal::MetalBuffer),
    Cuda(cuda::CudaBuffer),
}

pub struct Texture {
    pub internal: TextureInternal,
}
impl Texture {

    pub fn create_view(&self, _desc: &TextureViewDescriptor) -> TextureView { TextureView {} }
    pub fn size(&self) -> Extent3d { Extent3d::default() }
    pub fn format(&self) -> TextureFormat { TextureFormat::default() }

}

pub enum TextureInternal {
    Vulkan(vulkan::VulkanTexture),
    Metal(metal::MetalTexture),
    Cuda(cuda::CudaTexture),
}

pub struct CommandEncoder {
    pub internal: CommandEncoderInternal,
}
impl CommandEncoder {

    pub fn begin_render_pass(&mut self, _desc: &RenderPassDescriptor) -> RenderPass { RenderPass {} }
    pub fn begin_compute_pass(&mut self, desc: &ComputePassDescriptor) -> ComputePass {
        match &mut self.internal {
            CommandEncoderInternal::Cuda(cuda_enc) => ComputePass { internal: ComputePassInternal::Cuda(cuda_enc.begin_compute_pass(desc)) },
            _ => ComputePass { internal: ComputePassInternal::Dummy },
        }
    }
    pub fn finish(self) -> CommandBuffer { CommandBuffer {} }
    pub fn copy_texture_to_buffer(&mut self, _source: ImageCopyTexture, _destination: ImageCopyBuffer, _copy_size: Extent3d) {}
    pub fn copy_buffer_to_buffer(&mut self, _source: &Buffer, _source_offset: u64, _destination: &Buffer, _destination_offset: u64, _copy_size: u64) {}
    pub fn clear_buffer(&mut self, _buffer: &Buffer, _offset: u64, _size: Option<std::num::NonZeroU64>) {}

}

pub enum CommandEncoderInternal {
    Vulkan(vulkan::VulkanCommandEncoder),
    Metal(metal::MetalCommandEncoder),
    Cuda(cuda::CudaCommandEncoder),
}

pub fn enumerate_adapters() -> Vec<Adapter> {
    let mut adapters = Vec::new();
    
    // #[cfg(all(target_os = "macos", feature = "metal"))] // usually metal is just imported
    adapters.extend(metal::enumerate_adapters());
    
    adapters.extend(vulkan::enumerate_adapters());
    adapters.extend(cuda::enumerate_adapters());
    
    adapters
}


pub struct RenderPass {}

#[derive(Clone)]
pub struct ComputePipeline {
    pub internal: ComputePipelineInternal,
}

#[derive(Clone)]
pub enum ComputePipelineInternal {
    Cuda(cuda::CudaComputePipeline),
    Dummy,
}

pub struct ComputePass {
    pub internal: ComputePassInternal,
}

pub enum ComputePassInternal {
    Cuda(cuda::CudaComputePass),
    Dummy,
}

impl ComputePass {
    pub fn set_pipeline(&mut self, pipeline: &ComputePipeline) {
        if let (ComputePassInternal::Cuda(cuda_pass), ComputePipelineInternal::Cuda(cuda_pipe)) = (&mut self.internal, &pipeline.internal) {
            cuda_pass.set_pipeline(cuda_pipe);
        }
    }
    pub fn set_bind_group(&mut self, _index: u32, _bind_group: &BindGroup, _offsets: &[u32]) {}
    pub fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        if let ComputePassInternal::Cuda(cuda_pass) = &mut self.internal {
            cuda_pass.dispatch_workgroups(x, y, z);
        }
    }
}
pub struct CommandBuffer {}
pub struct ImageCopyTexture<'a> { pub texture: &'a Texture, pub mip_level: u32, pub origin: Origin3d, pub aspect: TextureAspect }
pub struct ImageCopyBuffer<'a> { pub buffer: &'a Buffer, pub layout: ImageDataLayout }
pub struct Origin3d { pub x: u32, pub y: u32, pub z: u32 }
pub struct ImageDataLayout { pub offset: u64, pub bytes_per_row: Option<u32>, pub rows_per_image: Option<u32> }
