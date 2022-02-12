use std::{
    error::Error,
    fs::File, 
};

use simplelog::*;

use gof_lib::{buffer::*, editor::Editor};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("./debug.log")?)?;

    let args: Vec<String> = std::env::args().collect();
    
    let mut buffers: Vec<Buffer> = Vec::new();
    if let Some(path) = args.get(1) {
        buffers.push(
            Buffer::open_file(path)?
        );
    } else {
        buffers.push(
            Buffer::new()?
        );
    }

    let mut editor = Editor::new_from_buffers(buffers)?;

    editor.run_app_loop()
}

