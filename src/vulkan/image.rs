
use vulkanalia::prelude::v1_0::*;
use anyhow::*;
use std::fs::File;

use crate::app::AppData;

use super::{buffers::buffer::{create_buffer, get_memory_type_index}, commands::{begin_single_time_commands, end_single_time_commands}};
use std::ptr::copy_nonoverlapping as memcpy;

pub unsafe fn create_texture_image(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {

    let image = File::open("resources/rook.png")?;

    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels)?;

    let size = reader.output_buffer_size() as u64;
    let (width, height) = reader.info().size();

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance, device, data, size, 
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?;
    
    let memory = device.map_memory(
        staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(pixels.as_ptr(), memory.cast(), pixels.len());

    device.unmap_memory(staging_buffer_memory);

        let (texture_image, texture_image_memory) = create_image(
            instance, 
            device, 
            data, 
            width, 
            height, 
            vk::Format::R8G8B8A8_SRGB, 
            
            // vk::ImageTiling::LINEAR: Texels are laid out in a row-major order like the 
            //   pixels array (first row, second row, etc.). This means the individual texels
            //   can be easily accessed by the CPU.
            // vk::ImageTiling::OPTIMAL: Texels are laid out in an implementation defined order
            //   for optimal access (optimal for GPU access, depends on the implementation).
            //   Individual texels cannot be accessed by the CPU, as the layout is not intuitive.
            vk::ImageTiling::OPTIMAL, 

            // vk::ImageUsageFlags::SAMPLED: Allows us to access the image from the shader.
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

    data.texture_image = texture_image;
    data.texture_image_memory = texture_image_memory;

    transition_image_layout(
        device, 
        data, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageLayout::UNDEFINED, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL
    )?;

    copy_buffer_to_image(
        device, 
        data, 
        staging_buffer, 
        data.texture_image,
        width, 
        height
    )?;

    transition_image_layout(
        device, 
        data, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}


pub unsafe fn create_texture_image_view(
    device: &Device,
    data: &mut AppData,
) -> Result<()> {

    data.texture_image_view = create_image_view(device, data.texture_image, vk::Format::R8G8B8A8_SRGB)?;

    Ok(())
}


pub unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {

    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {width, height, depth: 1})
        .array_layers(1)
        .mip_levels(1)
        .format(format)
        .tiling(tiling)
        .usage(usage)

        // vk::ImageLayout::UNDEFINED: Not usable by the GPU and the very first transaction will
        //   discard the texels.
        // vk::ImageLayout::PREINITIALIZED: Not usable by the GPU, but the very first transition
        //   will preserve the texels.
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .samples(vk::SampleCountFlags::_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .flags(vk::ImageCreateFlags::empty());

    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);
    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(instance, data, properties, requirements)?);
    
    let memory = device.allocate_memory(&info, None)?;
    
    device.bind_image_memory(image, memory, 0)?;

    Ok((image, memory))
}

pub unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
) -> Result<vk::ImageView> {

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .format(format)
        .view_type(vk::ImageViewType::_2D)
        .subresource_range(subresource_range);

    let image_view = device.create_image_view(&info, None)?;
    
    Ok(image_view)
}

pub unsafe fn transition_image_layout(
    device: &Device,
    data: &AppData,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<()> {
    
    let (
        src_access_mask,
        dst_access_mask,
        src_stage_mask,
        dst_stage_mask,
    ) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        _ => return Err(anyhow!("Unsupported image layout transition!"))
    };

    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .old_layout(old_layout)
        .new_layout(new_layout)
        
        // We are not using the barrier to transfer ownership between queues.
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    device.cmd_pipeline_barrier(
        command_buffer,
        src_stage_mask,
        dst_stage_mask,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier]
    );

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

pub unsafe fn copy_buffer_to_image(
    device: &Device,
    data: &AppData,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, data)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)

        // 0 for row length and image height indicates that the pixels are tightly packed 
        // and there is no padding bytes between rows of the image.
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D {x: 0, y: 0, z: 0})
        .image_extent(vk::Extent3D {width, height, depth: 1});

    device.cmd_copy_buffer_to_image(
        command_buffer,
        buffer,
        image,

        // Indicates which layout the image is currently using.
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region]);

    end_single_time_commands(device, data, command_buffer)?;
    
    Ok(())
}