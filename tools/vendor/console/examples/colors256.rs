use console::style;

fn main() {
    for i in 0..=255 {
        print!("{:03} ", style(i).color256(i));
        if i % 16 == 15 {
            println!("");
        }
    }

    for i in 0..=255 {
        print!("{:03} ", style(i).black().on_color256(i));
        if i % 16 == 15 {
            println!("");
        }
    }
}
