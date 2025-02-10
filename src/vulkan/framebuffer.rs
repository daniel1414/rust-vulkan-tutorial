use crate::app::AppData;
use anyhow::Result;

use vulkanalia::prelude::v1_0::*;

/// Creates a framebuffer for every swapchain image view.
/// Used by the graphics pipeline to render a whole frame.
/// 
/// A framebuffer is a collection of attachments (color, depth, stencil etc.)
/// used as the target for rendering operations.
/// 
/// Internally, a framebuffer references image views, which point to GPU memory
/// for the attachments.
/// 
/// Complexities arise due to compatibility requirements, resizing, multisampling, and synchronization.
pub unsafe fn create_framebuffers(
    device: &Device,
    data: &mut AppData
) -> Result<()> {

    data.framebuffers = data.swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[data.color_image_view, data.depth_image_view, *i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.render_pass)

                // Each attachment corresponds to one of the attachments
                // defined in the render pass. In this case the color attachment.
                // Multiple attachments allow for advanced techniques like deffered shading and post-processing.
                .attachments(attachments)

                // The framebuffer's dimensions MUST match the swapchain image's dimensions.
                .width(data.swapchain_extent.width)
                .height(data.swapchain_extent.height)

                // Corresponds to the number of layers in the images used by its attachments.
                // Multiple layers are used for rendering to cube maps, texture arrays, or VR applications.
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}