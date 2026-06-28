// Dummy structs to replace wgpu descriptors and types
use super::Backend;

#[derive(Debug, Clone, Default)]
pub struct Instance {}
impl Instance {
    pub fn new(_: InstanceDescriptor) -> Self { Self {} }
    pub fn request_adapter(&self, _desc: &RequestAdapterOptions) -> std::future::Ready<Option<super::Adapter>> { std::future::ready(None) }
    pub fn enumerate_adapters(&self, _backends: super::Backend) -> Vec<super::Adapter> { vec![] }
    pub fn create_surface<W>(&self, _window: W) -> Result<Surface<'static>, SurfaceError> { Ok(Surface { _marker: std::marker::PhantomData }) }
}
#[derive(Debug, Clone, Default)]
pub struct InstanceDescriptor { pub backends: Backend }

#[derive(Debug, Clone, Default)]
pub struct RequestAdapterOptions<'a> {
    pub power_preference: PowerPreference,
    pub compatible_surface: Option<&'a Surface<'a>>,
    pub force_fallback_adapter: bool,
}

#[derive(Debug, Clone, Default)]
pub enum PowerPreference { #[default] LowPower, HighPerformance }

#[derive(Debug)]
pub struct Surface<'a> { pub _marker: std::marker::PhantomData<&'a ()> }
impl<'a> Surface<'a> {
    pub fn configure(&self, _device: &super::Device, _config: &SurfaceConfiguration) {}
    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> { Err(SurfaceError::Timeout) }
    pub fn get_capabilities(&self, _adapter: &super::Adapter) -> SurfaceCapabilities { SurfaceCapabilities { formats: vec![TextureFormat::Rgba8Unorm], present_modes: vec![PresentMode::AutoVsync], alpha_modes: vec![CompositeAlphaMode::Auto] } }
    pub fn get_default_config(&self, _adapter: &super::Adapter, w: u32, h: u32) -> Option<SurfaceConfiguration> { Some(SurfaceConfiguration { width: w, height: h, ..Default::default() }) }
}
#[derive(Debug, Clone, Default)]
pub struct SurfaceCapabilities { pub formats: Vec<TextureFormat>,
    pub present_modes: Vec<PresentMode>,
    pub alpha_modes: Vec<CompositeAlphaMode>, }
pub struct SurfaceTexture { pub texture: super::Texture, pub suboptimal: bool }
impl SurfaceTexture {
    pub fn present(self) {}
}

#[derive(Debug, Clone, Default)]
pub struct DeviceDescriptor<'a> {
    pub label: Option<&'a str>,
    pub required_features: super::Features,
    pub required_limits: super::Limits,
}

#[derive(Debug, Clone, Default)]
pub struct CommandEncoderDescriptor<'a> { pub label: Option<&'a str> }

#[derive(Debug, Clone, Default)]
pub struct BufferDescriptor<'a> {
    pub label: Option<&'a str>,
    pub size: u64,
    pub usage: BufferUsages,
    pub mapped_at_creation: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BufferUsages { pub bits: u32 }
impl BufferUsages {
    pub fn empty() -> Self { Self { bits: 0 } }
}
impl BufferUsages {
    pub const MAP_READ: Self = Self { bits: 1 };
    pub const MAP_WRITE: Self = Self { bits: 2 };
    pub const COPY_SRC: Self = Self { bits: 4 };
    pub const COPY_DST: Self = Self { bits: 8 };
    pub const INDEX: Self = Self { bits: 16 };
    pub const VERTEX: Self = Self { bits: 32 };
    pub const UNIFORM: Self = Self { bits: 64 };
    pub const STORAGE: Self = Self { bits: 128 };
    pub const INDIRECT: Self = Self { bits: 256 };
    pub const QUERY_RESOLVE: Self = Self { bits: 512 };
}
impl std::ops::BitOr for BufferUsages {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { Self { bits: self.bits | rhs.bits } }
}
impl std::ops::BitOrAssign for BufferUsages {
    fn bitor_assign(&mut self, rhs: Self) { self.bits |= rhs.bits; }
}

#[derive(Debug, Clone, Default)]
pub struct TextureDescriptor<'a> {
    pub label: Option<&'a str>,
    pub size: Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub view_formats: &'a [TextureFormat],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TextureDimension { #[default] D1, D2, D3 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFormat { #[default] Rgba8Unorm, Rgba8UnormSrgb, Bgra8Unorm, Bgra8UnormSrgb }
impl TextureFormat {
    pub fn is_srgb(&self) -> bool { false }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureUsages { pub bits: u32 }
impl TextureUsages {
    pub const COPY_SRC: Self = Self { bits: 1 };
    pub const COPY_DST: Self = Self { bits: 2 };
    pub const TEXTURE_BINDING: Self = Self { bits: 4 };
    pub const STORAGE_BINDING: Self = Self { bits: 8 };
    pub const RENDER_ATTACHMENT: Self = Self { bits: 16 };
}
impl std::ops::BitOr for TextureUsages {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { Self { bits: self.bits | rhs.bits } }
}

#[derive(Debug, Clone)]
pub struct TextureView {}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct TextureViewDescriptor<'a> {
    pub label: Option<&'a str>,
    pub format: Option<TextureFormat>,
    pub dimension: Option<TextureViewDimension>,
    pub aspect: TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TextureViewDimension { #[default] D1, D2, D2Array, Cube, CubeArray, D3 }

#[derive(Debug, Clone, Copy, Default)]
pub enum TextureAspect { #[default] All, StencilOnly, DepthOnly }


pub struct BindGroup {}
#[derive(Debug, Clone)]
pub struct ShaderModule {}

#[derive(Debug, Clone)]
pub struct ShaderModuleDescriptor<'a> {
    pub label: Option<&'a str>,
    pub source: ShaderSource<'a>,
}

#[derive(Debug, Clone)]
pub enum ShaderSource<'a> { Wgsl(std::borrow::Cow<'a, str>) }

#[derive(Debug, Clone)]
pub struct BindGroupLayoutDescriptor<'a> {
    pub label: Option<&'a str>,
    pub entries: &'a [BindGroupLayoutEntry],
}

#[derive(Debug, Clone)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: ShaderStages,
    pub ty: BindingType,
    pub count: Option<std::num::NonZeroU32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ShaderStages { pub bits: u32 }
impl ShaderStages {
    pub const VERTEX: Self = Self { bits: 1 };
    pub const FRAGMENT: Self = Self { bits: 2 };
    pub const COMPUTE: Self = Self { bits: 4 };
}

#[derive(Debug, Clone)]
pub enum BindingType {
    Buffer { ty: BufferBindingType, has_dynamic_offset: bool, min_binding_size: Option<std::num::NonZeroU64> },
    Sampler(SamplerBindingType),
    Texture { sample_type: TextureSampleType, view_dimension: TextureViewDimension, multisampled: bool },
    StorageTexture { access: StorageTextureAccess, format: TextureFormat, view_dimension: TextureViewDimension },
}
impl Default for BindingType {
    fn default() -> Self { BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None } }
}

#[derive(Debug, Clone, Copy)]
pub enum BufferBindingType { Uniform, Storage { read_only: bool } }

#[derive(Debug, Clone, Copy)]
pub enum SamplerBindingType { Filtering, NonFiltering, Comparison }

#[derive(Debug, Clone, Copy)]
pub enum TextureSampleType { Float { filterable: bool }, Depth, Sint, Uint }

#[derive(Debug, Clone, Copy)]
pub enum StorageTextureAccess { ReadOnly, WriteOnly, ReadWrite }

#[derive(Debug, Clone)]
pub struct PipelineLayoutDescriptor<'a> {
    pub label: Option<&'a str>,
    pub bind_group_layouts: &'a [&'a BindGroupLayout],
    pub push_constant_ranges: &'a [PushConstantRange],
    pub immediate_size: u32,
}
#[derive(Debug, Clone)]
pub struct BindGroupLayout {}
#[derive(Debug)]
pub struct PushConstantRange {}

#[derive(Debug, Clone)]
pub struct ComputePipelineDescriptor<'a> {
    pub label: Option<&'a str>,
    pub layout: Option<&'a PipelineLayout>,
    pub module: Option<&'a ShaderModule>,
    pub ptx_payload: Option<&'a str>,
    pub entry_point: Option<&'a str>,
    pub cache: Option<()>,
    pub compilation_options: PipelineCompilationOptions,
}
#[derive(Debug, Clone)]
pub struct PipelineLayout {}

#[derive(Debug, Clone)]
pub struct BindGroupDescriptor<'a> {
    pub label: Option<&'a str>,
    pub layout: &'a BindGroupLayout,
    pub entries: &'a [BindGroupEntry<'a>],
}

#[derive(Debug, Clone)]
pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub resource: BindingResource<'a>,
}
#[derive(Debug, Clone)]
pub enum BindingResource<'a> {
    Buffer(BufferBinding<'a>),
    BufferArray(&'a [BufferBinding<'a>]),
    TextureView(&'a TextureView),
    TextureViewArray(&'a [&'a TextureView]),
}
#[derive(Debug, Clone)]
pub struct BufferBinding<'a> {
    pub buffer: &'a super::Buffer,
    pub offset: u64,
    pub size: Option<std::num::NonZeroU64>,
}

#[derive(Debug, Clone, Default)]
pub struct ComputePassDescriptor<'a> {
    pub label: Option<&'a str>,
    pub timestamp_writes: Option<()>,
}

#[derive(Debug, Clone, Default)]
pub struct RenderPassDescriptor<'a> {
    pub timestamp_writes: Option<()>, 
    pub occlusion_query_set: Option<()>, 
    pub multiview_mask: Option<u32>,
    pub label: Option<&'a str>,
    pub color_attachments: &'a [Option<RenderPassColorAttachment<'a>>],
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment>,
}

#[derive(Debug, Clone)]
pub struct RenderPassColorAttachment<'a> {
    pub depth_slice: Option<u32>,
    pub view: &'a TextureView,
    pub resolve_target: Option<&'a TextureView>,
    pub ops: Operations<Color>,
}

#[derive(Debug, Clone)]
pub struct Operations<V> {
    pub load: LoadOp<V>,
    pub store: StoreOp,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadOp<V> { Clear(V), Load }

#[derive(Debug, Clone, Copy)]
pub enum StoreOp { Store, Discard }

#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Clone)]
pub struct RenderPassDepthStencilAttachment {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollType { Wait { timeout: Option<std::time::Duration>, submission_index: Option<u64> }, Poll }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollError { Timeout, WrongSubmissionIndex }

pub type SubmissionIndex = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferAsyncError {}

#[derive(Debug)]
pub struct BufferView<'a> { pub _marker: std::marker::PhantomData<&'a ()> }
impl<'a> BufferView<'a> { pub fn len(&self) -> usize { 0 } }
#[derive(Debug)]
pub struct BufferViewMut<'a> { pub _marker: std::marker::PhantomData<&'a mut ()> }
impl<'a> BufferViewMut<'a> { pub fn len(&self) -> usize { 0 } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositeAlphaMode { #[default] Auto, Opaque, PreMultiplied, PostMultiplied, Inherit }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresentMode { #[default] AutoVsync, AutoNoVsync, Fifo, FifoRelaxed, Immediate, Mailbox }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceError { Timeout, Outdated, Lost, OutOfMemory, Other }
impl std::fmt::Display for SurfaceError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "SurfaceError") } }
impl std::error::Error for SurfaceError {}

#[derive(Debug, Clone, Default)]
pub struct SurfaceConfiguration {
    pub usage: TextureUsages,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub present_mode: PresentMode,
    pub desired_maximum_frame_latency: u32,
    pub alpha_mode: CompositeAlphaMode,
    pub view_formats: Vec<TextureFormat>,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineCompilationOptions {}

pub struct PollStatus {}
impl PollStatus { pub fn wait_finished(&self) -> bool { true } }
