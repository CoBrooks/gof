use crossterm::style::{ ContentStyle, Color, Stylize };

use crate::ui::{
    rect::Rect,
    window::{ WindowInfo, Window, StyledContent }
};

#[derive(Debug)]
pub struct LineNumbers {
    info: WindowInfo,
    bounds: Option<Rect>
}

impl LineNumbers {
    pub fn new(info: WindowInfo) -> Self {
        LineNumbers { info, bounds: None }
    }
}

impl<STATE> Window<STATE> for LineNumbers {
    fn info(&self) -> WindowInfo {
        self.info
    }

    fn lines(&self) -> Vec<StyledContent> {
        let mut lines = Vec::new();
        let height = self.bounds.map(|r| r.height).unwrap();

        for i in 1..height {
            lines.push(StyledContent::from_styled(
                format!("{:3}\n", i), 
                ContentStyle::default().with(Color::Grey)
            ));
        }

        lines
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = Some(new_bounds);
    }
    fn get_bounds(&self) -> Rect {
        self.bounds.unwrap_or_default()
    }
}
