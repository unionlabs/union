use clap::Parser;
pub use init::InitCommand;
pub use update::UpdateCommand;

mod init;
mod update;

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(about = "Initialize light client")]
    Init(InitCommand),
    #[clap(about = "Update light client")]
    Update(UpdateCommand),
}
