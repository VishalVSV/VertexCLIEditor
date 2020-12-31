use crate::editor::*;

use crossterm::{event::{read,EnableMouseCapture,DisableMouseCapture},execute,terminal::{enable_raw_mode,disable_raw_mode}};
use std::io::{Write,stdout};
use crossterm::style::Color;

use std::error::Error;

pub struct TermWindow<T>
    where T: Drawable 
{
    pub editor: Editor,
    screen: T
}

pub trait Drawable {
    fn put_char(&mut self,x: usize,y: usize,c: char);
    fn put_string(&mut self,x: usize,y: usize,string: &str) {
        let mut i = 0;
        for c in string.chars() {
            self.put_char(x + i, y, c);
            i += 1;
        }
    }
    fn resize(&mut self,width: usize,height: usize);
    fn draw(&self) -> Result<(),Box<dyn Error>>;
    fn clear(&mut self,c: char);
    fn new(width: usize,height: usize) -> Self;
    fn color(&mut self, start: usize,end: usize, color: Color);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn to_string(&self) -> String;
}

pub mod backends {
    use crate::windowing::Drawable;
    use std::io::{Write,stdout};
    use crossterm::style::Color;
    use std::error::Error;

    pub struct ColoringHint {
        pub start: usize,
        pub end: usize,
        pub color: Color
    }

    pub struct CharGrid {
        buffer: Vec<char>,
        width: usize,
        height: usize,
        hints: Vec<ColoringHint>
    }

    impl Drawable for CharGrid {
        fn put_char(&mut self,x: usize, y: usize, c: char) {
            if x < self.width && y < self.height {
                self.buffer[x + y * self.width] = c;
            }
        }

        fn resize(&mut self,width: usize,height: usize) {
            self.width = width;
            self.height = height;
            self.buffer = Vec::with_capacity(width * height);
            for _ in 0..(width * height) {
                self.buffer.push(' ');
            }
        }

        fn new(width: usize,height: usize) -> Self {
            let mut buffer = Vec::with_capacity(width * height);

            for _ in 0..(width * height) {
                buffer.push(' ');
            }

            CharGrid {
                buffer,
                width,
                height,
                hints: Vec::new()
            }
        }

        fn clear(&mut self,c: char) {
            self.hints.clear();
            for i in 0..(self.width * self.height) {
                self.buffer[i] = c;
            }
        }

        fn color(&mut self,start: usize,end: usize, color: Color) {
            self.hints.push(ColoringHint {
                start,end,color
            });
        }

        fn draw(&self) -> Result<(),Box<dyn Error>> {
            print!("{}",crossterm::cursor::Hide);
            stdout().flush().unwrap();
            print!("{}",crossterm::cursor::MoveTo(0,0));

            let mut i = 0;
            for c in &self.buffer {
                for hint in &self.hints {
                    if i >= hint.start && i < hint.end {
                        crossterm::execute!(stdout(),crossterm::style::SetForegroundColor(hint.color))?;
                    }
                    if i != 0 && i - 1 >= hint.start && i - 1 < hint.end && !(i >= hint.start && i < hint.end) {
                        crossterm::execute!(stdout(),crossterm::style::SetForegroundColor(Color::White))?;
                    }
                }
                print!("{}",c);
                i += 1;
            }

            print!("{}",crossterm::cursor::MoveTo(0,0));
            stdout().flush().unwrap();
            print!("{}",crossterm::cursor::Show);
            stdout().flush().unwrap();

            Ok(())
        }

        fn width(&self) -> usize {
            self.width
        }
        
        fn height(&self) -> usize {
            self.height
        }

        fn to_string(&self) -> String {
            let mut string = String::with_capacity(self.buffer.len());

            for c in &self.buffer {
                string.push(*c);
            }

            string
        }
    }
}

pub enum UpdateResult {
    Draw,
    NOp,
    Exit
}

impl<T> TermWindow<T> where T: Drawable {
    pub fn new() -> Self {
        let (w,h) = crossterm::terminal::size().unwrap();
        Self {
            editor: Editor::new(),
            screen: T::new(w.into(),h.into())
        }
    }

    pub fn start(&mut self) -> Result<(),Box<dyn Error>> {
        enable_raw_mode()?;

        if self.editor.open_docs.len() == 0 {
            self.editor.make_new_doc("new 1".to_string());
        }

        self.editor.currently_open_doc = Some(0);
        
        let mut stdout = stdout();
        execute!(stdout, EnableMouseCapture)?;

        execute!(stdout,crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        self.screen.clear(' ');
        self.editor.draw(&mut self.screen);
        self.screen.draw()?;
        self.editor.update_cursor();
        loop {
            match self.editor.update(&mut self.screen, read().unwrap()) {
                UpdateResult::Draw => {
                    self.screen.clear(' ');
                    self.editor.draw(&mut self.screen);
                    self.screen.draw()?;
                    self.editor.update_cursor();
                },
                UpdateResult::NOp => {},
                UpdateResult::Exit => break
            }
        }

        execute!(stdout,crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        execute!(stdout,crossterm::cursor::MoveTo(0,0))?;
        execute!(stdout, DisableMouseCapture)?;
        disable_raw_mode()?;

        Ok(())
    }
}