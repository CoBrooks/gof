use super::*;
use crate::{buffer::buffercontents::Direction as Dir, syntax::SyntaxHighlighter};

use termion::event::Key;
use tui::text::Span;

#[derive(Debug, PartialEq)]
pub enum EditorMode {
    Normal,
    Insert,
    Scroll
}

#[derive(Debug)]
pub struct Buffer {
    pub filepath: Option<PathBuf>,
    pub contents: BufferContents,
    pub syn: SyntaxHighlighter,
    pub selected: bool,
    pub layout: Layout,
    pub mode: EditorMode,
    pub height: u16,
}

impl Buffer {
    pub fn new(syn: SyntaxHighlighter) -> Self {
        debug!("Creating new empty buffer.");

        let contents = BufferContents::new();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(4),
                    Constraint::Min(0),
                ].as_ref()
            ).margin(0);

        Buffer {
            filepath: None,
            contents,
            syn,
            selected: true,
            layout,
            mode: EditorMode::Normal,
            height: 0,
        }
    }

    pub fn open_file(syn: SyntaxHighlighter, filepath: &str) -> Result<Self, Box<dyn Error>> {
        debug!("Creating buffer from file: '{}'", filepath);

        let contents = BufferContents::load_file(filepath);
        let filepath: PathBuf = PathBuf::from_str(filepath)?;

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(5),
                    Constraint::Min(0),
                ].as_ref()
            ).margin(0);

        Ok(Buffer {
            filepath: Some(filepath),
            contents,
            syn,
            selected: true,
            layout,
            mode: EditorMode::Normal,
            height: 0,
        })
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.height = f.size().height - 2;

        let block_name = if let Some(path) = &self.filepath {
            path.file_name().unwrap().to_str().unwrap()
        } else {
            "buffer"
        };

        let mut block_name = Span::from(block_name);
        block_name.style = Style::default().fg(Color::Blue);
        
        let chunks = self.layout.split(f.size());
        self.height = f.size().height - 2;

        let contents_block = Block::default()
            .title(block_name)
            .borders(Borders::ALL & !Borders::LEFT)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let lines = self.contents.get_rendered_lines(self.height);
        // let text = Text::from(lines);
        let text = self.syn.highlight_lines(&lines);
        
        let contents = Paragraph::new(text)
            .block(contents_block.clone());
        f.render_widget(contents, chunks[1]);

        let numbers_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::Black));

        let numbers = Paragraph::new(self.generate_line_numbers())
            .block(numbers_block.clone())
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(numbers, chunks[0]);

        let area = chunks[1];
        let (cx, cy) = self.contents.drawn_cursor();
        let (x, y) = (area.left() + cx as u16, area.top() + cy as u16 + 1);

        f.set_cursor(x, y);
    }

    pub fn handle_keypress(&mut self, key: Key) {
        let mode = &self.mode;

        match (mode, key) {
            // ---------
            // ALL MODES
            // ---------
            (_, Key::Esc) => {
                // self.contents.move_cursor(-1, 0); // same as vim (a - ESC keeps cursor in place,
                // i - ESC moves left)
                self.mode = EditorMode::Normal;
            },
            // -----------
            // NORMAL MODE 
            // -----------
            (EditorMode::Normal, Key::Char('h')) | 
            (_, Key::Left) => 
                self.contents.move_cursor(Dir::Left, 1),
            (EditorMode::Normal, Key::Char('j')) |
            (_, Key::Down) => 
                self.contents.move_cursor(Dir::Down, 1),
            (EditorMode::Normal, Key::Char('k')) | 
            (_, Key::Up) => 
                self.contents.move_cursor(Dir::Up, 1),
            (EditorMode::Normal, Key::Char('l')) |
            (_, Key::Right) => 
                self.contents.move_cursor(Dir::Right, 1),
            (EditorMode::Normal, Key::Char('i')) => 
                self.mode = EditorMode::Insert,
            (EditorMode::Normal, Key::Char('n')) => 
                self.mode = EditorMode::Scroll,
            (EditorMode::Normal, Key::Char('I')) => {
                self.contents.move_cursor_line_start();
                self.mode = EditorMode::Insert;
            },
            (EditorMode::Normal, Key::Char('a')) => {
                self.contents.move_cursor(Dir::Right, 1);
                self.mode = EditorMode::Insert;
            },
            (EditorMode::Normal, Key::Char('A')) => {
                self.contents.move_cursor_line_end();
                self.mode = EditorMode::Insert;
            },
            (EditorMode::Normal, Key::Char('W')) => {
                if let Some(filepath) = &self.filepath {
                    match self.contents.save_file(filepath.to_str().unwrap()) {
                        Ok(_) => info!("Writing buffer to file {:?}", filepath),
                        Err(e) => error!("Error saving file: {:?}.", e),
                    }
                }
            },
            // -----------
            // SCROLL MODE 
            // -----------
            (EditorMode::Scroll, Key::Char('k')) =>
                self.contents.scroll(-1),
            (EditorMode::Scroll, Key::Char('j')) =>
                self.contents.scroll(1),
            (EditorMode::Scroll, Key::Char('K')) =>
                self.contents.scroll(-(self.height as isize) / 2),
            (EditorMode::Scroll, Key::Char('J')) =>
                self.contents.scroll(self.height as isize / 2),
            (EditorMode::Scroll, Key::Char('g')) =>
                self.contents.move_to_top(),
            (EditorMode::Scroll, Key::Char('G')) =>
                self.contents.move_to_bottom(self.height),
            // -----------
            // INSERT MODE 
            // -----------
            (EditorMode::Insert, code) => {
                match code {
                    Key::Char('\n') => {
                       self.contents.insert_character('\n');
                       self.contents.move_cursor(Dir::Down, 1);
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
                    _ => panic!("Unsupported key {:?}", code)
                }
            },
            _ => { }
        }
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
