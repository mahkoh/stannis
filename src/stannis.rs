#![crate_id = "stannis"]
#![feature(globs, macro_rules)]

extern crate ncurses;
extern crate tox;
extern crate signals;
extern crate fdpoll;
extern crate debug;
extern crate libc;

mod colors;
mod ui;
mod term;
mod contacts;
mod utfbuf;
mod prompt;
mod commands;

fn main() {
    ui::run();
}
