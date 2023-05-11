use argh::FromArgs;

mod cmd;
use cmd::clone::CloneCmd;
use cmd::compile::CompileCmd;

#[derive(Debug, FromArgs)]
/// App
struct App {
    #[argh(subcommand)]
    cmd: Command,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Command {
    Clone(CloneCmd),
    Compile(CompileCmd),
}

fn main() {
    let app: App = argh::from_env();

    match app.cmd {
        Command::Clone(clone) => clone.run(),
        Command::Compile(compile) => compile.run(),
    }
}
