use vulkanalia::prelude::v1_0::*;
use anyhow::*;

use crate::app::AppData;

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        
        // This buffer will be used only by a single queue (the graphics queue), 
        // so we can make it exclusive for better performance.
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    // This creates a buffer handle, but no memory is allocated for it yet.
    let buffer = device.create_buffer(&buffer_info, None)?;

    // These are the requirements for the buffer that we need to find suitable memory for.
    // The memory must be big enough and satisfy the right requirements (needs to have
    // the right memory type bits set).
    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_type_index = get_memory_type_index(instance, data, properties, requirements)?;
    
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    // If the offset happens to be non-zero, it must be divisible by requirements.alignment.
    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

/// Returns a memory type index for memory that satisfies the given requirements
/// and has the given properties.
unsafe fn get_memory_type_index(
    instance: &Instance,
    data: &mut AppData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory: vk::PhysicalDeviceMemoryProperties = instance.get_physical_device_memory_properties(data.physical_device);

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type: vk::MemoryType = memory.memory_types[*i as usize];

            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

/// Copies data from one buffer to another.
/// The source buffer has to have the vk::BufferUsageFlags::TRANSFER_SRC
/// and the destination buffer has to have the VK::BufferUsageFlags::TRANSFER_DST flags.
pub unsafe fn copy_buffer(
    device: &Device,
    data: &AppData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {

    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_pool)
        .command_buffer_count(1);

    let command_buffer = 
        device.allocate_command_buffers(&info)?[0];
    
    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    let region = vk::BufferCopy::builder().size(size);
    
    device.begin_command_buffer(command_buffer, &info)?;
        device.cmd_copy_buffer(command_buffer, source, destination, &[region]);
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder()
        .command_buffers(command_buffers);

    device.queue_submit(data.graphics_queue, &[info], vk::Fence::null())?;
    device.queue_wait_idle(data.graphics_queue)?;

    device.free_command_buffers(data.command_pool, command_buffers);

    Ok(())
}