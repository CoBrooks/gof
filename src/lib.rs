use std::{
    error::Error,
    path::PathBuf,
    str::FromStr,
    fs::File, 
    io::BufReader,
};

use ropey::Rope;
use tui::{
    backend::Backend,
    layout::{
        Constraint, 
        Layout,
        Direction
    },
    widgets::{
        Block,
        Borders, 
        Paragraph, 
        BorderType
    }, 
    Frame, 
    style::{
        Style, 
        Color
    }
};

#[macro_use] extern crate log;

// Anything below 60 is noticeably slow
pub const FPS: f32 = 120.0;
pub const TAB_LENGTH: usize = 4;

fn is_word_boundary(c: &char) -> bool {
    !(c.is_ascii_alphanumeric() || ['_', '-'].contains(c))
}

pub mod buffer;
pub mod editor;
pub mod syntax;
pub mod config;

