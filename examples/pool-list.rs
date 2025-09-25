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
    tb.push_record(["name", "type", "state", "read", "write", "cksum", "slow"]);

    fn push_vdev(
        tb: &mut Builder,
        vd: &Vdev,
        indent: usize,
        last: bool,
    ) -> Result<(), Box<dyn Error>> {
        let vs = vd.stats()?;
        let hook = if last { "└ " } else { "├ " };
        let graph = match indent {
            0 => "".into(),
            1 => hook.into(),
            _ => iter::repeat("│ ")
                .take(indent - 1)
                .chain(iter::once(hook))
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

        let mut vds = vd.children()?;
        let last = vds.pop();
        for cvd in vds {
            push_vdev(tb, &cvd, indent + 1, false)?;
        }
        if let Some(cvd) = last {
            push_vdev(tb, &cvd, indent + 1, true)?;
        }

        Ok(())
    }

    for pool in z.pools()? {
        let root = pool.root_vdev()?;

        push_vdev(&mut tb, &root, 0, false)?;
    }

    let table = tb.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}
