mod cmd;

use crate::cmd::*;
use clap::{Parser, Subcommand};

/// Zote the Mighty
#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Display the version of OpenZFS in use
    Version(Version),
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        Command::Version(cmd) => cmd.run(),
    }
}
