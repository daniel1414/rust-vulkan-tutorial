use vulkanalia::prelude::v1_0::*;
use anyhow::*;
use std::ptr::copy_nonoverlapping as memcpy;

use crate::app::AppData;

use super::vertex::{Vertex, VERTICES};

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData
) -> Result<()> {

    let buffer_info = vk::BufferCreateInfo::builder()
        
        // The buffer needs to hold all our vertex data.
        .size((size_of::<Vertex>() * VERTICES.len()) as u64)

        // And the buffer will be used as vertex data to be consumed by the pipeline.
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        
        // This buffer will be used only by the graphics queue, so we can make it exclusive.
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    // We're not allocating anything yet.
    data.vertex_buffer = device.create_buffer(&buffer_info, None)?;

    // These are the requirements for the buffer that we need to find suitable memory for.
    // The memory must be big enough and satisfy the right requirements (needs to have
    // the right memory type bits set).
    let requirements = device.get_buffer_memory_requirements(data.vertex_buffer);

    let memory_properties = 
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE;

    let memory_type_index = get_memory_type_index(instance, data, memory_properties, requirements)?;
    
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);

    data.vertex_buffer_memory = device.allocate_memory(&memory_info, None)?;

    // If the offset happens to be non-zero, it must be divisible by requirements.alignment.
    device.bind_buffer_memory(data.vertex_buffer, data.vertex_buffer_memory, 0)?;
    
    // This command allows us to access a region of the specified memory resource 
    // defined by an offset and size.
    let memory = device.map_memory(
        data.vertex_buffer_memory, 0, buffer_info.size, vk::MemoryMapFlags::empty())?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());

    // Need to unmap after mapping.
    device.unmap_memory(data.vertex_buffer_memory);

    Ok(())
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