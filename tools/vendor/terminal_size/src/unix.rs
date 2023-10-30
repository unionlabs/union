use super::{Height, Width};
use std::os::unix::io::RawFd;

/// Returns the size of the terminal defaulting to STDOUT, if available.
///
/// If STDOUT is not a tty, returns `None`
pub fn terminal_size() -> Option<(Width, Height)> {
    terminal_size_using_fd(libc::STDOUT_FILENO)
}

/// Returns the size of the terminal using the given file descriptor, if available.
///
/// If the given file descriptor is not a tty, returns `None`
pub fn terminal_size_using_fd(fd: RawFd) -> Option<(Width, Height)> {
    use libc::ioctl;
    use libc::isatty;
    use libc::{winsize as WinSize, TIOCGWINSZ};
    let is_tty: bool = unsafe { isatty(fd) == 1 };

    if !is_tty {
        return None;
    }

    let mut winsize = WinSize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    if unsafe { ioctl(fd, TIOCGWINSZ.into(), &mut winsize) } == -1 {
        return None;
    }

    let rows = winsize.ws_row;
    let cols = winsize.ws_col;

    if rows > 0 && cols > 0 {
        Some((Width(cols), Height(rows)))
    } else {
        None
    }
}

#[test]
/// Compare with the output of `stty size`
fn compare_with_stty() {
    use std::process::Command;
    use std::process::Stdio;

    let (rows, cols) = if cfg!(target_os = "illumos") {
        // illumos stty(1) does not accept a device argument, instead using
        // stdin unconditionally:
        let output = Command::new("stty")
            .stdin(Stdio::inherit())
            .output()
            .unwrap();
        assert!(output.status.success());

        // stdout includes the row and columns thus: "rows = 80; columns = 24;"
        let vals = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| {
                // Split each line on semicolons to get "k = v" strings:
                line.split(';')
                    .map(str::trim)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .flatten()
            .filter_map(|term| {
                // split each "k = v" string and look for rows/columns:
                match term.splitn(2, " = ").collect::<Vec<_>>().as_slice() {
                    ["rows", n] | ["columns", n] => Some(n.parse().unwrap()),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        (vals[0], vals[1])
    } else {
        let output = if cfg!(target_os = "linux") {
            Command::new("stty")
                .arg("size")
                .arg("-F")
                .arg("/dev/stderr")
                .stderr(Stdio::inherit())
                .output()
                .unwrap()
        } else {
            Command::new("stty")
                .arg("-f")
                .arg("/dev/stderr")
                .arg("size")
                .stderr(Stdio::inherit())
                .output()
                .unwrap()
        };

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        // stdout is "rows cols"
        let mut data = stdout.split_whitespace();
        println!("{}", stdout);
        let rows = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
        let cols = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
        (rows, cols)
    };
    println!("{} {}", rows, cols);

    if let Some((Width(w), Height(h))) = terminal_size() {
        assert_eq!(rows, h);
        assert_eq!(cols, w);
    } else {
        panic!("terminal_size() return None");
    }
}
