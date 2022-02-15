use std::collections::BTreeMap;
use std::{collections::HashMap, ops::Range};
use std::error::Error;

use regex::Regex;
use toml::Value;
use tui::style::{ Color, Style };
use tui::text::{Text, Span, Spans};

use crate::config::ThemeDefinition;

const SYNTAX_TOKENS: &[&str] = &[ "comment", "string", "keyword-alt", "keyword", "class", "fn-call", "property", "number" ];

#[derive(Hash, PartialEq, PartialOrd, Debug, Eq, Ord)]
pub enum SyntaxTokens {
    Comment,
    String,
    Class,
    FnCall,
    Property,
    Keyword,
    KeywordAlt,
    Number,
}

impl From<&str> for SyntaxTokens {
    fn from(s: &str) -> SyntaxTokens {
        match s {
            "number" => Self::Number,
            "property" => Self::Property,
            "fn-call" => Self::FnCall,
            "class" => Self::Class,
            "keyword" => Self::Keyword,
            "keyword-alt" => Self::KeywordAlt,
            "string" => Self::String,
            "comment" => Self::Comment,
            _ => panic!()
        }
    }
}

#[derive(Debug)]
pub struct SyntaxHighlighter {
    name: String,
    extension: String,
    theme: ThemeDefinition,
    regex_sets: BTreeMap<SyntaxTokens, Vec<Regex>>
}

impl SyntaxHighlighter {
    pub fn new(theme: ThemeDefinition, syntax_def_filepath: &str) -> Result<Self, Box<dyn Error>> {
        let contents = std::fs::read_to_string(syntax_def_filepath)?;
        let toml_val = contents.parse::<Value>()?;

        let regex_sets = Self::get_sets_from_toml(&toml_val["syntax"])?;
        let info = toml_val["info"].as_table().unwrap();
        let name = info["name"].to_string();
        let extension = info["extension"].to_string();

        Ok(Self {
            name,
            extension,
            theme, 
            regex_sets
        })
    }

    fn get_sets_from_toml(toml: &Value) -> Result<BTreeMap<SyntaxTokens, Vec<Regex>>, Box<dyn Error>> {
        let mut sets: BTreeMap<SyntaxTokens, Vec<Regex>> = BTreeMap::new();
        for token in SYNTAX_TOKENS {
            let mut set: Vec<Regex> = Vec::new();
            if let Some(regex_vals) = toml[token].as_array() {
                for val in regex_vals {
                    if let Some(expr) = val.as_str() {
                        set.push(Regex::new(expr)?);
                    }
                }
            }
            sets.insert((*token).into(), set);
        }

        Ok(sets)
    }

    pub fn highlight_lines<'a>(&self, lines: &'a str) -> Text<'a> {
        let mut highlighted_lines: Vec<Spans> = Vec::new();

        let mut highlighted_ranges: HashMap<Range<usize>, Color> = HashMap::new();

        for (token, exprs) in &self.regex_sets {
            for expr in exprs {
                if expr.is_match(lines) {
                    for mat in expr.captures_iter(lines).filter_map(|c| c.get(1)) {
                        let inside_other_match = highlighted_ranges.keys()
                            .find_map(|k| if k.contains(&mat.start()) { Some(true) } else { None })
                            .unwrap_or(false);
                        // debug!("{} = {:?} @ [{}..{}] ({})", mat.as_str(), token, mat.start(), mat.end(), inside_other_match);
                        if !inside_other_match {
                            highlighted_ranges.insert(mat.start()..mat.end(), self.theme.colors[token]);
                        }
                    }
                }
            }
        }

        // debug!("Highlighted Ranges: {:?}", highlighted_ranges);

        let mut word_acc: String = String::new();
        let mut line_acc: Vec<Span> = Vec::new();
        let mut current_color_index: (Color, usize) = (Color::White, lines.len());
        for (i, c) in lines.char_indices() {
            let k = highlighted_ranges.keys()
                .find_map(|k| if k.start == i { Some(k) } else { None });
            if let Some(key) = k {
                line_acc.push(Span::styled(word_acc, Style::default().fg(current_color_index.0)));
                word_acc = String::new();
                current_color_index = (highlighted_ranges[key], key.end);
            }

            word_acc.push(c);

            if current_color_index.1 - 1 == i {
                line_acc.push(Span::styled(word_acc, Style::default().fg(current_color_index.0)));
                word_acc = String::new();
                current_color_index = (Color::White, lines.len()); 
            }

            if c == '\n' {
                line_acc.push(Span::styled(word_acc, Style::default().fg(current_color_index.0)));
                word_acc = String::new();
                highlighted_lines.push(Spans::from(line_acc.clone()));
                line_acc = Vec::new();
            }
        }

        let t = Text::from(highlighted_lines);

        t
    }
}
