use std::{collections::HashSet, usize};
use std::error::Error;
use std::io::Write;

use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::{
    queue,
    cursor::MoveTo,
    style::{
        Color,
        ContentStyle,
        PrintStyledContent,
        Stylize
    }, terminal::{Clear, ClearType},
};

use super::{
    border::{
        Border,
        BorderStyle
    },
    rect::Rect,
    window::{ 
        FillMode,
        Window,
        WindowAlignment, 
        WindowInfo,
        WindowMode,
        WindowRoot
    }
};

pub enum CursorUpdateMode {
    Absolute,
    RelativeToSelected
}

pub struct UI<STATE> {
    pub state: STATE,
    pub cursor_position: (u16, u16),
    windows: Vec<Box<dyn Window<STATE>>>,
    selected: usize,
    hidden: HashSet<usize>,
    window_bounds: Option<Vec<Rect>>,
    border: Option<Border>,
    recalculate: bool
}

impl<STATE: Clone> UI<STATE> {
    pub fn new(windows: Vec<Box<dyn Window<STATE>>>, state: STATE) -> Self {
        debug!("Creating new ui with {} windows.", windows.len());

        Self {
            state,
            cursor_position: (0, 0),
            windows, 
            selected: 0,
            hidden: HashSet::new(),
            window_bounds: None,
            border: None,
            recalculate: true
        }
    }

    pub fn windows(&self) -> Vec<&Box<dyn Window<STATE>>> {
        self.windows.iter()
            .enumerate()
            .filter(|(i, _)| !self.hidden.contains(i))
            .map(|(_, w)| w)
            .collect()
    }

    pub fn windows_mut(&mut self) -> Vec<&mut Box<dyn Window<STATE>>> {
        self.windows.iter_mut()
            .enumerate()
            .filter(|(i, _)| !self.hidden.contains(i))
            .map(|(_, w)| w)
            .collect()
    }

    pub fn selected(&self) -> &Box<dyn Window<STATE>> {
        &self.windows[self.selected]
    }
    
    pub fn selected_mut(&mut self) -> &mut Box<dyn Window<STATE>> {
        &mut self.windows[self.selected]
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn select_window(&mut self, index: usize) {
        self.selected = index;
    }

    pub fn select_next_window(&mut self) -> Result<(), Box<dyn Error>> {
        if self.hidden.len() == self.windows.len() {
            if self.windows.is_empty() {
                return Err("There are no windows.".into());
            } else {
                return Err("All windows are hidden.".into());
            }
        }

        let mut i = 0;
        loop {
            self.selected += 1;

            if self.selected >= self.windows.len() {
                self.selected = 0;
                i += 1;

                if i >= 2 {
                    panic!("infinite loop in UI::select_next_window()");
                }

                continue;
            }

            if self.hidden.contains(&self.selected) {
                continue;
            }

            if !self.selected().info().selectable {
                continue;
            } 

            break;
        }

        Ok(())
    }

    pub fn hide_window(&mut self, index: usize) {
        self.hidden.insert(index);

        if index == self.selected {
            self.select_next_window().unwrap();
        }

        self.recalculate = true;
    }

    pub fn show_window(&mut self, index: usize) {
        self.hidden.remove(&index);
        self.recalculate = true;
    }

    pub fn update_windows_state(&mut self) {
        let state = self.state.clone();

        for window in self.windows_mut() {
            window.update_state(&state);
        }
    }

    pub fn pass_input_to_selected(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let mut state = self.state.clone();
        self.selected_mut().handle_input(&mut state, code, modifiers).unwrap();

        self.state = state;
    }

    pub fn update_cursor_position(&mut self, position: (u16, u16), mode: CursorUpdateMode) {
        let (x, y) = position;

        match mode {
            CursorUpdateMode::Absolute => self.cursor_position = position,
            CursorUpdateMode::RelativeToSelected => {
                let selected = self.selected().get_bounds();
                let (window_x, window_y) = (selected.x, selected.y);

                self.cursor_position = (window_x + x + 1, window_y + y + 1);
            }
        }
    }

    pub fn draw_cursor<Q: Write>(&mut self, queue: &mut Q) -> Result<(), Box<dyn Error>> {
        let (x, y) = self.cursor_position;
        self.move_cursor(queue, x, y)
    }

    pub fn move_cursor_to_window_origin(&mut self) {
        self.update_cursor_position((0, 0), CursorUpdateMode::RelativeToSelected);
    }

    fn window_max_width(&self, window: Rect, used_space: &Vec<Rect>) -> Option<u16> {
        let Rect { x, y, height, .. } = window;

        let mut min_width = u16::MAX;
        for space in used_space {
            if x < space.x {
                if y + height >= space.y && y < space.y + space.height {
                    min_width = min_width.min(space.x - x);
                }
            }
        }

        if min_width != u16::MAX {
            Some(min_width)
        } else {
            None
        }
    }

    fn window_max_height(&self, window: Rect, used_space: &Vec<Rect>) -> Option<u16> {
        let Rect { x, y, width, .. } = window;

        let mut min_height = u16::MAX;
        for space in used_space {
            if y < space.y {
                if x >= space.x || x + width < space.x + space.width {
                    min_height = min_height.min(space.y - y);
                }
            }
        }

        if min_height != u16::MAX {
            Some(min_height)
        } else {
            None
        }
    }

    fn calculate_origin(&self, used_space: &Vec<Rect>, alignment: WindowAlignment, term_size: (u16, u16)) -> (u16, u16) {
        if used_space.is_empty() {
            match alignment {
                WindowAlignment::Top => (0, 0),
                WindowAlignment::Bottom => (0, term_size.1),
            }
        } else {
            let (max_width, max_height) = term_size;

            match alignment {
                WindowAlignment::Top => {
                    for y in 0..max_height {
                        for x in 0..max_width {
                            if !used_space.iter().any(|r| r.contains((x, y))) {
                                let x = x.min(term_size.0);
                                let y = y.min(term_size.1).max(1) - 1;

                                return (x, y);
                            }
                        }
                    }

                    unreachable!()
                },
                WindowAlignment::Bottom => {
                    for y in 1..max_height {
                        for x in 0..max_width {
                            if !used_space.iter().any(|r| r.contains((x, max_height - y))) {
                                let x = x.min(term_size.0).max(0);
                                let y = (max_height - y).min(term_size.1).max(1);

                                return (x, y);
                            }
                        }
                    }

                    unreachable!()
                }
            }
        }
    }

    fn get_window_dimensions(&self, info: WindowInfo, used_space: &Vec<Rect>) -> Result<Rect, Box<dyn Error>> {
        let term_dim = crossterm::terminal::size()?;

        let WindowInfo { root, mode, .. } = info;
        let (x, mut y) = match root {
            WindowRoot::Point { x, y } => (x, y),
            WindowRoot::Floating(alignment) => {
                self.calculate_origin(&used_space, alignment, term_dim)
            }
        };

        let (term_width, term_height) = term_dim;
        let term_height = term_height - 1;

        let (width, height) = match mode {
            WindowMode::Bounds { width, height } => {
                if let WindowRoot::Floating(WindowAlignment::Bottom) = root {
                    y -= height;
                }

                (width, height)
            },
            WindowMode::FillH { height } => {
                if let WindowRoot::Floating(WindowAlignment::Bottom) = root {
                    y -= height;
                }

                let rect: Rect = (x, y, 0, height).into();
                let width = self.window_max_width(rect, &used_space).unwrap_or(term_width - x);

                (width, height)
            },
            WindowMode::FillV { width } => {
                let rect: Rect = (x, y, width, 0).into();
                let height = self.window_max_height(rect, &used_space).unwrap_or(term_height - y);

                (width, height)
            },
            WindowMode::Fill(FillMode::FillHFirst) => {
                let rect: Rect = (x, y, 0, 0).into();
                let width = self.window_max_width(rect, &used_space).unwrap_or(term_width - x);

                let rect: Rect = (x, y, width, 0).into();
                let height = self.window_max_height(rect, &used_space).unwrap_or(term_height - y);

                (width, height)
            },
            WindowMode::Fill(FillMode::FillVFirst) => {
                let rect: Rect = (x, y, 0, 0).into();
                let height = self.window_max_height(rect, &used_space).unwrap_or(term_height - y);
                
                let rect: Rect = (x, y, 0, height).into();
                let width = self.window_max_width(rect, &used_space).unwrap_or(term_width - x);

                (width, height)
            },
            WindowMode::Fill(FillMode::MaxArea) => {
                // H First
                let rect: Rect = (x, y, 0, 0).into();
                let width_h = self.window_max_width(rect, &used_space).unwrap_or(term_width - x);

                let rect: Rect = (x, y, width_h, 0).into();
                let height_h = self.window_max_height(rect, &used_space).unwrap_or(term_height - y);
                let area_h = width_h * height_h;
                
                // V First
                let rect: Rect = (x, y, 0, 0).into();
                let height_v = self.window_max_height(rect, &used_space).unwrap_or(term_height - y);
                
                let rect: Rect = (x, y, 0, height_v).into();
                let width_v = self.window_max_width(rect, &used_space).unwrap_or(term_width - x);
                let area_v = width_v * height_v;

                if area_h > area_v {
                    (width_h, height_h)
                } else {
                    (width_v, height_v)
                }
            },
        };

        Ok(Rect { x, y, width, height })
    }

    pub fn recalculate_ui(&mut self) -> Result<(), Box<dyn Error>> {
        crossterm::execute!(std::io::stdout(), Clear(ClearType::All))?;
        info!("Recalculating UI...");

        let mut used_space: Vec<Rect> = Vec::new();
        let mut border = Border::new();

        for window in self.windows() {
            let bounds = self.get_window_dimensions(window.info(), &used_space)?;
            used_space.push(bounds);

            border.append(bounds.border_points());
        }

        for (window, bound) in self.windows_mut().iter_mut().zip(used_space.clone()) {
            window.set_bounds(bound);
            // debug!("window at {bound:#}.");
        }

        border.recalculate_directions();

        self.window_bounds = Some(used_space);
        self.border = Some(border);

        self.recalculate = false;
        // info!("Finished recalculating UI.");

        Ok(())
    }

    fn draw_content<T: Write>(&self, queue: &mut T) -> Result<(), Box<dyn Error>> {
        if let Some(bounds) = &self.window_bounds {
            for (window, bound) in self.windows().iter().zip(bounds) {
                let Rect { x, y, width, height } = *bound;
                self.move_cursor(queue, x + 1, y + 1)?;

                let lines = window.lines();
                let mut line_num = 1;

                for line in lines {
                    if line_num > height {
                        break;
                    }

                    self.move_cursor(queue, x + 1, y + line_num)?;

                    for (content, style) in line.iter_chunks() {
                        let content = if content.len() > width as usize {
                            &content[..width as usize]
                        } else {
                            content
                        };

                        queue!(
                            queue,
                            PrintStyledContent(
                                style.apply(content)
                            )
                        )?;
                    }

                    line_num += 1;
                }
            }
        } else {
            unreachable!()
        }

        Ok(())
    }

    fn draw_borders<T: Write>(&self, queue: &mut T) -> Result<(), Box<dyn Error>> {
        if let Some(border) = &self.border {
            border.draw(queue, BorderStyle::Light, ContentStyle::default().with(Color::Black))?;
        } else {
            unreachable!()
        }

        Ok(())
    }

    fn draw_titles<T: Write>(&self, queue: &mut T) -> Result<(), Box<dyn Error>> {
        if let Some(bounds) = &self.window_bounds {
            let titles: Vec<(&str, Option<ContentStyle>)> = self.windows().iter()
                .map(|w| (w.title(), w.title_style()))
                .collect();

            for ((title, style), bound) in titles.iter().zip(bounds) {
                let Rect { x, y, width, .. } = *bound;
                self.move_cursor(queue, x + 1, y)?;

                let style = style.unwrap_or_default();

                let content = if title.len() > width as usize - 2 {
                    &title[..width as usize - 2]
                } else {
                    title
                };

                queue!(
                    queue,
                    PrintStyledContent(style.apply(content))
                )?;
            }
        } else {
            unreachable!()
        }

        Ok(())
    }

    pub fn draw<T: Write>(&mut self, queue: &mut T) -> Result<(), Box<dyn Error>> {
        if self.window_bounds.is_none() || self.border.is_none() || self.recalculate {
            self.recalculate_ui()?;
        }

        self.clear_all(queue)?;

        self.draw_content(queue)?;
        self.draw_borders(queue)?;
        self.draw_titles(queue)?;

        // Now handled by Application::run()
        // queue.flush()?;

        Ok(())
    }

    pub fn clear_all<T: Write>(&self, queue: &mut T) -> Result<(), Box<dyn Error>> {
        queue!(queue, Clear(ClearType::All)).map_err(|e| e.into())
    }

    pub fn move_cursor<T: Write>(&self, queue: &mut T, x: u16, y: u16) -> Result<(), Box<dyn Error>> {
        queue!(queue, MoveTo(x, y)).map_err(|e| e.into())
    }
}
