extern crate ansi_term;
use ansi_term::{Style, Colour::*};

// This example prints out the 16 basic colours.

fn main() {
    let normal = Style::default();

    println!("{} {}", normal.paint("Normal"), normal.bold().paint("bold"));
    println!("{} {}", Black.paint("Black"),   Black.bold().paint("bold"));
    println!("{} {}", Red.paint("Red"),       Red.bold().paint("bold"));
    println!("{} {}", Green.paint("Green"),   Green.bold().paint("bold"));
    println!("{} {}", Yellow.paint("Yellow"), Yellow.bold().paint("bold"));
    println!("{} {}", Blue.paint("Blue"),     Blue.bold().paint("bold"));
    println!("{} {}", Purple.paint("Purple"), Purple.bold().paint("bold"));
    println!("{} {}", Cyan.paint("Cyan"),     Cyan.bold().paint("bold"));
    println!("{} {}", White.paint("White"),   White.bold().paint("bold"));
}
