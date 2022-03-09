use std::fmt::Display;

use super::*;
use crate::{buffer::buffercontents::Direction as Dir, syntax::SyntaxHighlighter, editor::InputAction};

use termion::event::Key;
use tui::{text::{Span, Spans}, style::Modifier, layout::Rect};

#[derive(Debug, PartialEq, Clone)]
pub enum EditorMode {
    Normal,
    Insert,
    Scroll,
    Delete,
    Change
}

impl EditorMode {
    pub fn get_span(&self) -> Span {
        Span::styled(self.to_string().to_uppercase(), self.get_style())
    }

    fn get_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Blue,
            Self::Insert => Color::Magenta,
            Self::Scroll => Color::Yellow,
            Self::Delete => Color::Red,
            Self::Change => Color::Green,
        };

        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Normal => "normal",
            Self::Insert => "insert",
            Self::Scroll => "scroll",
            Self::Delete => "delete",
            Self::Change => "change",
        })
    }
}

#[derive(Debug)]
pub struct BufferLayout {
    pub line_numbers: Rect,
    pub text: Rect,
    pub status_bar: Rect,
    pub split: bool,
    layouts: Vec<Layout>,
}

impl Default for BufferLayout {
    fn default() -> Self {
        let layouts = vec![
            // splits content from status_bar
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(0),
                        Constraint::Length(3)
                    ].as_ref()
                ).margin(0),
            // splits line_numbers from text
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(5),
                        Constraint::Min(0),
                    ].as_ref()
                ).margin(0),
        ];

        Self {
            line_numbers: Rect::default(),
            text: Rect::default(),
            status_bar: Rect::default(),
            split: false,
            layouts
        }
    }
}

impl BufferLayout {
    pub fn split(&mut self, frame: Rect) {
        if let [ contents, status_bar ] = self.layouts[0].split(frame)[..] {
            if let [ line_numbers, text ] = self.layouts[1].split(contents)[..] {
                self.line_numbers = line_numbers;
                self.text = text;
                self.status_bar = status_bar;

                self.split = true;
            }
        } else {
            error!("Layout error");
        }
    }

    pub fn get_default_blocks<'a>(&'a self, buffer_name: Span<'a>) -> [Block<'a>; 3] {
        [
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::Black)),
            Block::default()
                .title(buffer_name)
                .borders(Borders::ALL & !Borders::LEFT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::DarkGray)),
        ]
    }
}

#[derive(Debug)]
pub struct Buffer<'a> {
    pub filepath: Option<PathBuf>,
    pub contents: BufferContents,
    pub syn: SyntaxHighlighter<'a>,
    pub selected: bool,
    pub layout: BufferLayout,
    pub mode: EditorMode,
    pub height: u16,
    pub cmd_accumulator: String,
}

impl<'a> Buffer<'a> {
    pub fn new(syn: SyntaxHighlighter<'a>) -> Self {
        debug!("Creating new empty buffer.");

        Buffer {
            filepath: None,
            contents: BufferContents::new(),
            syn,
            selected: true,
            layout: BufferLayout::default(),
            mode: EditorMode::Normal,
            height: 0,
            cmd_accumulator: String::new()
        }
    }

    pub fn open_file(syn: SyntaxHighlighter<'a>, filepath: &str) -> Result<Self, Box<dyn Error>> {
        debug!("Creating buffer from file: '{}'", filepath);

        let contents = BufferContents::load_file(filepath);
        let filepath: PathBuf = PathBuf::from_str(filepath)?;

        Ok(Buffer {
            filepath: Some(filepath),
            contents,
            syn,
            selected: true,
            layout: BufferLayout::default(),
            mode: EditorMode::Normal,
            height: 0,
            cmd_accumulator: String::new()
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.height = f.size().height - 2;

        if !self.layout.split {
            self.layout.split(f.size());
        }

        let block_name = if let Some(path) = &self.filepath {
            path.file_name().unwrap().to_str().unwrap()
        } else {
            "buffer"
        };

        let block_name = Span::styled(block_name, Style::default().fg(Color::Blue));
        
        let lines = self.contents.get_rendered_lines(self.height);
        let text = self.syn.highlight_lines(&lines);

        let [ numbers_block, text_block, status_block ] = self.layout.get_default_blocks(block_name);

        let contents = Paragraph::new(text.into_owned())
            .block(text_block.clone());
        f.render_widget(contents, self.layout.text);

        let cmd_acc = format!(" | {}", self.cmd_accumulator);
        let status_bar_contents = Spans::from(vec![ 
            Span::from(" "),
            self.mode.get_span(), 
            Span::from(cmd_acc) 
        ]);
        let status = Paragraph::new(status_bar_contents)
            .block(status_block);
        f.render_widget(status, self.layout.status_bar);

        let numbers = Paragraph::new(self.generate_line_numbers())
            .block(numbers_block.clone())
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(numbers, self.layout.line_numbers);

        let area = self.layout.text;
        let (cx, cy) = self.contents.drawn_cursor();
        let (x, y) = (area.left() + cx as u16, area.top() + cy as u16 + 1);

        f.set_cursor(x, y);
    }

    pub fn handle_action(&mut self, action: &InputAction) {
        match action {
            InputAction::NormalMode => self.mode = EditorMode::Normal,
            InputAction::MoveLeft => self.contents.move_cursor(Dir::Left, 1),
            InputAction::MoveDown => self.contents.move_cursor(Dir::Down, 1),
            InputAction::MoveUp => self.contents.move_cursor(Dir::Up, 1),
            InputAction::MoveRight => self.contents.move_cursor(Dir::Right, 1),
            InputAction::InsertMode => self.mode = EditorMode::Insert,
            InputAction::Append => { 
                self.contents.move_cursor(Dir::Right, 1);
                self.mode = EditorMode::Insert;
            },
            InputAction::InsertLineStart => {
                self.contents.move_cursor_line_start();
                self.mode = EditorMode::Insert;
            },
            InputAction::AppendLineEnd => {
                self.contents.move_cursor_line_end();
                self.mode = EditorMode::Insert;
            },
            InputAction::ScrollMode => self.mode = EditorMode::Scroll,
            InputAction::DeleteMode => {
                self.cmd_accumulator = "d".into();
                self.mode = EditorMode::Delete; 
            },
 
            InputAction::ChangeMode => {
                self.cmd_accumulator = "c".into();
                self.mode = EditorMode::Change; 
            },
            InputAction::Write => {
                if let Some(filepath) = &self.filepath {
                    match self.contents.save_file(filepath.to_str().unwrap()) {
                        Ok(_) => info!("Writing buffer to file {:?}", filepath),
                        Err(e) => error!("Error saving file: {:?}.", e),
                    }
                }
            },
            InputAction::ScrollDown => self.contents.scroll(1),
            InputAction::ScrollUp => self.contents.scroll(-1),
            InputAction::PageDown => self.contents.scroll(self.height as isize / 2),
            InputAction::PageUp => self.contents.scroll(-(self.height as isize) / 2),
            InputAction::TopOfBuffer => self.contents.move_to_top(),
            InputAction::BottomOfBuffer => self.contents.move_to_bottom(self.height),
            InputAction::Delete => self.contents.delete_key_handler(),
            InputAction::NoOp(_) => { },
        }
    }

    pub fn handle_insert(&mut self, key: Key) {
        match key {
            Key::Char('\n') => {
               self.contents.insert_character('\n');
               self.contents.move_cursor(Dir::Down, 1);
               self.contents.move_cursor_line_start();
            },
            Key::Char('\t') => {
                self.contents.insert_str(&" ".repeat(TAB_LENGTH));
                self.contents.move_cursor(Dir::Right, TAB_LENGTH);
            },
            Key::Char(c) => {
                self.contents.insert_character(c);
                self.contents.move_cursor(Dir::Right, 1);
            },
            Key::Backspace => {
                self.contents.backspace_key_handler();
                self.contents.move_cursor(Dir::Left, 1);
            },
            Key::Delete => {
                self.contents.delete_key_handler();
            },
            Key::Esc => {
                self.mode = EditorMode::Normal;
            },
            _ => error!("Unsupported key {:?}", key)
        }
    }

    pub fn handle_delete(&mut self, key: Key) {
        match key {
            Key::Char('d') => {
                self.contents.delete_current_line();
            },
            Key::Char('w') => {
                if self.cmd_accumulator == "a" || self.cmd_accumulator == "i" {
                    self.contents.delete_word(true);
                } else {
                    self.contents.delete_word(false);
                }
            },
            Key::Char(c) => {
                self.cmd_accumulator.push(c);
                return;
            }
            _ => { }
        }

        self.cmd_accumulator = String::new();
        self.mode = EditorMode::Normal;
    }

    pub fn handle_change(&mut self, key: Key) {
        match key {
            Key::Char('w') => {
                if self.cmd_accumulator == "a" || self.cmd_accumulator == "i" {
                    self.contents.delete_word(true);
                } else {
                    self.contents.delete_word(false);
                }
            },
            Key::Char(c) => {
                self.cmd_accumulator.push(c);
                return;
            },
            _ => { }
        }
        
        self.cmd_accumulator = String::new();
        self.mode = EditorMode::Insert;
    }

    fn generate_line_numbers(&self) -> String {
        // TODO: determined by cursor position
        let mut numbers: String = String::new();

        let len = self.contents.len();

        for i in self.contents.scroll_offset..len {
            numbers.push_str(&format!("{:>3}\n", i + 1));
        }

        for _ in len..len + self.height as usize {
            numbers.push_str("...\n");
        }

        numbers
    }
}
