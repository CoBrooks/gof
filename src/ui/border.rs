use std::collections::HashMap;
use std::error::Error;
use std::io::Write;

use crossterm::{
    queue,
    cursor::MoveTo,
    style::{
        ContentStyle,
        PrintStyledContent
    }
};

const BORDER_CHARS_LIGHT: [&str; 12] = [ "─", "│", "┌", "┐", "└", "┘", "├", "┤", "┬", "┴", "┼", " " ];
const BORDER_CHARS_HEAVY: [&str; 12] = [ "━", "┃", "┏", "┓", "┗", "┛", "┣", "┫", "┳", "┻", "╋", " " ];

#[derive(Clone, Copy)]
enum BorderDirection {
    Horizontal = 0,
    Vertical = 1,
    CornerDownRight = 2,
    CornerDownLeft = 3,
    CornerUpRight = 4,
    CornerUpLeft = 5,
    TVerticalRight = 6,
    TVerticalLeft = 7,
    THorizontalDown = 8,
    THorizontalUp = 9,
    Junction = 10,
    Null = 13
}

pub enum BorderStyle {
    Light, 
    Heavy,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
struct Point(u16, u16);

impl From<(u16, u16)> for Point {
    fn from(p: (u16, u16)) -> Self {
        Point(p.0, p.1)
    }
}

pub struct Border {
    points: Vec<Point>,
    directions: HashMap<Point, BorderDirection>,
}

impl Border {
    pub fn new() -> Self {
        Self { points: Vec::new(), directions: HashMap::new() }
    }

    pub fn append(&mut self, other: Vec<(u16, u16)>) {
        for o in other {
            self.points.push(o.into());
            self.directions.insert(o.into(), BorderDirection::Null);
        }

        self.recalculate_directions();
    }

    pub fn recalculate_directions(&mut self) {
        for point in &self.points {
            self.directions.insert(*point, self.calculate_direction(*point));
        }
    }

    fn calculate_direction(&self, point: Point) -> BorderDirection {
        let Point(x, y) = point;

        // 0 = Top, 1 = Right, 2 = Down, 3 = Left
        let neighbors: [bool; 4] = [
            y != 0 && self.directions.contains_key(&Point(x, y - 1)),
            self.directions.contains_key(&Point(x + 1, y)),
            self.directions.contains_key(&Point(x, y + 1)),
            x != 0 && self.directions.contains_key(&Point(x - 1, y)),
        ];

        match neighbors {
            [false, true, false, _] | [false, _, false, true] => BorderDirection::Horizontal,
            [true, false, _, false] | [_, false, true, false] => BorderDirection::Vertical,
            [false, true, true, false] => BorderDirection::CornerDownRight,
            [false, false, true, true] => BorderDirection::CornerDownLeft,
            [true, true, false, false] => BorderDirection::CornerUpRight,
            [true, false, false, true] => BorderDirection::CornerUpLeft,
            [true, true, true, false] => BorderDirection::TVerticalRight,
            [true, false, true, true] => BorderDirection::TVerticalLeft,
            [false, true, true, true] => BorderDirection::THorizontalDown,
            [true, true, false, true] => BorderDirection::THorizontalUp,
            [true, true, true, true] => BorderDirection::Junction,
            [false, false, false, false] => BorderDirection::Null,
        }
    }

    pub fn draw<T: Write>(&self, queue: &mut T, style: BorderStyle, color: ContentStyle) -> Result<(), Box<dyn Error>> {
        for point in &self.points {
            let dir = self.directions[&point];
            let Point(x, y) = point;

            let c = match style {
                BorderStyle::Light => BORDER_CHARS_LIGHT[dir as usize],
                BorderStyle::Heavy => BORDER_CHARS_HEAVY[dir as usize],
            };

            queue!(
                queue, 
                MoveTo(*x, *y),
                PrintStyledContent(
                    color.apply(c)
                )
            )?;
        }

        Ok(())
    }
}
