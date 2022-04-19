<<<<<<< HEAD
use std::{
    error::Error,
    fs::File, 
};

use simplelog::*;
#[macro_use] extern crate log;

use gof_lib::{
    buffer::*, 
    editor::Editor,
    config::*, 
    syntax::SyntaxHighlighter
=======
use std::error::Error;
use std::fs::File;
use std::time::Duration;

use crossterm::event::{ KeyEvent, KeyCode, read, poll, Event as InputEvent };
use simplelog::{WriteLogger, Config};

use gof_ui::{
    application::{ Application, Event },
    ui::{
        *,
        window::{ Window, WindowAlignment, WindowInfo },
    },
    windows::*, AppState,
>>>>>>> 13b061f (Beginning of re-write using custom ui framework)
};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
<<<<<<< HEAD
    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("./debug.log")?)?;

    // Get config
    let theme = ThemeDefinition::from_file("./config/themes/default.toml")?;
    let syn = SyntaxHighlighter::new(theme, "./config/syntax-defs/rust.toml")?;

    let args: Vec<String> = std::env::args().collect();
    
    let mut buffers: Vec<Buffer> = Vec::new();
    if let Some(path) = args.get(1) {
        match Buffer::open_file(syn, path) {
            Ok(buffer) => buffers.push(buffer),
            Err(e) => {
                error!("Error creating buffer: {:?}", e);
                panic!("{:?}", e);
            }
        }
    } else {
        buffers.push(Buffer::new(syn));
    }

    match Editor::new_from_buffers(buffers, "./config/keys.toml") {
        Ok(mut editor) => {
            editor.run_app_loop()?;
            editor.exit()
        }
        Err(e) => {
            debug!("Error instantiating editor: {:?}.", e);
            panic!("{:?}", e);
        }
    }
}

=======
    WriteLogger::init(log::LevelFilter::Debug, Config::default(), File::create("./debug.log")?)?;

    let windows: Vec<Box<dyn Window<AppState>>> = vec![ 
        Empty::new(
            WindowInfo::new()
                .fill_vertical(32),
            "[ DIR TREE ]".to_string(),
        ).boxed(),
        Tabs::new(
            WindowInfo::new()
                .fill_horizontal(2),
        ).boxed(),
        Empty::new(
            WindowInfo::new()
                .align(WindowAlignment::Bottom)
                .fill_horizontal(3),
            "[ MODE / COMMAND GUTTER ]".to_string(),
        ).boxed(),
        LineNumbers::new(
            WindowInfo::new()
                .fill_vertical(4)
        ).boxed(),
        Buffer::new(
            WindowInfo::new()
                .fill(),
            "./src/main.rs"
        )?.boxed()
    ];
    
    let mut state = AppState::new();
    state.open_files = vec![ "./src/main.rs".into() ];

    let app = Application::new(windows, state);

    app.run(
        |_| { },
        |ui| app_loop(ui)
    )
}

fn app_loop(ui: &mut UI<AppState>) -> Event {
    if poll(Duration::from_secs_f32(1.0 / 180.0)).unwrap() {
        match read().unwrap() {
            InputEvent::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => 
                return Event::Exit,

            InputEvent::Key(KeyEvent { code: KeyCode::Char('n'), .. }) => {
                ui.state.sidebar_toggle = !ui.state.sidebar_toggle;

                if ui.state.sidebar_toggle {
                    ui.show_window(0);
                } else {
                    ui.hide_window(0);
                }
            },

            InputEvent::Key(KeyEvent { code: KeyCode::Char('R'), .. }) | 
            InputEvent::Resize(_, _) =>
                return Event::RecalculateUI,

            InputEvent::Key(KeyEvent { code, modifiers }) => {
                ui.pass_input_to_selected(code, modifiers);
                ui.update_cursor_position(ui.state.cursor_position, CursorUpdateMode::RelativeToSelected);
            },

            _ => { },
        }

        Event::Draw
    } else {
        Event::Sleep
    }
}


>>>>>>> 13b061f (Beginning of re-write using custom ui framework)
