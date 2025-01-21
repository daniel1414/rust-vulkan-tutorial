use std::u64;

use vk::{KhrSurfaceExtension, KhrSwapchainExtension};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use winit::window::Window;
use anyhow::{anyhow, Result};

use crate::vulkan::framebuffer::create_framebuffers;
use crate::vulkan::instance::create_instance;
use crate::vulkan::physical_device::pick_physical_device;
use crate::vulkan::device::create_logical_device;
use crate::vulkan::render_pass::create_render_pass;
use crate::vulkan::swapchain::{create_swapchain, create_swapchain_image_views};
use crate::vulkan::pipeline::create_pipeline;
use crate::vulkan::commands::{create_command_buffers, create_command_pool};
use crate::vulkan::synchronization::create_sync_objects;
use vulkanalia::Version;

pub const MAX_FRAMES_IN_FLIGHT: usize = 3;
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
pub const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);


/// The Vulkan App
#[derive(Clone, Debug)]
pub struct App {
    pub(crate) entry: Entry,
    pub(crate) instance: Instance,
    pub(crate) data: AppData,
    pub(crate) device: Device,
    pub(crate) frame: usize,
}

impl App {
    
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&instance, &device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        create_command_pool(&instance, &device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;

        Ok(Self {entry, instance, data, device, frame: 0})
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {

        // Ensures that the GPU has finished executing the commands for the current frame
        // (rendering & presenting) before starting a new frame. This avoids overwriting 
        // resources (like command buffers and semaphores) that are still in use.
        self.device.wait_for_fences(&[self.data.command_completion_fences[self.frame]], 
            true, 
            u64::MAX)?;

        // This semaphore ensures synchronization between the swapchain and the rendering process.
        let this_frame_image_available_semaphore = 
            self.data.image_available_semaphores[self.frame];

        // More like a request to aquire an image - we get the index instantly, but this
        // doesn't mean the image is ready to be processed. It will be once the semaphore
        // is signaled, only then can we perform operations on the image itself.
        let image_index = self
            .device
            .acquire_next_image_khr(
                self.data.swapchain,
                u64::MAX,

                // When the GPU is finished aquiring the image, it will signal this
                // semahpore, which is used by the command buffer to wait until this 
                // image is ready for rendering.
                this_frame_image_available_semaphore,
                vk::Fence::null()
            )?.0 as usize;

        // If a fence exists and hasn't been signaled for this image, means the GPU
        // is still processing it.
        if !self.data.image_usage_fences[image_index as usize].is_null() {

            // So we need to wait for the GPU to finish its operations on this image before proceeding.
            println!("Waiting for an image({}) that is in use.", image_index);
            self.device.wait_for_fences(
                &[self.data.image_usage_fences[image_index as usize]],
                true,
                u64::MAX
            )?;
        }

        // Associates the fence for the current frame with the swapchain image 
        // to track its usage.
        self.data.image_usage_fences[image_index as usize] = 
            self.data.command_completion_fences[self.frame];

        let wait_semaphores = &[this_frame_image_available_semaphore];

        // The pipeline waits at the COLOR_ATTACHMENT_OUTPUT stage, which is where rendering
        // to the swapchain image occurs.
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()

            // The GPU will wait with processing this command buffer until this semaphore is
            // signaled and it is signaled when the GPU is finished aquiring the image
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)

            // The GPU will signal this semaphore when the command buffer is done executing,
            // which means the image will be fully rendered to. We need this semaphore to be
            // signaled in order to make the GPU wait for it, before presenting this image.
            .signal_semaphores(signal_semaphores);

        // This step ensures the fence associated with the current frame is ready for the next submission.
        // Resetting is mandatory for the fence to unsignal it so it can be signaled again
        // by the GPU when commands are finished executing in the queue.
        self.device.reset_fences(&[self.data.command_completion_fences[self.frame]])?;
        
        // Submission involved associating the command buffer with synchronization primitives
        // (semaphores and fences) to coordinate execution.
        self.device.queue_submit(
            self.data.graphics_queue, 
            &[submit_info], 

            // The fence is signaled by the GPU when all commands in the submitted command buffer
            // have been fully executed by the graphics queue.
            self.data.command_completion_fences[self.frame],
        )?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            
            // The GPU will wait until this sepahore is signaled and it will be signaled when
            // the command buffer above (rendering to the image) will be finished.
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);
            
        self.device.queue_present_khr(self.data.present_queue, &present_info)?;

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub unsafe fn destroy(&mut self) {
        self.data.command_completion_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));

        // Freeing the command buffers is not mandatory as they are freed automatically 
        // when the command pool is destroyed.
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.device.destroy_command_pool(self.data.command_pool, None);
        self.data.framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data.swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));

        self.device.destroy_swapchain_khr(self.data.swapchain, None);
        self.device.destroy_device(None);
        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger,None);
        }
        self.instance.destroy_surface_khr(self.data.surface, None);
        self.instance.destroy_instance(None);
    }    
}


/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
    pub(crate) messenger: vk::DebugUtilsMessengerEXT,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) swapchain_format: vk::Format,
    pub(crate) swapchain_extent: vk::Extent2D,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_images: Vec<vk::Image>,
    pub(crate) swapchain_image_views: Vec<vk::ImageView>,
    pub(crate) render_pass: vk::RenderPass,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) framebuffers: Vec<vk::Framebuffer>,
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,

    /// These semaphores corespond to swapchain images and are signaled 
    /// when the GPU has finished aquiring an image from the swapchain.
    /// Used to synchronize rendering operations with image availability.
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,

    /// These semaphores are signaled when the GPU has finished rendering 
    /// to a swapchain image. They synchronize rendering operations with the
    /// presentation engine, ensuring the image is ready to be presented.
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,

    /// Signaled by the GPU when all commands for the current frame have 
    /// finished executing. Ensures that the CPU does not overwrite
    /// or reuse resources still in use by the GPU.
    pub(crate) command_completion_fences: Vec<vk::Fence>,

    /// Fences associated with swapchain images currently in use by the GPU.
    /// Ensures that a swapchain image is not overwritten or reused 
    /// while it is still being processed.
    pub(crate) image_usage_fences: Vec<vk::Fence>,
}