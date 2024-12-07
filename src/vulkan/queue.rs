use crate::app::AppData;
use super::errors;
use vulkanalia::prelude::v1_0::*;
use anyhow::{Result, anyhow};

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics: u32
}

impl QueueFamilyIndices {
    pub unsafe fn get(
        instance: &Instance,
        data: &AppData,
        physical_device: vk::PhysicalDevice
    ) -> Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        if let Some(graphics) = graphics {
            Ok(Self { graphics }) 
        } else {
            Err(anyhow!(errors::SuitabilityError("Mssing required queue families.")))
        }
    }
}
