use crate::{terminal::{self, Terminal}, document::{Document, self}};
use termion::event::Key;
use crate::size::Size;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

enum EditorState {
    EDITING,
}

struct StatusMessage {
    text: String
}

pub struct Editor {
    should_quit: bool,
    curr_pos: Position, // Position in File
    document: Document,
    offset: Position,
    size: Size,
    state: EditorState,
    status_message: StatusMessage,
}

impl StatusMessage {
    fn from(str: String) -> Self {
        Self {
            text: str,
        }
    }
}

impl Editor {
    pub fn default(size: Size) -> Self {
        let args: Vec<String> = env::args().collect();
        let doc = if args.len() > 1 {
            Document::open(&args[1]).unwrap()
        }else {
            Document::default()
        };
        let size: Size = Size { width: size.width, height: size.height-2};
        Self {
            size,
            should_quit: false,
            curr_pos: Position::default(),
            offset: Position::default(),
            document: doc,
            state: EditorState::EDITING,
            status_message: StatusMessage::from(String::from("Ctrl-Q to quit!")),
        }
    }

    pub fn run(&mut self) {
        terminal::Terminal::clear_screen();
        loop {
            if self.should_quit {
                terminal::Terminal::clear_screen();
                terminal::Terminal::cursor_position(&Position::default());
                println!("Goodbye!\r");
                break;
            }

            if let Err(error) = self.refresh_screen() {
                die(error);
            }

            if let Err(error) = self.process_keyboard() {
                die(error);
            }
            
        }
    }

    fn promt(&mut self, promt: &str) -> Result<String, std::io::Error> {
        let mut res = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{}{}", promt, res));
            self.refresh_screen()?;
            let k = Terminal::read_key()?;
            if k == Key::Backspace  {
                if res.len() > 0 {
                    res.pop();
                }
            }else if let Key::Char(c) = k {
                if c == '\n' {
                    break;
                }
                if !c.is_control() {
                    res.push(c);
                }
            }
        }
        Ok(res)
    }

    fn process_keyboard(&mut self) -> Result<(), std::io::Error> {
        match self.state {
            EditorState::EDITING => {
                let key = terminal::Terminal::read_key()?;
                match key {
                    Key::Ctrl('q') => self.should_quit = true,
                    Key::Char(c) => {
                        if c == '\n' {
                            self.offset.x = 0;
                        }
                        self.document.insert(c, &self.curr_pos);

                        let r_size = if c == '\t' {
                            document::TAB_SIZE
                        }else {
                            1
                        };
                        for _ in 0..r_size {
                            self.move_cursor(Key::Right);
                        }
                    },
                    Key::Ctrl('s') => {
                        if self.document.file_name.is_none() {
                            self.document.file_name = Some(self.promt("Save As: ")?);
                            self.status_message = StatusMessage::from("".to_string());
                            if self.document.file_name.as_ref().unwrap().len() == 0 {
                                return Ok(());
                            }
                        }
                        if self.document.save().is_ok() {

                        }else {
                            panic!("Cannot save file!");
                        }
                    },
                    Key::Backspace => {
                        if self.curr_pos.x > 0 || self.curr_pos.y > 0 {
                            self.move_cursor(Key::Left);
                            self.document.delete(&self.curr_pos);
                        }
                    },
                    Key::Left | Key::Right | Key::Up | Key::Down => self.move_cursor(key),
                    _ => (),
                }
            },
        }
        Ok(())
    }

    pub fn move_cursor(&mut self, key: Key) {
        // TODO: Make horizontal
        let Position{mut x, mut y} = self.curr_pos;
        match key {
            Key::Right => {
                if let Some(row) = self.document.row(y) {
                    if x < row.len() {
                        x += 1;
                        if x - self.offset.x >= self.size.width as usize {
                            //self.offset.x += 1;
                        }
                    }else {
                        y = if let Some(_) = self.document.row(y+1) {
                            x = 0;
                            self.offset.x = 0;
                            y+1
                        }else {
                            y
                        };
                    }
                }else {
                    x = 0;
                }
            },
            Key::Left => {
                if x == 0 && y > 0 {
                    y = y.saturating_sub(1);
                    x = if let Some(row) = self.document.row(y) {
                        row.len() + 1
                    } else {
                        0
                    };

                    if x + self.offset.x > self.size.width as usize {
                        self.offset.x = x.saturating_sub(self.size.width as usize);
                    }

                }else {
                    x = x.saturating_sub(1);

                    if (x as i64 - self.offset.x as i64) < 0{
                        self.offset.x = self.offset.x.saturating_sub(1);
                    }
                }

            },
            Key::Up => {
                if (y as i64 - self.offset.y as i64) <= 0 {
                    self.offset.y = self.offset.y.saturating_sub(1);
                }
                if y > 0 {
                    y -= 1;
                    /*let row: &crate::document::Row = self.document.row(y).unwrap();
                    if x.saturating_sub(self.offset.x) > self.size.width as usize {
                        x = std::cmp::min(row.len(), x);
                        self.offset.x = x.saturating_sub(self.size.width as usize - 1);
                        self.status_message = StatusMessage::from(format!("{}", self.offset.x));
                    }*/
                }

            },
            Key::Down => {
                if y < self.document.len().saturating_sub(1) {
                    y += 1;
                }
                if y - self.offset.y >= self.size.height as usize {
                    
                    self.offset.y += 1;
                }
            },
            _ => ()
        }

        if let Some(row) = self.document.row(y) {
            x = std::cmp::min(row.len(), x);
        }
        self.curr_pos = Position{x,y};
    }

    pub fn draw_status(&self) {
        terminal::Terminal::set_bg_color();
        terminal::Terminal::set_fg_color();
        let width = self.size.width as usize;
        let temp_name = &"[No Name]".to_string();
        let file_name = if let Some(file) = &self.document.file_name {
            file
        }else {
            temp_name
        };
        let message = format!("File: '{}' line: {}/{}", file_name, self.curr_pos.y+1, self.document.len());
        let spaces = " ".repeat(width.saturating_sub(message.len()));
        println!("{}{}\r", message, spaces);
        terminal::Terminal::reset_bg_color();
        terminal::Terminal::reset_fg_color();
    }

    pub fn draw_sub(&self) {
        terminal::Terminal::clear_curr_line();
        terminal::Terminal::set_bg_color();
        terminal::Terminal::set_fg_color();

        let mut text = self.status_message.text.clone();
        text.push_str(" ".repeat((self.size.width as usize) - text.len()).as_str());
        print!("{}\r", text);

        terminal::Terminal::reset_bg_color();
        terminal::Terminal::reset_fg_color();
    }

    pub fn refresh_screen(&self) -> Result<(), std::io::Error> {
        terminal::Terminal::hide_cursor();
        terminal::Terminal::cursor_position(&Position::default());
        self.draw_status();
        self.draw_rows();
        self.draw_sub();
        terminal::Terminal::cursor_position(&Position { x: self.curr_pos.x.saturating_sub(self.offset.x), y: self.curr_pos.y - self.offset.y+1 });        

        terminal::Terminal::show_cursor();
        terminal::Terminal::flush()
    }

    pub fn draw_rows(&self) {
        for i in 0..self.size.height {
            terminal::Terminal::clear_curr_line();
            if let Some(row) = self.document.row(i as usize + self.offset.y) {
                let start = self.offset.x;
                let end = self.offset.x + self.size.width as usize;
                let draw_row = row.render(start, end, &self.document.markup);

                println!("{}\r", draw_row);
            }else if i == self.size.height / 2 && self.document.is_empty() {
                self.print_welcome_message();
            }else {
                println!("~\r");
            }
        }
    }

    pub fn print_welcome_message(&self) {
        let mut message = format!("Welcome to LATE - Version {}", VERSION);
        let width = self.size.width as usize;
        let spaces = " ".repeat(width.saturating_sub(message.len()) / 2);
        message = format!("~{}{}", spaces, message);
        message.truncate(std::cmp::min(self.size.width as usize, message.len()));
        println!("{}\r", message);
    }
}

fn die(e: std::io::Error) {
    terminal::Terminal::clear_screen();
    panic!("{}\r", e);
}
