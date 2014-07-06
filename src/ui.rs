use nc = ncurses;
use signals;
use signals::{Signals, Pipe, Hangup, Terminate, WinSize};
use tox;
use tox::core::{Tox, Address, FaerrToolong, FaerrOwnkey, FaerrAlreadysent,
                FaerrBadchecksum, Event, NameChange};
use std;
use fdpoll::{FDPoll, Read};
use term;
use colors::*;
use contacts;
use commands::{Command, Quit, Add};
use commands;

use std::rc::{Rc};
use std::comm::{Select};

pub fn bootstrap() -> Tox {
    let tox = Tox::new(true).unwrap();

    let addresses = [
        ("192.254.75.98",   33445, "951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F"),
        ("144.76.60.215",   33445, "04119E835DF3E78BACF0F84235B300546AF8B936F035185E2A8E9E0A67C8924F"),
        ("23.226.230.47",   33445, "A09162D68618E742FFBCA1C2C70385E6679604B2D80EA6E84AD0996A1AC8A074"),
        ("37.187.20.216",   33445, "4FD54CFD426A338399767E56FD0F44F5E35FA8C38C8E87C8DC3FEAC0160F8E17"),
        ("54.199.139.199",  33445, "7F9C31FE850E97CEFD4C4591DF93FC757C7C12549DDD55F8EEAECC34FE76C029"),
        ("109.169.46.133",  33445, "7F31BFC93B8E4016A902144D0B110C3EA97CB7D43F1C4D21BCAE998A7C838821"),
        ("192.210.149.121", 33445, "F404ABAA1C99A9D37D61AB54898F56793E1DEF8BD46B1038B9D822E8460FAB67"),
    ];

    for &(ip, port, key) in addresses.iter() {
        let ip = ip.to_string();
        let key = box from_str(key).unwrap();
        if tox.bootstrap_from_address(ip, true, port, key).is_ok() {
            break;
        }
    }

    tox
}

pub fn run() {
    let tox = Rc::new(bootstrap());
    let mut ui = Ui::new(tox);
    ui.run();
}

enum StatusMessage {
    NoMsg,
    Error(&'static str),
}

struct Ui<'a> {
    tox: Rc<Tox>,
    contacts: contacts::View<'a>,
    shutdown: bool,
    status: StatusMessage,
    needs_update: bool,
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
            shutdown: false,
            status: NoMsg,
            needs_update: true,
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
                for e in tox.events() {
                    self.tox_event(e);
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
                        Hangup | Terminate => self.shutdown = true,
                        WinSize => self.resize(),
                        _ => { /* ignore sigpipe */ },
                    }
                }
            }

            if self.shutdown {
                break;
            }
        }

        fdpoll.abort();
        nc::endwin();
    }

    fn tox_event(&mut self, e: Event) {
        match e {
            NameChange(i, name) => self.tox_name_change(i, name),
            _ => { }
        }
    }

    fn tox_name_change(&mut self, id: i32, name: String) {
        self.contacts.tox_name_change(id, name);
        self.needs_update = true;
    }

    fn update(&mut self) {
        if !self.needs_update {
            return;
        }
        self.needs_update = false;
        self.update_statusline();
        self.contacts.update();
        normal!(COLOR_PAIR_DEFAULT);
    }

    fn update_statusline(&mut self) {
        nc::move(nc::LINES-1, 0);
        normal!(COLOR_PAIR_STATUS);
        match self.status {
            NoMsg => { },
            Error(s) => {
                nc::addstr(s);
            },
        }
        nc::clrtoeol();
        normal!(COLOR_PAIR_DEFAULT);
    }

    fn handle_key(&mut self) {
        match self.contacts.handle_key(nc::getch()) {
            Some(c) => self.handle_command(c),
            _ => { },
        }
        self.needs_update = true;
    }

    fn handle_command(&mut self, c: commands::Result) {
        let c = match c {
            Ok(c) => c,
            Err(s) => {
                self.status = Error(s);
                return;
            },
        };
        match c {
            Quit => self.shutdown = true,
            Add(addr, msg) => self.tox_add(addr, msg),
        }
    }

    fn tox_add(&mut self, addr: Address, msg: String) {
        let res = self.tox.add_friend(box addr, msg);
        if res.is_ok() {
            self.contacts.tox_add(res.ok().unwrap());
        } else {
            match res.unwrap_err() {
                FaerrToolong => self.status = Error("message too long"),
                FaerrOwnkey => self.status = Error("own key"),
                FaerrAlreadysent => self.status = Error("already sent"),
                FaerrBadchecksum => self.status = Error("bad checksum"),
                _ => self.status = Error("unknown error"),
            }
        }
        self.needs_update = true;
    }

    fn resize(&mut self) {
        use libc::{c_int};
        extern {
            fn resizeterm(lines: c_int, columns: c_int) -> c_int;
        }
        let (columns, lines) = term::dimensions();
        unsafe { resizeterm(lines as c_int, columns as c_int); }
        self.contacts.resize();
        self.needs_update = true;
    }
}

