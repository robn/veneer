mod list;

use list::List;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(about)]
pub struct Pool {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all pools
    List(List),
}

impl Pool {
    pub(crate) fn run(&self) {
        match &self.cmd {
            Command::List(cmd) => cmd.run(),
        }
    }
}
