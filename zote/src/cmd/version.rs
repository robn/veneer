use clap::Parser;

#[derive(Parser, Debug)]
pub struct Version;

impl Version {
    pub(crate) fn run(&self) {
        println!("{} {}", clap::crate_name!(), clap::crate_version!());
    }
}
