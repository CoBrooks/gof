use crossterm::style::{ Attribute, ContentStyle, Color, Stylize };

use crate::ui::{
    rect::Rect,
    window::{ WindowInfo, Window, StyledContent }
};

#[derive(Debug)]
pub struct Empty {
    title: String,
    info: WindowInfo,
    bounds: Option<Rect>,
}

impl Empty {
    pub fn new(info: WindowInfo, title: String) -> Self {
        Self {
            info,
            title,
            bounds: None,
        }
    }
}

impl<S> Window<S> for Empty {
    fn info(&self) -> WindowInfo {
        self.info
    }

    fn lines(&self) -> Vec<StyledContent> {
        Vec::new()
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn title_style(&self) -> Option<ContentStyle> {
        Some(
            ContentStyle::default()
                .with(Color::Blue)
                .attribute(Attribute::Bold)
        )
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = Some(new_bounds)
    }
    fn get_bounds(&self) -> Rect {
        self.bounds.unwrap_or_default()
    }
}
