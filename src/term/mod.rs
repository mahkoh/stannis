extern crate libc;

pub use self::term::{dimensions};

pub mod cwidth;

#[cfg(windows)]
mod term {
    use libc::{HANDLE, DWORD, BOOL, SHORT, WORD};
    use std::mem::{zeroed};

    #[allow(non_camel_case_types)]
    struct COORD {
      X: SHORT,
      Y: SHORT,
    }

    #[allow(non_camel_case_types)]
    struct SMALL_RECT {
      Left: SHORT,
      Top: SHORT,
      Right: SHORT,
      Bottom: SHORT,
    }

    #[allow(non_camel_case_types)]
    struct CONSOLE_SCREEN_BUFFER_INFO {
      dwSize: COORD,
      dwCursorPosition: COORD,
      wAttributes: WORD,
      srWindow: SMALL_RECT,
      dwMaximumWindowSize: COORD,
    }

    static STD_OUTPUT_HANDLE: DWORD = -11;

    extern "system" {
        fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
        fn GetConsoleScreenBufferInfo(
            hConsoleOutput: HANDLE,
            lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO) -> BOOL;
    }

    pub fn dimensions() -> (uint, uint) {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = unsafe { zeroed() };
        unsafe { 
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            GetConsoleScreenBufferInfo(handle,
                                       &mut csbi as *mut CONSOLE_SCREEN_BUFFER_INFO);
        }
        let cols = csbi.srWindow.Right - csbi.srWindow.Left + 1;
        let rows = csbi.srWindow.Bottom - csbi.srWindow.Top + 1;
        (cols as uint, rows as uint)
    }
}

#[cfg(unix)]
mod term {
    use libc::{c_int, c_ushort, STDOUT_FILENO};
    use std::mem::{zeroed};

    #[allow(non_camel_case_types)]
    struct winsize {
        ws_row: c_ushort,
        ws_col: c_ushort,
        #[allow(dead_code)]
        ws_xpixel: c_ushort,
        #[allow(dead_code)]
        ws_ypixel: c_ushort,
    }

    #[cfg(target_os = "linux")]
    #[cfg(target_os = "android")]
    static TIOCGWINSZ: c_int = 0x5413;
    #[cfg(target_os = "freebsd")]
    #[cfg(target_os = "macos")]
    static TIOCGWINSZ: c_int = 0x40087468;

    extern {
        fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    }

    pub fn dimensions() -> (uint, uint) {
        let mut w: winsize = unsafe { zeroed() };
        unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut w as *mut winsize); }
        (w.ws_col as uint, w.ws_row as uint)
    }
}
