// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::error::Error;
use std::iter;
use tabled::{builder::Builder, settings::Style};
use veneer::zfs::{self, Vdev};

fn main() -> Result<(), Box<dyn Error>> {
    let z = zfs::open()?;

    let mut tb = Builder::default();
    tb.set_header(["name", "type", "state", "read", "write", "cksum", "slow"]);

    fn push_vdev(tb: &mut Builder, vd: &Vdev, indent: usize) -> Result<(), Box<dyn Error>> {
        let vs = vd.stats()?;
        let graph = match indent {
            0 => "".into(),
            1 => "└ ".into(),
            _ => iter::repeat("  ")
                .take(indent - 1)
                .chain(iter::once("└ "))
                .collect::<String>(),
        };
        tb.push_record([
            format!("{}{}", graph, vd.name()),
            format!("{:?}", vd.typ()),
            format!("{}", vs.state),
            format!("{}", vs.read_errors),
            format!("{}", vs.write_errors),
            format!("{}", vs.checksum_errors),
            format!("{}", vs.slow_ios),
        ]);

        for cvd in vd.children()? {
            push_vdev(tb, &cvd, indent + 1)?;
        }

        Ok(())
    }

    for pool in z.pools()? {
        let root = pool.root_vdev()?;

        push_vdev(&mut tb, &root, 0)?;
    }

    let table = tb.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}
