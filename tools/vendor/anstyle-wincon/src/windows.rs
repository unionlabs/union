use windows_sys::Win32::System::Console::CONSOLE_CHARACTER_ATTRIBUTES;
use windows_sys::Win32::System::Console::CONSOLE_SCREEN_BUFFER_INFO;
use windows_sys::Win32::System::Console::FOREGROUND_BLUE;
use windows_sys::Win32::System::Console::FOREGROUND_GREEN;
use windows_sys::Win32::System::Console::FOREGROUND_INTENSITY;
use windows_sys::Win32::System::Console::FOREGROUND_RED;

use std::os::windows::io::RawHandle;

const FOREGROUND_CYAN: CONSOLE_CHARACTER_ATTRIBUTES = FOREGROUND_BLUE | FOREGROUND_GREEN;
const FOREGROUND_MAGENTA: CONSOLE_CHARACTER_ATTRIBUTES = FOREGROUND_BLUE | FOREGROUND_RED;
const FOREGROUND_YELLOW: CONSOLE_CHARACTER_ATTRIBUTES = FOREGROUND_GREEN | FOREGROUND_RED;
const FOREGROUND_WHITE: CONSOLE_CHARACTER_ATTRIBUTES =
    FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED;

pub fn get_screen_buffer_info(handle: RawHandle) -> std::io::Result<CONSOLE_SCREEN_BUFFER_INFO> {
    unsafe {
        let handle = std::mem::transmute(handle);
        if handle == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "console is detached",
            ));
        }

        let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if windows_sys::Win32::System::Console::GetConsoleScreenBufferInfo(handle, &mut info) != 0 {
            Ok(info)
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

pub fn set_console_text_attributes(
    handle: RawHandle,
    attributes: CONSOLE_CHARACTER_ATTRIBUTES,
) -> std::io::Result<()> {
    unsafe {
        let handle = std::mem::transmute(handle);
        if handle == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "console is detached",
            ));
        }

        if windows_sys::Win32::System::Console::SetConsoleTextAttribute(handle, attributes) != 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

pub fn get_colors(info: &CONSOLE_SCREEN_BUFFER_INFO) -> (anstyle::AnsiColor, anstyle::AnsiColor) {
    let attributes = info.wAttributes;
    let bg = from_nibble(attributes >> 4);
    let fg = from_nibble(attributes);
    (fg, bg)
}

pub fn set_colors(fg: anstyle::AnsiColor, bg: anstyle::AnsiColor) -> CONSOLE_CHARACTER_ATTRIBUTES {
    to_nibble(bg) << 4 | to_nibble(fg)
}

fn from_nibble(color: CONSOLE_CHARACTER_ATTRIBUTES) -> anstyle::AnsiColor {
    if color & FOREGROUND_WHITE == FOREGROUND_WHITE {
        // 3 bits high
        anstyle::AnsiColor::White
    } else if color & FOREGROUND_CYAN == FOREGROUND_CYAN {
        // 2 bits high
        anstyle::AnsiColor::Cyan
    } else if color & FOREGROUND_YELLOW == FOREGROUND_YELLOW {
        // 2 bits high
        anstyle::AnsiColor::Yellow
    } else if color & FOREGROUND_MAGENTA == FOREGROUND_MAGENTA {
        // 2 bits high
        anstyle::AnsiColor::Magenta
    } else if color & FOREGROUND_RED == FOREGROUND_RED {
        // 1 bit high
        anstyle::AnsiColor::Red
    } else if color & FOREGROUND_GREEN == FOREGROUND_GREEN {
        // 1 bit high
        anstyle::AnsiColor::Green
    } else if color & FOREGROUND_BLUE == FOREGROUND_BLUE {
        // 1 bit high
        anstyle::AnsiColor::Blue
    } else {
        // 0 bits high
        anstyle::AnsiColor::Black
    }
    .bright(color & FOREGROUND_INTENSITY == FOREGROUND_INTENSITY)
}

fn to_nibble(color: anstyle::AnsiColor) -> CONSOLE_CHARACTER_ATTRIBUTES {
    let mut attributes = 0;
    attributes |= match color.bright(false) {
        anstyle::AnsiColor::Black => 0,
        anstyle::AnsiColor::Red => FOREGROUND_RED,
        anstyle::AnsiColor::Green => FOREGROUND_GREEN,
        anstyle::AnsiColor::Yellow => FOREGROUND_YELLOW,
        anstyle::AnsiColor::Blue => FOREGROUND_BLUE,
        anstyle::AnsiColor::Magenta => FOREGROUND_MAGENTA,
        anstyle::AnsiColor::Cyan => FOREGROUND_CYAN,
        anstyle::AnsiColor::White => FOREGROUND_WHITE,
        anstyle::AnsiColor::BrightBlack
        | anstyle::AnsiColor::BrightRed
        | anstyle::AnsiColor::BrightGreen
        | anstyle::AnsiColor::BrightYellow
        | anstyle::AnsiColor::BrightBlue
        | anstyle::AnsiColor::BrightMagenta
        | anstyle::AnsiColor::BrightCyan
        | anstyle::AnsiColor::BrightWhite => unreachable!("brights were toggled off"),
    };
    if color.is_bright() {
        attributes |= FOREGROUND_INTENSITY;
    }
    attributes
}

#[test]
fn to_from_nibble() {
    const COLORS: [anstyle::AnsiColor; 16] = [
        anstyle::AnsiColor::Black,
        anstyle::AnsiColor::Red,
        anstyle::AnsiColor::Green,
        anstyle::AnsiColor::Yellow,
        anstyle::AnsiColor::Blue,
        anstyle::AnsiColor::Magenta,
        anstyle::AnsiColor::Cyan,
        anstyle::AnsiColor::White,
        anstyle::AnsiColor::BrightBlack,
        anstyle::AnsiColor::BrightRed,
        anstyle::AnsiColor::BrightGreen,
        anstyle::AnsiColor::BrightYellow,
        anstyle::AnsiColor::BrightBlue,
        anstyle::AnsiColor::BrightMagenta,
        anstyle::AnsiColor::BrightCyan,
        anstyle::AnsiColor::BrightWhite,
    ];
    for expected in COLORS {
        let nibble = to_nibble(expected);
        let actual = from_nibble(nibble);
        assert_eq!(expected, actual, "Intermediate: {}", nibble);
    }
}
