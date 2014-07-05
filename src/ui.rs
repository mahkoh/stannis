use nc = ncurses;
use signals;
use signals::{Signals, Pipe, Hangup, Terminate, WinSize};
use tox;
use std;
use tox::core::{Tox};
use fdpoll::{FDPoll, Read};
use term;
use colors::*;
use contacts;

use std::rc::{Rc};
use std::comm::{Select};

pub fn bootstrap() -> Tox {
    let tox = Tox::new(true).unwrap();
    tox
}

pub fn run() {
    let tox = Rc::new(bootstrap());
    let mut ui = Ui::new(tox);
    ui.run();
}

struct Ui<'a> {
    tox: Rc<Tox>,
    contacts: contacts::View<'a>,
}

impl<'a> Ui<'a> {
    pub fn new(tox: Rc<Tox>) -> Ui {
        use libc::{c_int, c_char};
        extern {
            fn use_default_colors() -> c_int;
            fn set_escdelay(size: c_int) -> c_int;
            fn setlocale(category: c_int, locale: *const c_char) -> *const c_char;
        }
        unsafe { setlocale(0, [0i8].as_ptr()); }
        nc::initscr();
        nc::start_color();
        unsafe { use_default_colors(); }
        nc::cbreak();
        unsafe { set_escdelay(25); }
        nc::keypad(nc::constants::stdscr, true);
        nc::noecho();
        nc::nonl();

        nc::init_color(COLOR_BACKGROUND,  0x1C * 4, 0x1C * 4, 0x1C * 4);
        nc::init_color(COLOR_HEADERS,     0xA3 * 4, 0x81 * 4, 0xA6 * 4);
        nc::init_color(COLOR_ENTRY,       0xDD * 4, 0xDD * 4, 0xDD * 4);
        nc::init_color(COLOR_SEPARATOR,   0x2B * 4, 0x33 * 4, 0x36 * 4);
        nc::init_color(COLOR_SELECTED_BG, 0x40 * 4, 0x40 * 4, 0x40 * 4);
        nc::init_color(COLOR_STATUS_BG,   0x61 * 4, 0x20 * 4, 0x20 * 4);
        nc::init_color(COLOR_PROMPT_BG,   0x36 * 4, 0x20 * 4, 0x65 * 4);

        nc::init_pair(COLOR_PAIR_DEFAULT,   COLOR_ENTRY,     COLOR_BACKGROUND);
        nc::init_pair(COLOR_PAIR_HEADER,    COLOR_HEADERS,   COLOR_BACKGROUND);
        nc::init_pair(COLOR_PAIR_SEPARATOR, COLOR_SEPARATOR, COLOR_BACKGROUND);
        nc::init_pair(COLOR_PAIR_SELECTED,  COLOR_ENTRY,     COLOR_SELECTED_BG);
        nc::init_pair(COLOR_PAIR_STATUS,    COLOR_ENTRY,     COLOR_STATUS_BG);
        nc::init_pair(COLOR_PAIR_PROMPT,    COLOR_ENTRY,     COLOR_PROMPT_BG);

        nc::bkgd(' ' as u32 | nc::COLOR_PAIR(COLOR_PAIR_DEFAULT) as u32);

        Ui {
            tox: tox,
            contacts: contacts::View::new(),
        }
    }

    pub fn run(&mut self) {
        let sigs = Signals::new().unwrap();
        // sigs.subscribe(Interrupt);
        sigs.subscribe(Hangup);
        sigs.subscribe(Terminate);
        sigs.subscribe(Pipe);
        sigs.subscribe(WinSize);

        let fdpoll = FDPoll::new(10).unwrap();
        fdpoll.add(0, Read).unwrap();
        fdpoll.wait().unwrap();

        let select = Select::new();
        let mut fdpoll_hdl = select.handle(&fdpoll.rcv);
        unsafe { fdpoll_hdl.add(); }
        let mut sig_hdl = select.handle(sigs.receiver());
        unsafe { sig_hdl.add(); }
        let tox = self.tox.clone();
        let mut tox_hdl = select.handle(&tox.events);
        unsafe { tox_hdl.add(); }

        loop {
            self.update();

            let r = select.wait();

            if r == tox_hdl.id() {
                for e in self.tox.events() {

                }
            }

            if r == fdpoll_hdl.id() {
                fdpoll.rcv.recv().ok();
                // We're only watching stdin right now.
                self.handle_key();
                fdpoll.wait().ok();
            }

            if r == sig_hdl.id() {
                for s in sigs.iter() {
                    match s {
                        Hangup | Terminate => self.shutdown(),
                        WinSize => self.resize(),
                        _ => { /* ignore sigpipe */ },
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        self.update_statusline();
        self.contacts.update();
        normal!(COLOR_PAIR_DEFAULT);
    }

    fn update_statusline(&mut self) {
        nc::move(nc::LINES-1, 0);
        normal!(COLOR_PAIR_STATUS);
        nc::clrtoeol();
        normal!(COLOR_PAIR_DEFAULT);
    }

    fn handle_key(&mut self) {
        self.contacts.handle_key(nc::getch());
    }

    fn shutdown(&mut self) {
    }

    fn resize(&mut self) {
        use libc::{c_int};
        extern {
            fn resizeterm(lines: c_int, columns: c_int) -> c_int;
        }
        let (columns, lines) = term::dimensions();
        unsafe { resizeterm(lines as c_int, columns as c_int); }
        self.contacts.resize();
    }
}

