use vulkanalia::prelude::v1_0::*;
use anyhow::Result;

use crate::app::AppData;

use super::queue::QueueFamilyIndices;
use super::buffers::index_buffer::INDICES;

/// A command pool is an object used to manage the memory allocation of command buffers.
/// Since command buffers are stored in GPU-accessible memory, the command pool
/// serves as the allocator for these buffers. Command pools also define the scope of 
/// command buffer lifecycle and help manage their reuse.
/// 
/// Command pools are needed for:
/// 
/// Efficient memory management: Instead of individually allocating GPU memory for each
///     command buffer, Vulkan uses pools for bulk allocation, reducing overhead.
/// Thread safety: Each thread typically gets its own command pool to avoid contention
///     during command buffer allocation and recording.
/// 
/// When a command pool is destroyed, all command buffers allocated from it 
/// are reset or destroyed automatically.
pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()

        // Options for a command pool:
        // 1. Transient: Optimized for short-lived command buffers.
        // 2. Resettable command buffers: Command buffers allocated from this pool
        //      can be individually reset, rather than resetting the entire pool.
        .flags(vk::CommandPoolCreateFlags::empty())

        // The command pool created is tied to a specific queue family, and thus
        // all the buffers allocated from it are tied to the same queue family as well.
        .queue_family_index(indices.graphics);

    data.command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}

/// A command buffer is a container that stores a sequence of GPU commands. These commands
/// tell Vulkan what to do, such as rendering, memory transfers, or pipeline state changes.
/// 
/// They are pre-recorded and submitted to a Vulkan queue for execution. This enables efficient
/// batching of commands and offloading work to the GPU.
/// 
/// Buffers are needed for:
/// 
/// Minimized overhead: Commands are once recorded and reused, avoiding expensive per-frame CPU processing.
/// Parallelism: Multiple threads can record commands into different command buffers 
///     simultaneously maximizing CPU utilization.
/// Synchronization: By organizing commands into discrete units (command buffers),
///     Vulkan can better synchronize GPU workloads and resource access.
/// 
/// Primary buffers: Can be submitted directly to the queue - used for 
///     high-level commands and orchestration.
/// Secondary buffers: Cannot be submitted directly: they must be executed 
///     by a primary command buffer. Useful for splitting work across threads 
///     (recording different parts of a scene)
/// 
pub unsafe fn create_command_buffers(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&alloc_info)?;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty())
            .inheritance_info(&inheritance);

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.02, 0.02, 0.02, 1.0]
            }
        };

        let clear_values = &[color_clear_value];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);
        
        
        device.begin_command_buffer(*command_buffer, &info)?;
            device.cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
        
            // The command buffer tracks state changes (e.g., pipeline bindings) and
            // ensures dependencies are managed correctly.
            // The pipeline is meant to operate on attachments and the render pass describes them
            // so the pipeline needs to be bound only after the render pass begins.
            device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);
            device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
            device.cmd_bind_index_buffer(*command_buffer, data.index_buffer, 0, vk::IndexType::UINT16);
            device.cmd_draw_indexed(*command_buffer, INDICES.len() as u32,
                1, 0, 0, 0);
            device.cmd_end_render_pass(*command_buffer);
        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}