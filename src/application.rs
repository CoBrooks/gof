use std::{
    io::{
        stdout, 
        Stdout, 
        BufWriter, Write,
    }, 
    error::Error, 
};

use crossterm::{
    execute, 
    cursor::MoveTo, 
    terminal::{
        ClearType, 
        Clear, 
        enable_raw_mode,
        disable_raw_mode, 
        EnterAlternateScreen, 
        LeaveAlternateScreen
    }, 
};

use crate::ui::{ UI, window::Window };

pub enum Event {
    Draw,
    RecalculateUI,
    Sleep,
    Exit
}

pub struct Application<STATE> {
    pub ui: UI<STATE>,
    queue: BufWriter<Stdout>
}

impl<STATE: Clone> Application<STATE> {
    pub fn new(windows: Vec<Box<dyn Window<STATE>>>, state: STATE) -> Self {
        let ui = UI::new(windows, state);

        execute!(
            stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).unwrap();
        
        Self {
            ui,
            queue: BufWriter::new(stdout())
        }
    }

    pub fn run<START, LOOP>(mut self, mut on_start: START, mut main_loop: LOOP) -> Result<(), Box<dyn Error>> 
    where START: FnMut(&mut UI<STATE>),
          LOOP: FnMut(&mut UI<STATE>) -> Event,
    {
        self.ui.recalculate_ui()?;
        self.ui.select_next_window()?;
        self.ui.update_windows_state();

        enable_raw_mode()?;

        self.draw()?;
        self.ui.move_cursor_to_window_origin();
        self.ui.draw_cursor(&mut self.queue)?;

        on_start(&mut self.ui);
        self.queue.flush()?;

        loop {
            let loop_res = main_loop(&mut self.ui);

            self.ui.update_windows_state();

            match loop_res {
                Event::Draw => {
                    self.draw()?;
                },
                Event::RecalculateUI => {
                    self.ui.recalculate_ui()?;
                    self.draw()?;
                },
                Event::Exit => {
                    break;
                },
                Event::Sleep => { }
            }

            self.ui.draw_cursor(&mut self.queue)?;

            self.queue.flush()?;
        }

        self.exit()
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.ui.draw(&mut self.queue)?;
        Ok(())
    }

    pub fn exit(&self) -> Result<(), Box<dyn Error>> {
        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            LeaveAlternateScreen
        )?;

        disable_raw_mode()?;

        Ok(())
    }
}
