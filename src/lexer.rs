use regex::Regex;
use crossterm::style::Color;
use crate::Drawable;

use std::str::FromStr;

use std::collections::HashMap;

pub struct SyntaxHighlighter {
    buffer: String,
    syntax_coloring: HashMap<String,Vec<(Regex,Color)>>
}

impl SyntaxHighlighter {
    pub fn new(config: String) -> Self {

        let color_reg = regex::Regex::from_str(r#"color rgb\((?P<r>[0-9]+)( )*,(?P<g>[0-9]+)( )*,(?P<b>[0-9]+)( )*\) (?P<regex>.*$)"#).unwrap();

        let mut syntax_coloring = HashMap::new();

        let mut colors = Vec::new();
        for line in config.lines() {
            if line.starts_with("file") {
                if let Some(file_type) = line.strip_prefix("file ") {
                    syntax_coloring.insert(file_type.trim().to_string(), colors.clone());
                    colors.clear();
                }
            }

            if line.starts_with("color") {
                if let Some(cap) = color_reg.captures(&line) {
                    if let Some(r) = cap.name("r").map(|r| r.as_str().parse::<u8>().ok()).flatten() {
                        if let Some(g) = cap.name("g").map(|g| g.as_str().parse::<u8>().ok()).flatten() {
                            if let Some(b) = cap.name("b").map(|b| b.as_str().parse::<u8>().ok()).flatten() {
                                if let Some(regex) = cap.name("regex").map(|regex| regex::Regex::from_str(regex.as_str()).ok()).flatten() {
                                    colors.push((regex,Color::from((r,g,b))));
                                }
                                else {
                                    println!("{:?} err occured",regex::Regex::from_str(cap.name("regex").unwrap().as_str()).err());
                                    let _ = crossterm::event::read();
                                }
                            }
                        }
                    }
                }
            }
        }

        Self {
            buffer: String::new(),
            syntax_coloring
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }


    pub fn highlight<T>(&mut self,window: &mut T,deviation: usize,file_type: &Option<String>)
        where T: Drawable 
    {
        if let Some(file_type) = file_type {
            let window_string = window.to_string();

            if let Some(syntax_coloring) = self.syntax_coloring.get(file_type) {
                for (regex,color) in syntax_coloring {
                    for caps in regex.captures_iter(&window_string) {
                        if let Some(cap) = caps.name("color") {
                            let x = (cap.start() - deviation) % window.width();
                            let y = (cap.start() - deviation) / window.width();
                            if y != 0 && x >= 4 {
                                window.color(cap.start() - deviation, cap.end() - deviation, *color);
                            }
                        }                
                    }
                }
            }
        }
    }
}