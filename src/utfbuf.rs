use std::char::{from_u32};

pub struct UtfBuf {
    left: uint,
    val: u32,
}

impl UtfBuf {
    pub fn new() -> UtfBuf {
        UtfBuf {
            left: 0,
            val: 0,
        }
    }

    pub fn push(&mut self, c: u8) -> Option<char> {
        if c < 0b1000_0000 {
            self.left = 0;
            return Some(c as char)
        } else if c >= 0b1100_0000 {
            match c >> 4 {
                0b1100 | 0b1101 => {
                    self.left = 1;
                    self.val = (c & 0b0001_1111) as u32;
                },
                0b1110 => {
                    self.left = 2;
                    self.val = (c & 0b0000_1111) as u32;
                },
                0b1111 => {
                    self.left = 3;
                    self.val = (c & 0b0000_0111) as u32;
                },
                _ => {
                    self.left = 0;
                },
            }
        } else if self.left > 0 {
            self.left -= 1;
            self.val = (self.val << 6) | (c & 0b0011_1111) as u32;
            if self.left == 0 {
                return from_u32(self.val);
            }
        }
        None
    }
}
