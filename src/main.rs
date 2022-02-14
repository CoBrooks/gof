use std::{
    error::Error,
    fs::File, 
};

use simplelog::*;
#[macro_use] extern crate log;

use gof_lib::{
    buffer::*, 
    editor::Editor,
    config::*, syntax::SyntaxHighlighter
};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("./debug.log")?)?;

    // Get config
    let theme = ThemeDefinition::from_file("./config/themes/default.toml")?;
    let syn = SyntaxHighlighter::new(theme, "./config/syntax-defs/rust.toml")?;

    let args: Vec<String> = std::env::args().collect();
    
    let mut buffers: Vec<Buffer> = Vec::new();
    if let Some(path) = args.get(1) {
        match Buffer::open_file(syn, path) {
            Ok(buffer) => buffers.push(buffer),
            Err(e) => {
                error!("Error creating buffer: {:?}", e);
                panic!("{:?}", e);
            }
        }
    } else {
        buffers.push(Buffer::new(syn));
    }

    match Editor::new_from_buffers(buffers) {
        Ok(mut editor) => editor.run_app_loop(),
        Err(e) => {
            debug!("Error instantiating editor: {:?}.", e);
            panic!("{:?}", e);
        }
    }
}

