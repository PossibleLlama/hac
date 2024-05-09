use std::ops::{Add, Sub};

use crate::text_object::cursor::Cursor;
use ropey::Rope;

#[derive(Debug, Clone, PartialEq)]
pub struct Readonly;
#[derive(Debug, Clone, PartialEq)]
pub struct Write;

#[derive(Debug, Clone, PartialEq)]
pub struct TextObject<State = Readonly> {
    content: Rope,
    state: std::marker::PhantomData<State>,
}

impl<State> Default for TextObject<State> {
    fn default() -> Self {
        let content = String::default();

        TextObject {
            content: Rope::from_str(&content),
            state: std::marker::PhantomData,
        }
    }
}

impl TextObject<Readonly> {
    pub fn from(content: &str) -> TextObject<Readonly> {
        let content = Rope::from_str(content);
        TextObject::<Readonly> {
            content,
            state: std::marker::PhantomData::<Readonly>,
        }
    }

    pub fn with_write(self) -> TextObject<Write> {
        TextObject::<Write> {
            content: self.content,
            state: std::marker::PhantomData,
        }
    }
}

impl TextObject<Write> {
    pub fn insert_char(&mut self, c: char, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content.insert_char(col_offset, c);
    }

    pub fn erase_backwards_up_to_line_start(&mut self, cursor: &Cursor) {
        if cursor.col().eq(&0) {
            return;
        }
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset.saturating_sub(1)..col_offset)
            .ok();
    }

    pub fn erase_previous_char(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset.saturating_sub(1)..col_offset)
            .ok();
    }

    pub fn erase_current_char(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        self.content.try_remove(col_offset..col_offset.add(1)).ok();
    }

    pub fn current_line(&self, cursor: &Cursor) -> Option<&str> {
        self.content.line(cursor.row()).as_str()
    }

    pub fn line_len(&self, line: usize) -> usize {
        let mut line_len = 0;
        if let Some(line) = self.content.line(line).as_str() {
            line_len = line.len();
            line.contains('\r')
                .then(|| line_len = line_len.saturating_sub(1));
            line.contains('\n')
                .then(|| line_len = line_len.saturating_sub(1));
        }
        line_len
    }

    pub fn erase_until_eol(&mut self, cursor: &Cursor) {
        let line = self.content.line_to_char(cursor.row());
        let next_line = self.content.line_to_char(cursor.row().add(1));
        let col_offset = line + cursor.col();
        self.content
            .try_remove(col_offset.saturating_sub(1)..next_line.saturating_sub(1))
            .ok();
    }

    pub fn find_char_after_whitespace(&self, cursor: &Cursor) -> (usize, usize) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        let mut walked = 0;
        let mut found = false;

        for char in self.content.chars_at(col_offset) {
            match (char, found) {
                (c, false) if c.is_whitespace() => {
                    found = true;
                    walked = walked.add(1);
                }
                (c, true) if !c.is_whitespace() => break,
                _ => walked = walked.add(1),
            }
        }
        let curr_idx = col_offset.add(walked);
        let curr_row = self.content.char_to_line(col_offset.add(walked));
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = curr_idx.sub(curr_row_start);
        (curr_col, curr_row)
    }

    pub fn find_char_before_whitespace(&self, cursor: &Cursor) -> (usize, usize) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        let mut found = false;
        let mut index = col_offset.saturating_sub(1);

        for _ in (0..col_offset.saturating_sub(1)).rev() {
            let char = self.content.char(index);
            match (char, found) {
                (c, false) if c.is_whitespace() => found = true,
                (c, true) if !c.is_whitespace() => break,
                _ => {}
            }
            index = index.saturating_sub(1);
        }

        let curr_row = self.content.char_to_line(index);
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = index - curr_row_start;

        (curr_col, curr_row)
    }

    pub fn find_char_after_separator(&self, cursor: &Cursor) -> (usize, usize) {
        let line = self.content.line_to_char(cursor.row());
        let col_offset = line + cursor.col();
        let mut walked = 0;
        let mut found = false;

        for char in self.content.chars_at(col_offset) {
            match (char, found) {
                (c, false) if !c.is_alphanumeric() => {
                    found = true;
                    walked = walked.add(1);
                }
                (c, true) if c.is_alphanumeric() => break,
                _ => walked = walked.add(1),
            }
        }

        let curr_idx = col_offset.add(walked);
        let curr_row = self.content.char_to_line(col_offset.add(walked));
        let curr_row_start = self.content.line_to_char(curr_row);
        let curr_col = curr_idx.sub(curr_row_start);
        (curr_col, curr_row)
    }

    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }
}

impl<State> std::fmt::Display for TextObject<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content.to_string())
    }
}

fn is_separator(c: char) -> bool {
    c.is_whitespace() || matches!(c, '.' | '/' | '\'' | '"')
}