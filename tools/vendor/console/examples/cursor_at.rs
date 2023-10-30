extern crate console;

use std::io;
use std::thread;
use std::time::Duration;

use console::{style, Term};

fn write_chars() -> io::Result<()> {
    let term = Term::stdout();
    let (heigth, width) = term.size();
    for x in 0..width {
        for y in 0..heigth {
            term.move_cursor_to(x as usize, y as usize)?;
            let text = if (x + y) % 2 == 0 {
                format!("{}", style(x % 10).black().on_red())
            } else {
                format!("{}", style(x % 10).red().on_black())
            };

            term.write_str(&text)?;
            thread::sleep(Duration::from_micros(600));
        }
    }
    Ok(())
}

fn main() {
    write_chars().unwrap();
}
