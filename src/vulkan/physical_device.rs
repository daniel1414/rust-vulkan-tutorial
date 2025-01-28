use std::collections::HashSet;

use vulkanalia::prelude::v1_0::*;
use crate::app::AppData;
use anyhow::{anyhow, Result};
use log::*;
use super::queue::QueueFamilyIndices;
use super::errors::SuitabilityError;
use super::swapchain::SwapchainSupport;

pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

pub unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, data, physical_device) {
            warn!("Skipping physical device ('{}'): {}", properties.device_name, error);
        } else {
            info!("Selected physical device  ('{}').", properties.device_name);
            data.physical_device = physical_device;
            return Ok(());
        }
    }
    Err(anyhow!("Failed to find a suitable physical device."))
}


pub unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    
    // Check if the physical device contains the needed queue indices (for graphics and presentation queues)
    QueueFamilyIndices::get(instance, data, physical_device)?;

    // Check if the physical device supports swapchains
    let support = SwapchainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")))
    }

    // Check if the physical device has the required extensions
    check_physical_device_extensions(instance, physical_device)?;

    let features = instance.get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotrophy.")));
    }

    Ok(())
}

pub unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError("Missing required device extensions!")))
    }
}