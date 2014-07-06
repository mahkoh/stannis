use tox::core::{Address};
use std;
use std::num::{FromPrimitive};

#[repr(uint)]
#[deriving(FromPrimitive)]
pub enum _Command {
    _Quit,
    _Add,
    _Del,
}

impl _Command {
    fn first() -> _Command {
        FromPrimitive::from_uint(0).unwrap()
    }

    fn next(self) -> Option<_Command> {
        FromPrimitive::from_uint(self as uint + 1)
    }

    fn text(self) -> &'static str {
        match self {
            _Quit => "q",
            _Add => "add",
            _Del => "del",
        }
    }

    fn parse(self, iter: TokenIter) -> Result {
        match self {
            _Quit => Ok(Quit),
            _Add => self.parse_add(iter),
            _Del => self.parse_del(iter),
        }
    }

    fn parse_add(self, mut iter: TokenIter) -> Result {
        let address = match iter.next() {
            Some(s) => match from_str(s) {
                Some(addr) => addr,
                _ => return Err("invalid address"),
            },
            None => return Err("missing address"),
        };
        let msg = match iter.next() {
            Some(s) if s.len() > 0 => s.to_string(),
            Some(_) => return Err("message musn't be empty"),
            _ => return Err("missing message"),
        };
        if iter.next().is_some() {
            return Err("too many arguments");
        }
        Ok(Add(address, msg))
    }

    fn parse_del(self, mut iter: TokenIter) -> Result {
        let s = match iter.next() {
            Some(s) => s,
            None => return Err("missing id"),
        };
        let id = match from_str(s) {
            Some(i) => i,
            None => return Err("invalid id"),
        };
        if iter.next().is_some() {
            return Err("too many arguments");
        }
        Ok(Del(id))
    }
}

pub type Result = std::result::Result<Command, &'static str>;

pub enum Command {
    Quit,
    Add(Address, String),
    Del(i32),
}

pub fn parse(s: &str) -> Result {
    let mut iter = s.tokens();
    let first = match iter.next() {
        Some(s) => s,
        None => return Err("empty command"),
    };
    let mut command = _Command::first();
    loop {
        if first == command.text() {
            return command.parse(iter);
        }
        command = match command.next() {
            Some(c) => c,
            None => break,
        };
    }
    Err("unknown command")
}

trait Tokens {
    fn tokens<'a>(&'a self) -> TokenIter<'a>;
}

impl<'a> Tokens for &'a str {
    fn tokens<'a>(&'a self) -> TokenIter<'a> {
        TokenIter {
            st: *self,
            cur: 0,
        }
    }
}

struct TokenIter<'a> {
    st: &'a str,
    cur: uint,
}

impl<'a> Iterator<&'a str> for TokenIter<'a> {
    fn next(&mut self) -> Option<&'a str> {
        if self.cur == self.st.len() {
            return None;
        }
        let bytes = self.st.as_bytes();
        while self.cur < self.st.len() && bytes[self.cur] == ' ' as u8 {
            self.cur += 1
        };
        let start = self.cur;
        let mut quoted = false;
        let mut escaped = false;
        while self.cur < self.st.len() {
            if !escaped && bytes[self.cur] == '"' as u8 {
                quoted = !quoted;
            }
            escaped = !escaped && bytes[self.cur] == '\\' as u8;
            if !quoted && bytes[self.cur] == ' ' as u8 {
                break;
            }
            self.cur += 1
        };
        Some(self.st.slice(start, self.cur))
    }
}
