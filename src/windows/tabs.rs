use std::path::PathBuf;

use crossterm::style::{ContentStyle, Stylize, Color};

use crate::ui::{
    rect::Rect,
    window::{ WindowInfo, Window, StyledContent }
};
use crate::AppState;

#[derive(Debug)]
pub struct Tabs {
    info: WindowInfo,
    bounds: Option<Rect>,
    open_files: Vec<PathBuf>,
    selected_file: usize,
}

impl Tabs {
    pub fn new(info: WindowInfo) -> Self {
        Self {
            info,
            bounds: None,
            open_files: Vec::new(),
            selected_file: 0
        }
    }
}

impl Window<AppState> for Tabs {
    fn info(&self) -> WindowInfo {
        self.info
    }

    fn lines(&self) -> Vec<StyledContent> {
        let before = {
            let mut acc = String::new();

            for i in 0..self.selected_file {
                acc += &format!(
                    "{} //", 
                    self.open_files[i]
                        .file_name().unwrap()
                        .to_str().unwrap());
            }

            acc
        };

        let selected = format!(
            " {}", 
            self.open_files[self.selected_file]
                .file_name().unwrap()
                .to_str().unwrap()
        );

        let after = {
            let mut acc = String::new();

            for i in self.selected_file+1..self.open_files.len() {
                acc += &format!(
                    "{} //", 
                    self.open_files[i]
                        .file_name().unwrap()
                        .to_str().unwrap());
            }

            acc
        };

        debug!("{before}{selected} // {after}");

        vec![
            // StyledContent::from(before),
            StyledContent::from_styled(selected, ContentStyle::default().with(Color::Blue)),
            StyledContent::from(after),
        ]
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = Some(new_bounds)
    }
    fn get_bounds(&self) -> Rect {
        self.bounds.unwrap_or_default()
    }

    fn update_state(&mut self, new_state: &AppState) {
        let AppState { open_files, selected_file, .. } = new_state;

        self.open_files = open_files.clone();
        self.selected_file = *selected_file;
    }
}
