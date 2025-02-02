
use std::{fs::File, io::BufReader};

use crate::app::AppData;
use anyhow::Result;

pub unsafe fn load_model(
    data: &mut AppData
) -> Result<()> {
    let mut reader = BufReader::new(File::open("resources/viking_room.obj")?);

    // We are interested only in the Vec<Model>, not in the Vec<Material>
    let (models, _) = tobj::load_obj_buf(
        &mut reader, 
        &tobj::LoadOptions { triangulate: true, ..Default::default() }, 
        |_| Ok(Default::default()),
    )?;

    Ok(())
}