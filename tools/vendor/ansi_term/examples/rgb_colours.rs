extern crate ansi_term;
use ansi_term::{Style, Colour};

// This example prints out a colour gradient in a grid by calculating each
// character’s red, green, and blue components, and using 24-bit colour codes
// to display them.

const WIDTH:  i32 = 80;
const HEIGHT: i32 = 24;

fn main() {
    for row in 0 .. HEIGHT {
        for col in 0 .. WIDTH {
            let r = (row * 255 / HEIGHT) as u8;
            let g = (col * 255 / WIDTH) as u8;
            let b = 128;

            print!("{}", Style::default().on(Colour::RGB(r, g, b)).paint(" "));
        }

        print!("\n");
    }
}
