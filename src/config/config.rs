use std::error::Error;
use std::collections::HashMap;

use tui::style::Color;
use toml::Value;

use crate::syntax::SyntaxTokens;

const COLOR_TYPES: &[&str] = &[ "number", "string", "comment", "fn-call", "class", "keyword", "keyword-alt", "property" ];

#[derive(Debug)]
pub struct ThemeDefinition {
    pub colors: HashMap<SyntaxTokens, Color>
}

impl ThemeDefinition {
    pub fn from_file(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let contents = std::fs::read_to_string(filepath)?;
        let toml_val: Value = contents.parse::<Value>()?;

        let mut colors: HashMap<SyntaxTokens, Color> = HashMap::new();
        for color in COLOR_TYPES {
            if let Some(val) = toml_val[color].as_str() {
                colors.insert((*color).into(), Self::string_to_color(val));
            }
        }

        Ok(Self { colors })
    }

    fn string_to_color(s: &str) -> Color {
        match s.to_lowercase().as_str() {
            "black"       => Color::Black,
            "red"         => Color::Red,
            "green"       => Color::Green,
            "yellow"      => Color::Yellow,
            "blue"        => Color::Blue,
            "magenta"     => Color::Magenta,
            "cyan"        => Color::Cyan,
            "gray"        => Color::Gray,
            "darkgray"    => Color::DarkGray,
            "lightred"    => Color::LightRed,
            "lightgreen"  => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue"   => Color::LightBlue,
            "lightmagenta"=> Color::LightMagenta,
            "lightcyan"   => Color::LightCyan,
            "white"       => Color::White,
            _             => Color::White
        }
    }
}
