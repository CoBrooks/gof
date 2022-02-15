use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use termion::event::Key;
use toml::Value;

use strum_macros::EnumString;

use crate::buffer::EditorMode;

#[derive(EnumString, Debug)]
pub enum InputAction {
    NormalMode,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    InsertMode,
    Append,
    AppendLineEnd,
    InsertLineStart,
    ScrollMode,
    Write,
    ScrollDown,
    ScrollUp,
    PageDown,
    PageUp,
    TopOfBuffer,
    BottomOfBuffer,
}

pub struct InputHandler {
    modal_binds: HashMap<String, InputAction>,
    global_binds: HashMap<String, InputAction>,
}

impl InputHandler {
    pub fn new(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let contents = std::fs::read_to_string(filepath)?;
        let toml_val: Value = contents.parse::<Value>()?;

        let mut modal_binds: HashMap<String, InputAction> = HashMap::new();
        let mut global_binds: HashMap<String, InputAction> = HashMap::new();

        let modes = toml_val.as_table().unwrap();

        for mode in modes.keys() {
            let actions = toml_val[mode].as_table().unwrap();
            for (action, key) in actions {
                let key = key.as_str().unwrap();
                if mode == "global" {
                    global_binds.insert(format!("{}", key), InputAction::from_str(&action)?);
                } else {
                    modal_binds.insert(format!("{}_{}", mode, key), InputAction::from_str(&action)?);
                }
            }
        }

        debug!("{:?}", &modal_binds);
        debug!("{:?}", &global_binds);
        Ok(Self { modal_binds, global_binds })
    }

    pub fn handle(&self, mode: &EditorMode, key: Key) -> Option<&InputAction> {
        let key = Self::key_to_string(key).unwrap_or_default();
        if let Some(action) = self.global_binds.get(&key) {
            Some(action)
        } else {
            let formatted_key = format!("{}_{}", mode, key);
            self.modal_binds.get(&formatted_key)
        }
    }

    fn key_to_string(key: Key) -> Option<String> {
        Some(match key {
            Key::Alt(c) | Key::Char(c) | Key::Ctrl(c) => format!("{}", c),
            Key::F(n) => format!("F{}", n),
            Key::Esc => "esc".into(),
            Key::Backspace => "backspace".into(),
            Key::Delete => "delete".into(),
            Key::Left => "left".into(),
            Key::Down => "down".into(),
            Key::Up => "up".into(),
            Key::Right => "right".into(),
            _ => return None
        })
    }
}
