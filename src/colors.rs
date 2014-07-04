#![macro_escape]

// Don't change the first 16 colors
pub static COLOR_BACKGROUND:  i16 = 16;
pub static COLOR_HEADERS:     i16 = 17;
pub static COLOR_ENTRY:       i16 = 18;
pub static COLOR_SEPARATOR:   i16 = 19;
pub static COLOR_SELECTED_BG: i16 = 20;
pub static COLOR_STATUS_BG:   i16 = 21;
pub static COLOR_PROMPT_BG:   i16 = 22;

pub static COLOR_PAIR_DEFAULT:   i16 = 1;
pub static COLOR_PAIR_HEADER:    i16 = 2;
pub static COLOR_PAIR_SEPARATOR: i16 = 3;
pub static COLOR_PAIR_SELECTED:  i16 = 4;
pub static COLOR_PAIR_STATUS:    i16 = 5;
pub static COLOR_PAIR_PROMPT:    i16 = 6;

#[macro_escape]
macro_rules! bold {
    ($c:expr) => {
        nc::bkgdset(nc::COLOR_PAIR($c) as u32 | nc::A_BOLD() as u32);
    }
}

#[macro_escape]
macro_rules! normal {
    ($c:expr) => {
        nc::bkgdset(nc::COLOR_PAIR($c) as u32);
    }
}

