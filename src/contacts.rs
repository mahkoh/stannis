use tox::core::{ClientId};
use nc = ncurses;
use colors::*;
use term::cwidth::{CharWidth, StringWidth};
use prompt::{Prompt};

struct FriendRequest {
    id: ClientId,
    message: String,
}

struct Friend {
    id: u32,
    name: String,
}

struct Group {
    id: u32,
}

enum Mode {
    CommandMode,
    SearchMode,
    InsertMode,
    NormalMode,
}

impl Mode {
    fn fmt(self) -> &'static str {
        match self {
            CommandMode => "[:] ",
            SearchMode => "[/] ",
            InsertMode => "[i] ",
            NormalMode => "[n] ",
        }
    }
}

pub struct View<'a> {
    requests: Vec<String>,
    friends: Vec<(String, String)>,
    groups: uint,
    top: uint,
    selected: Row<'a>,
    mode: Mode,
    prompt: Prompt,
}

#[deriving(Eq, PartialEq)]
enum Row<'a> {
    Header(&'a str),
    RequestRow(uint),
    GroupRow(uint),
    FriendRow(uint),
}

impl<'a> View<'a> {
    fn absolute(&self, row: Row) -> uint {
        match (row, self.requests.len(), self.groups) {
            (RequestRow(i), _, _) => 1+i,
            (GroupRow(i), r, _) => 1+i+r+((r != 0) as uint),
            (FriendRow(i), r, g) => 1+i+r+g+((r != 0) as uint)+((g != 0) as uint),
            _ => 0,
        }
    }

    pub fn new() -> View {
        let mut prompt = Prompt::new();
        prompt.set_prefix("[n] ");
        View {
            requests: vec!("number1".to_string(), "number2".to_string()),
            friends: vec!(("mahkoh".to_string(), "shitposting".to_string()),
                          ("astonex".to_string(), "ayy lmao".to_string())),
            groups: 30,
            top: 0,
            selected: RequestRow(0),
            mode: NormalMode,
            prompt: prompt,
        }
    }

    pub fn up(&mut self) {
        if self.top == 1 && self.absolute(self.selected) == 1 {
            self.top = 0;
            return;
        }
        match self.selected {
            RequestRow(i) => {
                if i > 0 {
                    self.selected = RequestRow(i-1);
                }
            },
            GroupRow(i) => {
                if i > 0 {
                    self.selected = GroupRow(i-1);
                } else if self.requests.len() > 0 {
                    self.selected = RequestRow(self.requests.len() - 1);
                }
            },
            FriendRow(i) => {
                if i > 0 {
                    self.selected = FriendRow(i-1);
                } else if self.groups > 0 {
                    self.selected = GroupRow(self.groups - 1);
                } else if self.requests.len() > 0 {
                    self.selected = RequestRow(self.requests.len() - 1);
                }
            },
            _ => { },
        }
        if self.absolute(self.selected) < self.top {
            self.top = self.absolute(self.selected);
        }
    }

    pub fn down(&mut self) {
        match self.selected {
            RequestRow(i) => {
                if i < self.requests.len() - 1 {
                    self.selected = RequestRow(i+1);
                } else  if self.groups > 0 {
                    self.selected = GroupRow(0);
                } else if self.friends.len() > 0 {
                    self.selected = FriendRow(0);
                }
            },
            GroupRow(i) => {
                if i < self.groups - 1 {
                    self.selected = GroupRow(i+1);
                } else if self.friends.len() > 0 {
                    self.selected = FriendRow(0);
                }
            },
            FriendRow(i) => {
                if i < self.friends.len() - 1 {
                    self.selected = FriendRow(i+1);
                }
            },
            _ => { },
        }
        if self.absolute(self.selected) - self.top >= nc::LINES as uint - 3 {
            self.top = self.absolute(self.selected) - nc::LINES as uint + 3;
        }
    }

    pub fn update(&self) {
        for (i, row) in self.iter().skip(self.top).take(nc::LINES as uint - 2).enumerate() {
            match row {
                Header(s) => self.print_header(i as i32, s),
                _ => self.print_entry(i as i32, row),
            }
        }
        self.prompt.draw(nc::LINES - 2);
        nc::refresh();
        match self.mode {
            NormalMode => nc::curs_set(nc::CURSOR_INVISIBLE),
            _ => nc::curs_set(nc::CURSOR_VISIBLE),
        };
    }

    fn print_header(&self, y: i32, text: &str) {
        bold!(COLOR_PAIR_HEADER);
        nc::mvaddstr(y, 0, text);
        normal!(COLOR_PAIR_SEPARATOR);
        nc::addch(' ' as u32);
        for _ in range(text.len(), nc::COLS as uint - 1) {
            nc::addch(nc::ACS_HLINE());
        }
        normal!(COLOR_PAIR_DEFAULT);
    }

    fn print_entry(&self, y: i32, row: Row) {
        if self.selected == row {
            bold!(COLOR_PAIR_SELECTED);
        }
        nc::mvaddch(y, 0, ' ' as u32);
        match row {
            RequestRow(i) => {
                nc::addstr(self.requests.get(i as uint).as_slice());
            },
            GroupRow(i) => {
                nc::addstr(format!("Groupchat {}", i).as_slice());
            },
            FriendRow(i) => {
                let &(ref friend, ref status) = self.friends.get(i);
                nc::addstr(format!("{}   {}", friend, status).as_slice());
            }
            _ => { },
        }
        nc::clrtoeol();
        if self.selected == row {
            normal!(COLOR_PAIR_DEFAULT);
        }
    }

    fn iter<'b>(&'b self) -> RowIter<'b> {
        RowIter {
            view: self,
            row: RequestRow(0),
            header: true,
        }
    }

    pub fn handle_key(&mut self, key: i32) {
        match self.mode {
            NormalMode => self.handle_normal_mode_key(key),
            _ => self.prompt.key(key),
        }
    }

    /*
    fn prompt_del_word(&mut self) {
        let start;
        match self.prompt_text.as_slice().bytes().rev().position(|c| c != ' ' as u8) {
            Some(p) => start = self.prompt_text.len() - p,
            _ => {
                self.prompt_text.clear();
                return;
            }
        }
        match self.prompt_text.as_slice().slice_to(start).bytes().rev().position(|c| c == ' ' as u8) {
            Some(p) => {
                let end = start - p;
                self.prompt_text.truncate(end);
            }
            None => self.prompt_text.clear(),
        }
    }
    */

    pub fn handle_normal_mode_key(&mut self, key: i32) {
        if key < 128 {
            match key as u8 as char {
                'j' => self.down(),
                'k' => self.up(),
                'a' => self.mode = InsertMode,
                ':' => self.mode = CommandMode,
                _ => { },
            }
        }
    }
}

struct RowIter<'a> {
    view: &'a View<'a>,
    row: Row<'a>,
    header: bool,
}

impl<'a> Iterator<Row<'a>> for RowIter<'a> {
    fn next(&mut self) -> Option<Row> {
        match self.row {
            RequestRow(i) => {
                if self.header {
                    if self.view.requests.len() == 0 {
                        self.row = GroupRow(0);
                        self.next()
                    } else {
                        self.header = false;
                        Some(Header("Requests"))
                    }
                } else {
                    if i < self.view.requests.len() {
                        self.row = RequestRow(i+1);
                        Some(RequestRow(i))
                    } else {
                        self.row = GroupRow(0);
                        self.header = true;
                        self.next()
                    }
                }
            },
            GroupRow(i) => {
                if self.header {
                    if self.view.groups == 0 {
                        self.row = FriendRow(0);
                        self.next()
                    } else {
                        self.header = false;
                        Some(Header("Groups"))
                    }
                } else {
                    if i < self.view.groups {
                        self.row = GroupRow(i+1);
                        Some(GroupRow(i))
                    } else {
                        self.row = FriendRow(0);
                        self.header = true;
                        self.next()
                    }
                }
            },
            FriendRow(i) => {
                if self.header {
                    if self.view.friends.len() == 0 {
                        None
                    } else {
                        self.header = false;
                        Some(Header("Friends"))
                    }
                } else {
                    if i < self.view.friends.len() {
                        self.row = FriendRow(i+1);
                        Some(FriendRow(i))
                    } else {
                        None
                    }
                }
            },
            _ => None,
        }
    }
}