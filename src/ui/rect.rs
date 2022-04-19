use std::fmt::Display;

#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn area(&self) -> u16 {
        (self.width - self.x) * (self.height - self.y)
    }

    pub fn contains(&self, point: (u16, u16)) -> bool {
        let (x, y) = point;

        x >= self.x && x < self.x + self.width &&
            y >= self.y && y <= self.y + self.height
    }

    pub fn border_points(&self) -> Vec<(u16, u16)>{
        let mut border_points = Vec::new();

        for x in self.x..self.width + self.x {
            border_points.push((x, self.y));
            border_points.push((x, self.y + self.height));
        }

        for y in self.y..=self.height + self.y {
            border_points.push((self.x, y));
            border_points.push((self.width + self.x, y));
        }

        border_points
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Rect { x, y, width, height } = self;

        write!(f, "{width}x{height} @ ({x}, {y})")
    }
}

impl From<(u16, u16, u16, u16)> for Rect {
    fn from(r: (u16, u16, u16, u16)) -> Self {
        let (x, y, width, height) = r; 

        Self { x, y, width, height }
    }
}
