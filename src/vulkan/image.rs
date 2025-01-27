use vulkanalia::prelude::v1_0::*;
use anyhow::Result;
use std::fs::File;

use crate::app::AppData;

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

    Ok(())
}