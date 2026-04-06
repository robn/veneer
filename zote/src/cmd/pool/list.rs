use clap::Parser;

#[derive(Parser, Debug)]
pub struct List;

impl List {
    pub(crate) fn run(&self) {
        let z = veneer::open().unwrap();

        for pool in z.pools().unwrap() {
            println!("{}", pool.name());
        }
    }
}
