// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::error::Error;
use tabled::{builder::Builder, settings::Style};
use veneer::zfs::{self, Vdev};

fn main() -> Result<(), Box<dyn Error>> {
    let z = zfs::open()?;

    let mut tb = Builder::default();
    tb.set_header(["name", "type", "state", "read", "write", "cksum", "slow"]);

    fn push_vdev(tb: &mut Builder, name: String, vd: &Vdev) -> Result<(), Box<dyn Error>> {
        let vs = vd.stats()?;
        tb.push_record([
            format!("{}", name),
            format!("{:?}", vd.typ()),
            format!("{}", vs.state),
            format!("{}", vs.read_errors),
            format!("{}", vs.write_errors),
            format!("{}", vs.checksum_errors),
            format!("{}", vs.slow_ios),
        ]);

        for cvd in vd.children()? {
            push_vdev(tb, cvd.guid().to_string(), &cvd)?;
        }

        Ok(())
    }

    for pool in z.pools()? {
        let root = pool.root_vdev()?;

        push_vdev(&mut tb, pool.name(), &root)?;
    }

    let table = tb.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}
