use std::io::{Write,stdout};

use crate::windowing::Drawable;
use crossterm::style::Color;

use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

use crate::lexer::SyntaxHighlighter;
use crate::windowing::UpdateResult;

pub struct Editor {
    pub open_docs: Vec<Document>,
    pub currently_open_doc: Option<usize>,
    pub highlighter: SyntaxHighlighter,
    pub start_line: usize,
    pub tab_str: String,
    pub deviation: usize
}

impl Editor {
    pub fn new() -> Self {
        Self {
            open_docs: Vec::new(),
            currently_open_doc: None,
            start_line:0,
            tab_str: "    ".to_string(),
            deviation: 0,
            highlighter: SyntaxHighlighter::new(Editor::get_config())
        }
    }
	
	fn get_config() -> String {
		let mut dir = std::env::current_exe().expect("Couldn't get exe location!");
		dir.pop();
		dir.push("Config");
		dir.push("syntax.txt");
		std::fs::read_to_string(dir.to_str().expect("Couldn't get syntax location!")).expect("Syntax config not found!")
	}

    pub fn open(&mut self,path: String) {
        let (dev,doc) = Document::from_file(path);
        self.open_docs.push(doc);
        self.deviation += dev;
    }

    pub fn draw<T>(&mut self,window: &mut T)
    where T: Drawable {
        self.highlighter.reset();
        let start_x = 4;
        let start_y = 1;

        let mut x = start_x;
        let mut y = start_y;

        let mut line = 1;
        window.put_string(0, start_y, &format!("{:0>3}", line));

        for cell in &self.open_docs[self.currently_open_doc.unwrap()].cells {
            match cell {
                Cell::Char(c) => {
                    if line - 1 >= self.start_line && line - 1 < self.start_line + window.height() && y - self.start_line > 0 {
                        window.put_char(x, y - self.start_line, *c);
                    }
                    x += 1;
                },
                Cell::NewLine => {
                    y += 1;
                    x = start_x;
                    line += 1;
                    if line - 1 >= self.start_line && line - 1 < self.start_line + window.height() && y - self.start_line > 0 {
                        window.put_string(0, y- self.start_line, &format!("{:0>3}", line));
                    }
                }
            }

        }
        
            
        let mut off_x = 4;
        let mut doc_num = 0;
        for doc in &self.open_docs {
            window.color(off_x, off_x + doc.name.len(), 
                if doc_num == self.currently_open_doc.unwrap() {
                    Color::Red
                }
                else {
                    Color::White
                }
            );
            window.put_string(off_x, 0, &doc.name);
            off_x += doc.name.len() + 2;
            doc_num += 1;
        }

        if let Some(current_doc) = self.currently_open_doc {
            self.highlighter.highlight(window,self.deviation,&self.open_docs[current_doc].file_type);
        }
    }

    pub fn update<T>(&mut self,screen: &mut T,event: Event) -> UpdateResult
    where T: Drawable {
        if let Some(current_doc) = self.currently_open_doc {
            match event {
                Event::Key(x) =>{
                    match x.code {
                        KeyCode::Char(c) => {
                            if x.modifiers == KeyModifiers::CONTROL {
                                if c == 's' {
                                    self.open_docs[current_doc].save();
                                }

                                return UpdateResult::Draw;
                            }

                            if c == '(' {
                                self.open_docs[current_doc].insert(Cell::Char('('));
                                self.open_docs[current_doc].insert(Cell::Char(')'));
                                self.open_docs[current_doc].cursor_pos -= 1;
                            }
                            else if c == ')' {
                                if self.open_docs[current_doc].cursor_pos < self.open_docs[current_doc].cells.len() && self.open_docs[current_doc].cells[self.open_docs[current_doc].cursor_pos] == Cell::Char(')') {
                                    self.open_docs[current_doc].cursor_pos += 1;
                                }
                                else {
                                    self.open_docs[current_doc].insert(Cell::Char(')'));
                                }
                            }
                            else {
                                self.open_docs[current_doc].insert(Cell::Char(c));
                            }
                        },
                        KeyCode::Tab => {
                            for c in self.tab_str.chars() {
                                self.open_docs[current_doc].insert(Cell::Char(c));
                            }
                        },
                        KeyCode::Left => {
                            if x.modifiers == KeyModifiers::empty() {
                                self.open_docs[current_doc].move_cursor_left();

                            }
                            else {
                                if let Some(c_doc) = self.currently_open_doc {
                                    if c_doc != 0 {
                                        self.currently_open_doc = Some(c_doc - 1);
                                    }
                                }
                            }
                        },
                        KeyCode::Right => { 
                            if x.modifiers == KeyModifiers::empty() {
                                self.open_docs[current_doc].move_cursor_right(); 
                                
                            }
                            else {
                                if let Some(c_doc) = self.currently_open_doc {
                                    if c_doc + 1 < self.open_docs.len() {
                                        self.currently_open_doc = Some(c_doc + 1);
                                    }
                                }
                            }
                        },
                        KeyCode::Up => self.open_docs[current_doc].move_cursor_up(),
                        KeyCode::Down => self.open_docs[current_doc].move_cursor_down(),
                        KeyCode::Enter => self.open_docs[current_doc].insert(Cell::NewLine),
                        KeyCode::Backspace => {
                            if self.open_docs[current_doc].cursor_pos != 0 {
                                let pos = self.open_docs[current_doc].cursor_pos - 1;
                                self.open_docs[current_doc].delete(pos);
                                self.open_docs[current_doc].cursor_pos -= 1;
                            }
                            else {
                                return UpdateResult::NOp;
                            }
                        },
                        KeyCode::Delete => {
                            if self.open_docs[current_doc].cursor_pos < self.open_docs[current_doc].cells.len() {
                                let pos = self.open_docs[current_doc].cursor_pos;
                                self.open_docs[current_doc].delete(pos);
                            }
                            else {
                                return UpdateResult::NOp;
                            }
                        },
                        KeyCode::Esc => {
                            return UpdateResult::Exit;
                        }
                        _ => return UpdateResult::NOp
                    }
                },
                Event::Resize(_,_) => {
                    let (width,height) = crossterm::terminal::size().expect("Couldn't get resized size");
                    screen.resize(width.into(), height.into());
                }
                Event::Mouse(_) => {
                    
                }
            }

            if self.open_docs[current_doc].current_line() < self.start_line {
                self.start_line -= 1;
            }
            else if self.open_docs[current_doc].current_line() - self.start_line + 1 >= screen.height() {
                self.start_line += 1;
            }
        }

        UpdateResult::Draw
    }

    pub fn make_new_doc(&mut self,doc_name: String) {
        self.open_docs.push(Document::new(doc_name,"path".to_string()));
        if self.currently_open_doc.is_none() {
            self.currently_open_doc = Some(self.open_docs.len() - 1);
        }
    }

    pub fn update_cursor(&mut self) {
        let start_x = 4;
        let start_y = 1;

        let mut x = start_x;
        let mut y = start_y;

        let mut i = 0;

        for cell in &self.open_docs[self.currently_open_doc.unwrap()].cells {
            if i >= self.open_docs[self.currently_open_doc.unwrap()].cursor_pos {
                break;
            }
            match cell {
                Cell::NewLine => {
                    x = start_x;
                    y += 1;
                },
                Cell::Char(_) => x += 1
            }
            i += 1;
        }

        y -= self.start_line as u16;

        print!("{}",crossterm::cursor::MoveTo(x,y));
        stdout().flush().unwrap();
    }

    
}

#[derive(Debug,PartialEq)]
pub enum Cell {
    Char(char),
    NewLine,
}

pub struct Document {
    pub cells: Vec<Cell>,
    pub cursor_pos: usize,
    pub name: String,
    pub file_type: Option<String>,
    pub path: String
}

use std::collections::HashMap;


lazy_static! {
    static ref ICON_MAP: HashMap<&'static str,char> = [
        ("rs",'\u{E7A8}'),
        ("js",'\u{E781}'),
        ("html",'\u{E736}'),
        ("htm",'\u{E736}'),
        ("java",'\u{E738}'),
        ("cpp",'\u{FB71}'),
        ("cs",'\u{F81A}'),
        ("py",'\u{F81F}')
    ].iter().cloned().collect();
}

impl Document {
    pub fn new(name: String,path: String) -> Self {
        Self {
            cells: Vec::new(),
            cursor_pos: 0,
            name,
            file_type:None,
            path
        }
    }

    pub fn current_line(&self) -> usize {
        let mut line = 0;
        for i in 0..self.cursor_pos {
            if let Cell::NewLine = self.cells[i] {
                line += 1;
            }
        }
        line
    }

    pub fn from_file(path: String) -> (usize,Self) {
        let mut cells = Vec::new();

        let file = File::open(&path).expect("File not found");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                for c in line.chars() {
                    cells.push(Cell::Char(c));
                }
                cells.push(Cell::NewLine);
            }
        }

        cells.pop();

        let mut name = std::path::Path::new(&path).file_name().unwrap().to_str().unwrap().to_string();

        let mut deviation = 0;
        let mut file_type = None;

        if let Some(ext) = std::path::Path::new(&path).extension() {
            if ICON_MAP.contains_key(ext.to_str().unwrap()) {
                name.push(' ');
                name.push(ICON_MAP[ext.to_str().unwrap()]);
                file_type = Some(ext.to_str().unwrap().to_string());
                deviation = 2;
            }
        }

        (deviation,
        Self {
            cells,
            cursor_pos: 0,
            name,
            file_type,
            path
        })
    }

    pub fn save(&self) {
        if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&self.path) {
            file.set_len(0).unwrap();

            let mut content = String::new();
            for cell in &self.cells {
                match cell {
                    Cell::Char(c) => content.push(*c),
                    Cell::NewLine => content.push('\n')
                }
            }

            file.write(content.as_bytes()).expect(&format!("Couldn't access file path {}",&self.path));

            Document::error("Saved file!".to_string());
        }
        else {
            Document::error(format!("Couldn't open file {}",&self.path));
        }
    }

    fn error(err_msg: String) {
        print!("{}",err_msg);
        let _ = stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    pub fn insert(&mut self,cell: Cell) {
        self.cells.insert(self.cursor_pos, cell);
        self.cursor_pos += 1;
    }

    pub fn delete(&mut self,index: usize) {
        self.cells.remove(index);
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos + 1 <= self.cells.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos != 0 {
            self.cursor_pos -= 1;
        }
    }

    fn current_line_length(&mut self) -> usize {
        let original_cursor_pos = self.cursor_pos;

        self.move_to_last_line_end();

        let line_start = self.cursor_pos + 1;

        self.cursor_pos += 1;

        self.move_to_next_line_end();

        let line_length = self.cursor_pos - line_start;

        self.cursor_pos = original_cursor_pos;

        line_length
    }

    fn move_to_last_line_end(&mut self) {
        while self.cursor_pos != 0 {
            if self.cursor_pos >= self.cells.len() {
                break;
            }
            match &self.cells[self.cursor_pos] {
                Cell::NewLine => return,
                _ => {
                    self.cursor_pos -= 1;
                    continue;
                }
            }    
        }
    }

    pub fn move_cursor_up(&mut self) {
        let original_cursor_pos = self.cursor_pos;
        if self.cursor_pos >= self.cells.len() {
            if self.cursor_pos == 0 {
                return;
            }
            self.cursor_pos -= 1;
        }
        self.move_to_last_line_end();
        let diff = original_cursor_pos - self.cursor_pos;

        let line_cursor_pos = self.cursor_pos;
        
        if self.cursor_pos == 0 {
            return;
        }
        self.cursor_pos -= 1;
        self.move_to_last_line_end();

        if line_cursor_pos - self.cursor_pos >= diff {
            if diff == 0 {
                self.cursor_pos += diff;
            }
            else {
                self.cursor_pos += diff - 1;
            }
        }
        else {
            self.cursor_pos = line_cursor_pos;
        }
    }

    fn move_to_next_line_end(&mut self) {
        while self.cursor_pos + 1 < self.cells.len() {
            match &self.cells[self.cursor_pos] {
                Cell::NewLine => return,
                _ => {
                    self.cursor_pos += 1;
                    continue;
                }
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        let initial_pos = self.cursor_pos;
        self.move_to_last_line_end();
        let diff = initial_pos - self.cursor_pos;

        self.cursor_pos = initial_pos;

        self.move_to_next_line_end();

        self.cursor_pos += 1;

        let len = self.current_line_length();

        if diff < len {
            self.cursor_pos += diff;
        }
        else {
            self.cursor_pos += len;
        }

        if self.cursor_pos > self.cells.len() {
            self.cursor_pos = self.cells.len();
        }
    }
}