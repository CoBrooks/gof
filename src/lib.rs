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
