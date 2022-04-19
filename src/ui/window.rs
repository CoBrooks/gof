use std::error::Error;
use std::ops::Range;

use crossterm::style::ContentStyle;
use crossterm::event::{ KeyCode, KeyModifiers };

use super::rect::Rect;

#[derive(Debug, Clone)]
pub struct StyledChunk {
    start: usize,
    end: usize,
    style: ContentStyle
}

#[derive(Debug, Clone)]
pub struct StyledContent {
    chunks: Vec<StyledChunk>,
    content: String
}

impl StyledContent {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            content: String::new()
        }
    }

    pub fn from(content: String) -> Self {
        let chunks = vec![ StyledChunk { start: 0, end: content.len(), style: ContentStyle::default() } ];

        Self {
            chunks,
            content
        }
    }
    
    pub fn from_styled(content: String, style: ContentStyle) -> Self {
        let chunks = vec![ StyledChunk { start: 0, end: content.len(), style } ];

        Self {
            chunks,
            content
        }
    }

    pub fn style_range(&mut self, range: Range<usize>, style: ContentStyle) {
        let chunk = StyledChunk { start: range.start, end: range.end, style };

        for (i, c) in self.chunks.iter().enumerate() {
            // If the new chunk starts within an existing chunk...
            if c.start < chunk.start && chunk.start < c.end {
                let chunk_before = StyledChunk {
                    start: c.start,
                    end: chunk.start,
                    ..(*c)
                };

                // ...AND is contained entirely within that chunk 
                if chunk.end < c.end {
                    let chunk_after = StyledChunk {
                        start: chunk.end,
                        end: c.end,
                        ..(*c)
                    };

                    self.chunks[i] = chunk_before;
                    self.chunks.insert(i + 1, chunk);
                    self.chunks.insert(i + 2, chunk_after);
                // ...AND ends exactly where that chunk ends
                } else if chunk.end == c.end {
                    self.chunks[i] = chunk_before;
                    self.chunks.insert(i + 1, chunk);
                // ...AND ends within another chunk
                } else if chunk.end > c.end {
                    let (index, ends_in_chunk) = self.chunks
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.start < chunk.end && c.end > chunk.end)
                        .unwrap();

                    let chunk_after = StyledChunk {
                        start: chunk.end,
                        end: ends_in_chunk.end,
                        ..(*ends_in_chunk)
                    };

                    // remove "overwritten" chunks
                    self.chunks.drain(i..index);

                    self.chunks[i] = chunk_before;
                    self.chunks.insert(i + 1, chunk);
                    self.chunks.insert(i + 2, chunk_after);
                }

                break;
            }
        }
    }

    pub fn push(&mut self, content: String, style: ContentStyle) {
        let start = self.content.len();

        self.content.push_str(&content);
        self.style_range(start..self.content.len(), style);
    }

    pub fn iter_chunks(&self) -> std::vec::IntoIter<(&str, ContentStyle)> {
        let mut ret = Vec::new();

        for chunk in &self.chunks {
            ret.push((&self.content[chunk.start..chunk.end], chunk.style));
        }

        ret.into_iter()
    }

    pub fn slice(&self, range: Range<usize>) -> Self {
        let content = &self.content[range.clone()];
        let content = content.to_string();

        let mut chunks = Vec::new();

        for chunk in &self.chunks {
            // `chunk` lies entirely within `range`
            if chunk.start >= range.start && chunk.end <= range.end {
                chunks.push(chunk.clone());
                
                break;
            // `chunk` starts before, buts ends inside of `range`
            } else if chunk.start < range.start && chunk.end > range.start {
                let c = StyledChunk {
                    start: range.start,
                    ..(*chunk)
                };

                chunks.push(c);
                
                break;
            // `chunk` starts inside, buts ends beyond `range`
            } else if chunk.start < range.end && chunk.end > range.end {
                let c = StyledChunk {
                    end: range.end,
                    ..(*chunk)
                };

                chunks.push(c);

                break;
            }
        }

        Self {
            content,
            chunks
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FillMode {
    FillHFirst,
    FillVFirst,
    MaxArea
}

#[derive(Debug, Clone, Copy)]
pub enum WindowMode {
    Bounds { width: u16, height: u16 },
    FillH { height: u16 },
    FillV { width: u16 },
    Fill(FillMode)
}

#[derive(Debug, Clone, Copy)]
pub enum WindowRoot {
    Point { x: u16, y: u16 },
    Floating(WindowAlignment)
}

#[derive(Debug, Clone, Copy)]
pub enum WindowAlignment {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowInfo {
    pub root: WindowRoot,
    pub mode: WindowMode,
    pub selectable: bool
}

impl WindowInfo {
    pub fn new() -> Self {
        Self { 
            root: WindowRoot::Floating(WindowAlignment::Top), 
            mode: WindowMode::Fill(FillMode::FillHFirst),
            selectable: false
        }
    }

    pub fn origin(self, x: u16, y: u16) -> Self {
        Self {
            root: WindowRoot::Point { x, y },
            ..self
        }
    }
    
    pub fn bounds(self, width: u16, height: u16) -> Self {
        Self {
            mode: WindowMode::Bounds { width, height },
            ..self
        }
    }
    
    pub fn align(self, alignment: WindowAlignment) -> Self {
        Self {
            root: WindowRoot::Floating(alignment),
            ..self
        }
    }

    pub fn fill_horizontal(self, height: u16) -> Self {
        Self {
            mode: WindowMode::FillH { height },
            ..self
        }
    }
    
    pub fn fill_vertical(self, width: u16) -> Self {
        Self {
            mode: WindowMode::FillV { width },
            ..self
        }
    }

    pub fn fill(self) -> Self {
        Self {
            mode: WindowMode::Fill(FillMode::FillHFirst),
            ..self
        }
    }

    pub fn selectable(self) -> Self {
        Self {
            selectable: true,
            ..self
        }
    }
}

pub trait Window<STATE>: std::fmt::Debug {
    // Immutable required functions
    fn info(&self) -> WindowInfo;
    fn lines(&self) -> Vec<StyledContent>;
    fn get_bounds(&self) -> Rect;

    // Mutable required functions
    fn set_bounds(&mut self, new_bounds: Rect);
    
    // Optional functions
    fn title(&self) -> &str {
        ""
    }
    fn title_style(&self) -> Option<ContentStyle> {
        None
    }
    fn handle_input(&mut self, _state: &mut STATE, _code: KeyCode, _modifiers: KeyModifiers)
    -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn boxed(self) -> Box<dyn Window<STATE>>
    where Self: Sized + 'static {
        Box::new(self)
    }

    fn update_state(&mut self, _new_state: &STATE) { }
}
