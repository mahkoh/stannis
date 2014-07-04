use colors::*;
use utfbuf::{UtfBuf};
use nc = ncurses;
use term::cwidth::{CharWidth, StringWidth};

pub struct Prompt {
    text: String,
    prefix: String,
    prefix_width: uint,
    left: uint,
    right: uint,
    visible_width: uint,
    cursor: uint,
    cursor_term: uint,
    utfbuf: UtfBuf,
}

impl Prompt {
    pub fn new() -> Prompt {
        Prompt {
            text: String::new(),
            prefix: String::new(),
            prefix_width: 0,
            left: 0,
            right: 0,
            visible_width: 0,
            cursor: 0,
            cursor_term: 0,
            utfbuf: UtfBuf::new(),
        }
    }

    pub fn text<'a>(&'a self) -> &'a str {
        self.text.as_slice()
    }

    pub fn set_prefix(&mut self, prefix: &str) {
        self.prefix = prefix.to_string();
        let old_width = self.prefix_width;
        self.prefix_width = prefix.width();
        self.cursor_term += self.prefix_width - old_width;
        if old_width > self.prefix_width {
            let mut from_right = self.text.as_slice().slice_from(self.right).chars();
            while self.prefix_width + self.visible_width < nc::COLS as uint {
                match from_right.next() {
                    Some(c) => {
                        if self.prefix_width + self.visible_width + c.width2() <= nc::COLS as uint {
                            self.right += c.len_utf8_bytes();
                            self.visible_width += c.width2();
                        } else {
                            break;
                        }
                    },
                    None => break,
                }
            }
        } else if old_width < self.prefix_width {
            self.trim_right();
        }
    }

    /// Trim the visible text right so that it fits into the window.
    ///
    /// Trims at most 
    fn trim_right(&mut self) {
        let slice = self.text.as_slice();
        while self.prefix_width + self.visible_width > nc::COLS as uint {
            self.right = slice.prev_char(self.right);
            let c = self.text.as_slice().char_at(self.right);
            if self.right <= self.cursor {
                // undo this mistake
                self.right += c.len_utf8_bytes();
                return;
            }
            self.visible_width -= c.width2();
        }
    }

    // #[no_mangle]
    fn extend_right(&mut self) {
        let slice = self.text.as_slice();
        while self.prefix_width + self.visible_width < nc::COLS as uint {
            if self.right == self.text.len() {
                return;
            }
            let c = slice.char_at(self.right);
            if self.prefix_width + self.visible_width + c.width2() > nc::COLS as uint {
                return;
            }
            self.visible_width += c.width2();
            self.right += c.len_utf8_bytes();
        }
    }

    pub fn draw(&self, row: i32) {
        normal!(COLOR_PAIR_PROMPT);
        nc::move(row, 0);
        nc::addstr(self.prefix.as_slice());
        nc::addstr(self.text.as_slice().slice(self.left, self.right));
        nc::clrtoeol();
        nc::move(row, self.cursor_term as i32);
    }

    fn high_key(&mut self, key: i32) {
        match key {
            nc::KEY_BACKSPACE => self.del(),
            nc::KEY_LEFT => self.left(),
            nc::KEY_RIGHT => self.right(),
            _ => { }
        };
    }

    fn show_left(&mut self) {
        if self.prefix_width > nc::COLS as uint {
            self.left = self.cursor;
            self.right = self.cursor;
            self.visible_width = 0;
            self.cursor_term = nc::COLS as uint - 1;
            return;
        }
        let dest = (nc::COLS - self.prefix_width as i32)/4;
        let slice = self.text.as_slice();
        while self.cursor_term < dest as uint + self.prefix_width {
            match self.left {
                0 => break,
                _ => self.left = slice.prev_char(self.left),
            }
            let c = slice.char_at(self.left);
            self.cursor_term += c.width2();
            self.visible_width += c.width2();
        }
    }

    fn left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor = self.text.as_slice().prev_char(self.cursor);
        let c = self.text.as_slice().char_at(self.cursor);
        if self.cursor < self.left {
            self.left = self.cursor;
            self.visible_width += c.width2();
            self.show_left();
            self.trim_right();
        } else {
            self.cursor_term -= c.width2();
        }
    }

    fn right_word(&mut self) {
        if self.cursor == self.text.len() {
            return;
        }
        let next = self.text.as_slice().next_word(self.cursor);
        self.cursor_term += self.text.as_slice().slice(self.cursor, next).width();
        self.cursor = next;
        if next >= self.right {
            if self.right == self.text.len() && self.cursor_term < nc::COLS as uint {
                return;
            }
            self.visible_width += self.text.as_slice().slice(self.right, next).width();
            self.right = next;
            if next < self.text.len() {
                let c = self.text.as_slice().char_at(next);
                self.visible_width += c.width2();
                self.right += c.len_utf8_bytes();
            }
            self.show_right();
        }
    }

    fn left_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut prev = self.text.as_slice().prev_word(self.cursor);
        if prev != 0 {
            let bytes = self.text.as_slice().as_bytes();
            while bytes[prev] == ' ' as u8 {
                prev += 1;
            }
        }
        if prev >= self.left {
            self.cursor_term -= self.text.as_slice().slice(prev, self.cursor).width();
            self.cursor = prev;
        } else {
            self.visible_width += self.text.as_slice().slice(prev, self.left).width();
            self.left = prev;
            self.cursor = prev;
            self.cursor_term = self.prefix_width;
            self.show_left();
            self.trim_right();
        }
    }

    fn right(&mut self) {
        if self.cursor == self.text.len() {
            return;
        }
        let c = self.text.as_slice().char_at(self.cursor);
        self.cursor += c.len_utf8_bytes();
        self.cursor_term += c.width2();
        let w = if self.cursor == self.right {
            1
        } else {
            self.text.as_slice().char_at(self.cursor).width2()
        };
        if self.cursor_term + w > nc::COLS as uint {
            self.show_right();
        }
    }

    fn show_right(&mut self) {
        let visible = self.text.as_slice().slice(self.left, self.cursor);
        let mut from_right = self.text.as_slice().slice_from(self.right).chars().peekable();
        let dest = 3*(nc::COLS - self.prefix_width as i32)/4;
        if dest < 0 {
            self.left = self.cursor;
            self.right = self.cursor;
            self.visible_width = 0;
            self.cursor_term = self.prefix_width;
            return;
        }
        for c in visible.chars() {
            self.left += c.len_utf8_bytes();
            self.visible_width -= c.width2();
            self.cursor_term -= c.width2();
            match from_right.peek() {
                Some(&c) => {
                    if self.visible_width+self.prefix_width+c.width2() <= nc::COLS as uint {
                        from_right.next();
                        self.right += c.len_utf8_bytes();
                        self.visible_width += c.len_utf8_bytes();
                    }
                },
                None => { },
            }
            if self.cursor_term <= dest as uint + self.prefix_width {
                break;
            }
        }
    }

    fn del(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let len = {
            let prev = self.text.as_slice().prev_char(self.cursor);
            let len = self.cursor - prev;
            self.cursor = prev;
            len
        };
        let c = self.text.as_slice().char_at(self.cursor);
        self.right -= len;
        {
            let vec = unsafe { self.text.as_mut_vec() };
            for _ in range(0, len) {
                vec.remove(self.cursor);
            }
        }
        if self.cursor < self.left {
            self.left = self.cursor;
            self.show_left();
            self.trim_right();
        } else {
            self.cursor_term -= c.width2();
            self.visible_width -= c.width2();
            self.extend_right();
        }
    }

    fn del_forward(&mut self) {
        if self.cursor == self.text.len() {
            return;
        }
        let c = self.text.as_slice().char_at(self.cursor);
        let len = {
            let vec = unsafe { self.text.as_mut_vec() };
            let len = c.len_utf8_bytes();
            for _ in range(0, len) {
                vec.remove(self.cursor);
            }
            len
        };
        self.right -= len;
        self.visible_width -= c.width2();
        if self.right == self.left {
            self.show_left();
        } else {
            self.extend_right();
        }
    }

    fn del_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.text.as_slice().prev_word(self.cursor);
        let width = self.text.as_slice().slice(prev, self.cursor).width();
        {
            let vec = unsafe { self.text.as_mut_vec() };
            for _ in range(prev, self.cursor) {
                vec.remove(prev);
            }
        }
        self.right -= self.cursor - prev;
        if prev >= self.left {
            self.cursor = prev;
            self.cursor_term -= width;
            self.visible_width -= width;
        } else {
            self.left = prev;
            self.cursor = prev;
            self.visible_width -= self.cursor_term - self.prefix_width;
            self.cursor_term = self.prefix_width;
            self.show_left();
        }
        self.extend_right();
    }

    pub fn clear(&mut self) {
        self.left = 0;
        self.cursor = 0;
        self.cursor_term = self.prefix_width;
        self.right = 0;
        self.visible_width = 0;
        self.text.truncate(0);
    }

    fn control_key(&mut self, key: u32) {
        // Keys above 0x20 are unused
        match key {
            0x02 => /* c-b */ self.left(),
            // TODO(mahkoh) replace by c-c later
            0x04 => /* c-d */ self.clear(),
            0x06 => /* c-f */ self.right(),
            0x0E => /* c-n */ self.right_word(),
            0x10 => /* c-p */ self.left_word(),
            0x17 => /* c-w */ self.del_word(),
            0x18 => /* c-x */ self.del_forward(),
            _ => { }
        }
    }

    fn low_key(&mut self, key: u8) {
        let key = match self.utfbuf.push(key) {
            Some(key) => key,
            None => return,
        };
        let val = key as u32;
        if val < 0x20 || (val >= 0x7F && val < 0xA0) {
            self.control_key(val);
            return;
        }
        let width = match key.width2() {
            0 => return,
            n => n,
        };
        let len = {
            let mut buf = [0u8, ..4];
            let len = key.encode_utf8(buf.as_mut_slice());
            let vec = unsafe { self.text.as_mut_vec() };
            for i in range(0, len) {
                vec.insert(self.cursor + i, buf[i]);
            }
            len
        };
        self.cursor += len;
        self.right += len;
        self.cursor_term += width;
        self.visible_width += width;
        if self.cursor_term >= nc::COLS as uint {
            self.show_right();
        }
        self.trim_right();
    }

    pub fn key(&mut self, key: i32) {
        if key as u32 > 0xFF {
            self.high_key(key);
        } else {
            self.low_key(key as u8);
        }
    }
}

trait Movement<T, U> {
    fn prev_char(&self, pos: T) -> U;
    fn prev_word(&self, pos: T) -> U;
    fn next_word(&self, pos: T) -> U;
}

impl<'a> Movement<uint, uint> for &'a str {
    fn prev_char(&self, mut pos: uint) -> uint {
        if pos == 0 {
            return 0;
        }
        pos -= 1;
        let bytes = self.as_bytes();
        while bytes[pos] >= 0b1000_0000 && bytes[pos] < 0b1100_0000 {
            pos -= 1;
        }
        pos
    }

    fn prev_word(&self, mut pos: uint) -> uint {
        if pos == 0 {
            return 0;
        }
        pos -= 1;
        let bytes = self.as_bytes();
        while pos > 0 && bytes[pos]     == ' ' as u8 { pos -= 1; }
        while pos > 0 && bytes[pos]     != ' ' as u8 { pos -= 1; }
        while pos > 0 && bytes[pos - 1] == ' ' as u8 { pos -= 1; }
        pos + (pos != 0) as uint
    }

    fn next_word(&self, mut pos: uint) -> uint {
        let bytes = self.as_bytes();
        while pos < self.len() && bytes[pos] != ' ' as u8 { pos += 1; }
        while pos < self.len() && bytes[pos] == ' ' as u8 { pos += 1; }
        pos
    }
}
