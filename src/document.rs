use std::io::Write;

use unicode_segmentation::UnicodeSegmentation;

use crate::editor::Position;
use crate::markup::Markup;

pub const TAB_SIZE: usize = 1;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            len: 0,
        };
        row.update_len();
        row
    }
}

impl Row {

    pub fn render(&self, start: usize, end: usize, markup: &Markup) -> String {
        let start = std::cmp::min(start, end);
        let end = std::cmp::min(end, self.string.len());
        let mut char_result = String::new();
        for c in self.string.graphemes(true).skip(start).take(end.saturating_sub(start)) {
            if c == "\t" {
                char_result.push_str(&" ".repeat(TAB_SIZE));
            }
            else {
                char_result.push_str(c);
            }
        }

        let mut result: String = String::new();
        let splitted = char_result.split(" ");

        for split in splitted {
            if markup.keywords.contains(&String::from(split)) {
                result.push_str(&format!("{}{}{}", termion::color::Fg(termion::color::Blue), split, termion::color::Fg(termion::color::Reset)));
            }else {
                result.push_str(&split);
            }
            result.push(' ');
        }

        result
    }

    pub fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, c: char, index: usize) {
        if index >= self.len() {
            self.string.push(c);
        }else {
            let mut result: String = self.string[..].graphemes(true).take(index).collect();
            let remainder: String = self.string[..].graphemes(true).skip(index).collect();
            result.push(c);
            result.push_str(&remainder);
            self.string = result;
        }

        self.update_len();
    }

    pub fn append(&mut self, new: &Self){
        self.string = format!("{}{}", self.string, new.string);
        self.update_len();
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn delete(&mut self, index: usize) {
        if index >= self.len() {
            return;
        }
        let mut result: String = self.string[..].graphemes(true).take(index).collect();
        let remainder: String = self.string[..].graphemes(true).skip(index + 1).collect();
        result.push_str(&remainder);
        self.string = result;

        self.update_len();
    }

    pub fn split(&mut self, index: usize) -> Self {
        let beginn: String = self.string[..].graphemes(true).take(index).collect();
        let remainder: String = self.string[..].graphemes(true).skip(index).collect();
        self.string = beginn;
        self.update_len();
        Self::from(&remainder[..])
    }

}

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    pub markup: Markup,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in content.lines() {
            rows.push(Row::from(value));
        }
        let mut splitted = filename.split(".");
        let file_ending = splitted.nth(splitted.to_owned().count() - 1).unwrap();
        let markup = Markup::load(&(String::from("markup/") + file_ending + ".txt"))?;
        Ok(Self {
            rows,
            markup: markup,
            file_name: Some(filename.to_string())
        })
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = std::fs::File::create(file_name)?;
            let mut index = 0;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                if index < self.rows.len() - 1 {
                    file.write(b"\n")?;
                }
                index += 1;
            }
        }
        Ok(())
    }

    pub fn insert(&mut self, c: char, pos: &Position) {
        if c == '\n' {
            if pos.y > self.len() {
                return;
            }
            if pos.y == self.len() {
                self.rows.push(Row::default());
                return;
            }

            let new_row = self.rows.get_mut(pos.y).unwrap().split(pos.x);
            self.rows.insert(pos.y + 1, new_row);
            return;
        }
        if pos.y == self.len() {
            let mut row = Row::default();
            row.insert(c, 0);
            self.rows.push(row);
        }else if pos.y < self.len() {
            let row = self.rows.get_mut(pos.y).unwrap();
            row.insert(c, pos.x);
        }
    }

    pub fn delete(&mut self, pos: &Position) {
        if pos.y >= self.len() {
            return;
        }
        if pos.x == self.rows.get(pos.y).unwrap().len() && pos.y < self.len() - 1 {
            let next_row = self.rows.remove(pos.y + 1);
            let row = self.rows.get_mut(pos.y).unwrap();
            row.append(&next_row);
        }else {
            self.rows.get_mut(pos.y).unwrap().delete(pos.x);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
}
