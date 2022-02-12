use std::{
    error::Error, 
    io::{
        Stdout, 
        Write 
    }, 
    thread,
    time::Duration
};

use termion::{
    AsyncReader,
    async_stdin,
    cursor::*,
    event::Key,
    input::{
        Keys,
        TermRead,
    },
    raw::{
        IntoRawMode,
        RawTerminal,
    }, 
    screen::AlternateScreen, 
};
use tui::{
    backend::TermionBackend,
    Terminal
};

use crate::*;
use buffer::*;

type Term = Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>;
type In = Keys<AsyncReader>;
type Out = AlternateScreen<RawTerminal<Stdout>>;

pub struct Editor {
    pub buffers: Vec<Buffer>,
    stdin: In,
    out: Out,
    terminal: Term
}

impl Editor {
    pub fn new_from_buffers(buffers: Vec<Buffer>) -> Result<Self, Box<dyn Error>> {
        let (stdin, out, terminal) = Self::setup_terminal_crossterm()?;

        let ed = Editor {
            buffers,
            stdin,
            out,
            terminal
        };

        Ok(ed)
    }

    fn setup_terminal_crossterm() -> Result<(In, Out, Term), Box<dyn Error>> {
        let stdin = async_stdin().keys();
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        // Getting stdout twice... 
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);

        Ok((stdin, stdout, terminal))
    }
    
    pub fn run_app_loop(&mut self)
        -> Result<(), Box<dyn Error>> {

        let mut input_queue: Vec<Key> = Vec::new();
        self.draw_buffers(&mut input_queue)?;

        loop {
            // Async input handling
            if let Some(Ok(key)) = self.stdin.next() {
                match key {
                    Key::Char('Q') => {
                        return Ok(()) 
                    },
                    _ => input_queue.push(key)
                }
                
                if let Err(e) = self.draw_buffers(&mut input_queue) {
                    error!("{:?}", e);
                }
            }

            // Maximum "Framerate" of the editor (threads spawned in this `loop` are concurrent)
            thread::sleep(Duration::from_secs_f32(1.0 / FPS));
        }
    }

    fn draw_buffers(&mut self, input_queue: &mut Vec<Key>) 
        -> Result<(), Box<dyn Error>> {

        let terminal = &mut self.terminal;

        for buffer in &mut self.buffers {
            if buffer.selected {
                if let Some(event) = input_queue.pop() {
                    buffer.handle_keypress(event);
                }

                let e = match buffer.mode {
                    EditorMode::Normal => write!(self.out, "{}", SteadyBlock),
                    EditorMode::Insert => write!(self.out, "{}", BlinkingBar),
                    EditorMode::Scroll => write!(self.out, "{}", BlinkingBlock),
                };

                if let Err(e) = e {
                    error!("{:?}", e);
                }
            }
            
            terminal.draw(|f| buffer.draw(f))?;
        }
        
        Ok(())
    }
}
