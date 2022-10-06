use std::io::{self, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use crate::editor::Position;
use crate::size::Size;

pub struct Terminal {
    size: Size,
    _stdio: RawTerminal<std::io::Stdout>
}

impl Terminal {

    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1,
            },
            _stdio: stdout().into_raw_mode()?
        })
    }

    pub fn clear_curr_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    pub fn set_bg_color() {
        print!("{}", termion::color::Bg(termion::color::White));
    }

    pub fn reset_bg_color() {
        print!("{}", termion::color::Bg(termion::color::Reset));
    }

    pub fn set_fg_color() {
        print!("{}", termion::color::Fg(termion::color::Red));
    }

    pub fn reset_fg_color() {
        print!("{}", termion::color::Fg(termion::color::Reset));
    }

    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    pub fn show_cursor() {
        print!("{}", termion::cursor::Show);
    }

    pub fn hide_cursor() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn cursor_position(position: &Position) {
        print!("{}", termion::cursor::Goto(position.x.saturating_add(1) as u16, position.y.saturating_add(1) as u16));
    }

    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }
}
