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

use super::InputHandler;

type Term = Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>;
type In = Keys<AsyncReader>;
type Out = AlternateScreen<RawTerminal<Stdout>>;

pub struct Editor<'a> {
    pub buffers: Vec<Buffer<'a>>,
    input_handler: InputHandler,
    stdin: In,
    out: Out,
    terminal: Term
}

impl<'a> Editor<'a> {
    pub fn new_from_buffers(buffers: Vec<Buffer<'a>>, input_binds: &str) -> Result<Self, Box<dyn Error>> {
        let input_handler = InputHandler::new(input_binds)?;
        match Self::setup_terminal_crossterm() {
            Ok((stdin, out, terminal)) => {
                let ed = Editor {
                    buffers,
                    input_handler,
                    stdin,
                    out,
                    terminal
                };

                Ok(ed)
            },
            Err(e) => {
                debug!("Error setting up terminal: {:?}.", e);
                panic!("{:?}", e);
            }
        }
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

        if let Err(e) = self.draw_buffers(None) {
            error!("{:?}", e);
        }

        loop {
            // Async input handling
            if let Some(Ok(key)) = self.stdin.next() {
                let input;
                match key {
                    Key::Char('Q') => {
                        return Ok(()) 
                    },
                    _ => input = Some(key)
                }
                
                if let Err(e) = self.draw_buffers(input) {
                    error!("{:?}", e);
                }
            }

            // Maximum "Framerate" of the editor (threads spawned in this `loop` are concurrent)
            thread::sleep(Duration::from_secs_f32(1.0 / FPS));
        }
    }

    pub fn exit(mut self) -> Result<(), Box<dyn Error>> {
        // Ensure cursor is back to Solid Block
        write!(self.out, "{}", SteadyBlock)?;

        self.out.flush()?;

        Ok(())
    }

    fn draw_buffers(&mut self, input: Option<Key>) 
        -> Result<(), Box<dyn Error>> {

        let terminal = &mut self.terminal;

        for buffer in &mut self.buffers {
            if buffer.selected {
                if let Some(event) = input {
                    let action = self.input_handler.handle(&buffer.mode, event);
                    debug!("Handling action {:?}", action);

                    match buffer.mode {
                        EditorMode::Insert => buffer.handle_insert(event),
                        EditorMode::Delete => buffer.handle_delete(event),
                        EditorMode::Change => buffer.handle_change(event),
                        _ => {
                            if let Some(action) = action {
                                buffer.handle_action(action);
                            } else {
                                debug!("Unhandled key: {:?}.", event);
                            }
                        }
                    }
                }

                match buffer.mode {
                    EditorMode::Normal => write!(self.out, "{}", SteadyBlock),
                    EditorMode::Insert => write!(self.out, "{}", BlinkingBar),
                    EditorMode::Scroll => write!(self.out, "{}", BlinkingBlock),
                    EditorMode::Delete => write!(self.out, "{}", SteadyUnderline),
                    EditorMode::Change => write!(self.out, "{}", SteadyUnderline),
                }?;
            }
            
            if let Err(e) = terminal.draw(|f| buffer.draw(f)) {
                debug!("Failed to draw to terminal: {:?}", e);
            }
        }
        
        Ok(())
    }
}
