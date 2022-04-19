<<<<<<< HEAD
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

=======
use std::path::PathBuf;

#[macro_use] extern crate log;

pub mod ui;
pub mod application;
pub mod windows;

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub sidebar_toggle: bool,
    pub cursor_position: (u16, u16),
    pub open_files: Vec<PathBuf>,
    pub selected_file: usize
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
>>>>>>> 13b061f (Beginning of re-write using custom ui framework)
