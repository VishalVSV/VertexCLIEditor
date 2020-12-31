mod editor;
mod windowing;
mod lexer;

#[macro_use]
extern crate lazy_static;

use crate::windowing::*;
use crate::windowing::backends::CharGrid;

fn main() {
    let mut window: TermWindow<CharGrid> = TermWindow::new();
    let mut iter = std::env::args();
    iter.next();
    for file in iter {
        window.editor.open(file);
    }
    let _ = window.start();
}