use vulkanalia::prelude::v1_0::*;
use crate::app::AppData;
use anyhow::{anyhow, Result};
use log::*;
use super::queue::QueueFamilyIndices;


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
    QueueFamilyIndices::get(instance, data, physical_device)?;
    Ok(())
}