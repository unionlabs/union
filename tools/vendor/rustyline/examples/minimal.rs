use rustyline::{Editor, Result};

/// Minimal REPL
fn main() -> Result<()> {
    let mut rl = Editor::<()>::new();
    loop {
        let line = rl.readline("> ")?; // read
        println!("Line: {}", line); // eval / print
    } // loop
}
