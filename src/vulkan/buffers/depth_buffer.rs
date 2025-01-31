use vulkanalia::prelude::v1_0::*;
use anyhow::*;
use crate::app::AppData;

pub unsafe fn create_depth_objects(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {

        
    Ok(())
}

unsafe fn get_supported_format(
    instance: &Instance,
    data: &AppData,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {

    candidates
        .iter()
        .cloned()
        .find(| f| {
            let properties = instance.get_physical_device_format_properties(
                data.physical_device, *f);
            
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        }).ok_or_else(|| anyhow!("Failed to find supported format!"))
}