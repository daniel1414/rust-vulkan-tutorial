#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod app;
mod vulkan;

use app::App;
use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

fn main() -> Result<()> {
    pretty_env_logger::init();

    log::trace!("a trace example");
    log::debug!("deboogging");
    log::info!("such information");
    log::warn!("o_O");
    log::error!("boom");

    // Window

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Vulkan here we goo!!!")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event, .. } => { 
                match event {
                    WindowEvent::RedrawRequested if !elwt.exiting() => unsafe { app.render(&window) }.unwrap(),
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                        unsafe { app.device.device_wait_idle().unwrap(); }
                        unsafe { app.destroy(); }
                    }
                    _ => ()
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
