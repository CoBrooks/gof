use std::{path::PathBuf, error::Error, io::BufReader, fs::File, fmt::Display};

use crossterm::{style::{ ContentStyle, Color, Stylize, Attribute }, event::{KeyCode, KeyModifiers}};
use ropey::Rope;

use crate::ui::{
    rect::Rect,
    window::{ WindowInfo, Window, StyledContent },
};
use crate::AppState;

#[derive(Debug)]
pub struct Buffer {
    info: WindowInfo,
    bounds: Option<Rect>,
    filepath: PathBuf,
    cursor_position: (u16, u16),
    content: Rope
}

impl Buffer {
    pub fn new_empty_buffer(info: WindowInfo) -> Self {
        Buffer {
            info,
            bounds: None,
            filepath: PathBuf::new(),
            cursor_position: (0, 0),
            content: Rope::new()
        }
    }

    pub fn new<F>(info: WindowInfo, filepath: F) -> Result<Self, Box<dyn Error>>
    where F: Into<PathBuf> + Display {
        let mut b = Self::new_empty_buffer(info);
        b.load_file(filepath)?;

        Ok(b)
    }

    pub fn load_file<F>(&mut self, filepath: F) -> Result<(), Box<dyn Error>>
    where F: Into<PathBuf> + Display {
        debug!("Loading contents of file {}...", &filepath);

        match File::open(filepath.into()) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let rope = Rope::from_reader(reader).unwrap();

                self.content = rope;

                debug!("...Success!");
                Ok(())
            },
            Err(e) => {
                error!("Error loading file: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}

impl Window<AppState> for Buffer {
    fn info(&self) -> WindowInfo {
        WindowInfo {
            selectable: true,
            ..self.info
        }
    }

    fn lines(&self) -> Vec<StyledContent> {
        self.content.lines()
            .map(|l| StyledContent::from(l.to_string()))
            .collect()
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = Some(new_bounds);
    }
    fn get_bounds(&self) -> Rect {
        self.bounds.unwrap_or_default()
    }

    fn title(&self) -> &str {
        self.filepath.to_str().unwrap()
    }

    fn title_style(&self) -> Option<ContentStyle> {
        Some(
            ContentStyle::default()
                .with(Color::Blue)
                .attribute(Attribute::Bold)
        )
    }

    fn handle_input(&mut self, state: &mut AppState, code: KeyCode, _modifiers: KeyModifiers) 
    -> Result<(), Box<dyn Error>> {
        let (mut x, mut y) = self.cursor_position;
        let Rect { width, height, .. } = self.bounds.unwrap();

        match code {
            KeyCode::Char('h') | KeyCode::Left => {
                x = if x > 0 { x - 1 } else { 0 };
            },
            KeyCode::Char('j') | KeyCode::Down => {
                y = if y < height - 2 { y + 1 } else { height - 2 };
            },
            KeyCode::Char('k') | KeyCode::Up => {
                y = if y > 0 { y - 1 } else { 0 };
            },
            KeyCode::Char('l') | KeyCode::Right => {
                x = if x < width - 2 { x + 1 } else { width - 2 };
            },
            _ => { }
        }

        state.cursor_position = (x, y);

        Ok(())
    }

    fn update_state(&mut self, new_state: &AppState) {
        let AppState { cursor_position, .. } = new_state;
        self.cursor_position = *cursor_position;
    }
}
