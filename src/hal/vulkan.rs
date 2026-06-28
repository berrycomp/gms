use super::{Adapter, AdapterInfo, Backend, DeviceType, AdapterInternal};
use ash::{vk, Entry, Instance, Device};
use std::sync::Arc;

pub struct VulkanAdapter {
    pub entry: Arc<Entry>,
    pub instance: Arc<Instance>,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_index: u32,
}

impl VulkanAdapter {
    pub fn request_device(&self) -> VulkanDevice {
        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(self.queue_family_index)
            .queue_priorities(&queue_priorities).build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info));

        let device = unsafe {
            self.instance
                .create_device(self.physical_device, &device_create_info, None)
                .expect("Failed to create Vulkan device")
        };

        let queue = unsafe { device.get_device_queue(self.queue_family_index, 0) };
        
        let device = Arc::new(device);

        VulkanDevice {
            device: device.clone(),
            queue: VulkanQueue {
                device: device.clone(),
                queue,
                family_index: self.queue_family_index,
            },
        }
    }
}

#[derive(Clone)]
pub struct VulkanDevice {
    pub device: Arc<Device>,
    pub queue: VulkanQueue,
}

#[derive(Clone)]
pub struct VulkanQueue {
    pub device: Arc<Device>,
    pub queue: vk::Queue,
    pub family_index: u32,
}

pub struct VulkanBuffer {
    pub device: Arc<Device>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: u64,
}

impl VulkanBuffer {
    pub fn map_memory(&self) -> *mut u8 {
        unsafe {
            self.device
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
                .expect("Failed to map buffer memory") as *mut u8
        }
    }

    pub fn unmap_memory(&self) {
        unsafe {
            self.device.unmap_memory(self.memory);
        }
    }
}

pub struct VulkanTexture {
    pub device: Arc<Device>,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

pub struct VulkanCommandEncoder {
    pub device: Arc<Device>,
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
}

impl VulkanCommandEncoder {
    pub fn new(device: Arc<Device>, queue_family_index: u32) -> Self {
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER).build();
        let command_pool = unsafe {
            device.create_command_pool(&pool_info, None).expect("Failed to create command pool")
        };

        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe {
            device.allocate_command_buffers(&alloc_info.build()).expect("Failed to allocate command buffer")[0]
        };

        Self {
            device,
            command_pool,
            command_buffer,
        }
    }

    pub fn begin(&self) {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT).build();
        unsafe {
            self.device.begin_command_buffer(self.command_buffer, &begin_info).expect("Failed to begin command buffer");
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.end_command_buffer(self.command_buffer).expect("Failed to end command buffer");
        }
    }

    pub fn submit(&self, queue: &VulkanQueue) {
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(std::slice::from_ref(&self.command_buffer)).build();
        unsafe {
            self.device.queue_submit(queue.queue, std::slice::from_ref(&submit_info), vk::Fence::null())
                .expect("Failed to submit command buffer");
            // Wait idle for simplicity in this basic initialization
            self.device.queue_wait_idle(queue.queue).expect("Failed to wait on queue");
        }
    }
}

pub fn enumerate_adapters() -> Vec<Adapter> {
    // Further implementation for querying physical devices via Entry and Instance
    // will go here.
    vec![]
}
