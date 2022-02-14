use std::io::BufWriter;

use super::*;

pub enum Direction {
    Left,
    Down,
    Up,
    Right
}

impl Direction {
    pub fn from_num_lines(n: isize) -> Self {
        if n > 0 { Direction::Down } else { Direction::Up }
    }
}

#[derive(Debug)]
pub struct BufferContents {
    pub rope: Rope,
    pub line: usize,
    pub col: usize,
    pub scroll_offset: usize
}

impl BufferContents {
    pub fn new() -> Self {
        debug!("Creating empty buffer.");

        let rope = Rope::new();

        Self {
            rope, 
            line: 0,
            col: 0,
            scroll_offset: 0,
        }
    }

    pub fn load_file(filepath: &str) -> Self {
        match File::open(filepath) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let rope = Rope::from_reader(reader).unwrap();

                debug!("Loaded contents of file {}", filepath);

                Self {
                    rope,
                    line: 0,
                    col: 0,
                    scroll_offset: 0
                }
            },
            Err(e) => {
                debug!("Error loading file: {:?}", e);
                Self::new()
            }
        }
    }

    pub fn save_file(&self, filepath: &str) -> Result<(), std::io::Error> {
        self.rope.write_to(
            BufWriter::new(
                File::create(filepath)?
            )
        )
    }

    pub fn join_lines(&self) -> String {
        format!("{}", self.rope)
    }

    pub fn get_rendered_lines(&self, height: u16) -> String {
        if let Ok(start) = self.rope.try_line_to_char(self.scroll_offset) {
            if let Ok(end) = self.rope.try_line_to_char((height + 1) as usize + self.scroll_offset) {
                format!("{}", self.rope.slice(start..end))
            } else {
                format!("{}", self.rope.slice(start..))
            }
        } else {
            unreachable!("Other navigation logic should prevent this.")
        }
    }

    pub fn len(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn char_idx(&self) -> usize {
        self.rope.line_to_char(self.line) + self.col
    }

    pub fn scroll(&mut self, lines: isize) {
        let new_scroll = self.scroll_offset as isize + lines;

        if new_scroll < 0 {
            self.scroll_offset = 0;
        } else if new_scroll - 1 > self.len() as isize {
            self.scroll_offset = self.len() - 1;
        } else {
            self.scroll_offset = new_scroll as usize;
        }

        self.move_cursor(Direction::from_num_lines(lines), lines.abs() as usize);
    }

    pub fn move_to_top(&mut self) {
        self.line = 0;
        self.scroll_offset = 0;
        self.col = 0;
    }
    
    pub fn move_to_bottom(&mut self, height: u16) {
        self.line = self.len() - 1;
        self.scroll_offset = self.len() - height as usize;
        self.col = 0;
    }

    pub fn move_cursor(&mut self, d: Direction, n: usize) {
        let mut new_line = self.line as isize;
        let mut new_col = self.col as isize;

        match d {
            Direction::Left => {
                new_col -= n as isize;
                if new_col < 0 {
                    new_line -= 1;
                }
            },
            Direction::Down => {
                new_line += n as isize;
            },
            Direction::Up => {
                new_line -= n as isize;
            },
            Direction::Right => {
                new_col += n as isize;
                if new_col > self.current_line_len() as isize - 1 {
                    new_col = self.current_line_len() as isize - new_col;
                    new_line += 1;
                }
            },
        }

        self.line = new_line.clamp(0, self.len() as isize) as usize;
        if new_col < 0 {
            new_col = self.current_line_len() as isize - new_col - 2;
        }
        self.col = new_col.clamp(0, self.current_line_len().max(1) as isize - 1) as usize;
    }

    pub fn current_line_len(&self) -> usize {
        self.rope.line(self.line).to_string().len()
    }

    pub fn current_line_str(&self) -> &str {
        self.rope.line(self.line).as_str().unwrap()
    }

    pub fn drawn_cursor(&self) -> (usize, usize) {
        (self.col, self.line - self.scroll_offset)
    }

    pub fn move_cursor_line_start(&mut self) {
        self.col = self.current_line_len() - self.current_line_str().trim_start().len();
    }
    
    pub fn move_cursor_line_end(&mut self) {
        self.col = self.current_line_str().trim_end().len();
    }

    pub fn insert_character(&mut self, c: char) {
        self.rope.insert_char(self.char_idx(), c);
    }
    
    pub fn insert_str(&mut self, s: &str) {
        self.rope.insert(self.char_idx(), s);
    }

    pub fn backspace_key_handler(&mut self) {
        self.rope.remove(self.char_idx()-1..self.char_idx());
    }
    
    pub fn delete_key_handler(&mut self) {
        self.rope.remove(self.char_idx()..self.char_idx()+1);
    }
}
