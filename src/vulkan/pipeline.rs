use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::bytecode::Bytecode;

use crate::app::AppData;

use super::vertex::Vertex;

/// The graphics pipeline in Vulkan is a sequence of steps that the GPU follows to 
/// transform input data (like vertex positions) into a rendered image on a framebuffer.
/// It consists of several fixed-function stages (like input assembly and rasterization)
/// and programmable stages (vertex and fragment shaders).
/// 
/// The pipeline specifies how to render to the render pass' attachments, including:
/// 1. How to write to the color attachment (via fragment shaders)
/// 2. Whether to use depth/stencil attachments for testing.
/// 
/// The pipeline is linked to a specific subpass of the render pass.
/// Multiple pipelines can be used in different subpasses, each with its own configuration.
/// 
/// Must output data to the color attachment in the right format (same as in the render pass
/// and swapchain).
pub unsafe fn create_pipeline(
    device: &Device,
    data: &mut AppData
) -> Result<()> {
    let vert = include_bytes!("shaders/vert.spv");
    let frag = include_bytes!("shaders/frag.spv");

    let vert_module = create_shader_module(device, vert)?;
    let frag_module = create_shader_module(device, frag)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // Area of the framebuffer to render to. In our case the whole area.
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    // Area of the framebuffer that fragments are allowed to affect. In our case the whole area.
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D {x: 0, y: 0})
        .extent(data.swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    // The rasterization state divides polygons into fragments (which end up being pixels on the screen)
    // and performs fragment culling - removing fragments that don't make it into the view.
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    
    // Blending new fragments with the existing ones in the framebuffer.
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    // The pipeline layout is like a blueprint that defines:
    // 1. Descriptor sets: How resources like textures and uniform buffers are accessed 
    //    by the shaders.
    // 2. Push constants: Small amounts of data sent to shaders for per-draw customization.
    let set_layouts = &[data.descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts);

    data.pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(data.pipeline_layout)

        // Link this pipeline to the correct render pass.
        .render_pass(data.render_pass)

        // And the right subpass.
        .subpass(0);

    data.pipeline = device.create_graphics_pipelines(vk::PipelineCache::null(), 
        &[info], None)?.0[0];

    device.destroy_shader_module(vert_module, None);
    device.destroy_shader_module(frag_module, None);

    Ok(())
}

unsafe fn create_shader_module(
    device: &Device,
    bytecode: &[u8],
) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();
    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    let module = device.create_shader_module(&info, None)?;
    Ok(module)
}