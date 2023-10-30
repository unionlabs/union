extern crate ansi_term;
use ansi_term::Colour;

// This example prints out the 256 colours.
// They're arranged like this:
//
// - 0 to 8 are the eight standard colours.
// - 9 to 15 are the eight bold colours.
// - 16 to 231 are six blocks of six-by-six colour squares.
// - 232 to 255 are shades of grey.

fn main() {

    // First two lines
    for c in 0..8 {
        glow(c, c != 0);
        print!(" ");
    }
    print!("\n");
    for c in 8..16 {
        glow(c, c != 8);
        print!(" ");
    }
    print!("\n\n");

    // Six lines of the first three squares
    for row in 0..6 {
        for square in 0..3 {
            for column in 0..6 {
                glow(16 + square * 36 + row * 6 + column, row >= 3);
                print!(" ");
            }

            print!("  ");
        }

        print!("\n");
    }
    print!("\n");

    // Six more lines of the other three squares
    for row in 0..6 {
        for square in 0..3 {
            for column in 0..6 {
                glow(124 + square * 36 + row * 6 + column, row >= 3);
                print!(" ");
            }

            print!("  ");
        }

        print!("\n");
    }
    print!("\n");

    // The last greyscale lines
    for c in 232..=243 {
        glow(c, false);
        print!(" ");
    }
    print!("\n");
    for c in 244..=255 {
        glow(c, true);
        print!(" ");
    }
    print!("\n");
}

fn glow(c: u8, light_bg: bool) {
    let base = if light_bg { Colour::Black } else { Colour::White };
    let style = base.on(Colour::Fixed(c));
    print!("{}", style.paint(&format!(" {:3} ", c)));
}
