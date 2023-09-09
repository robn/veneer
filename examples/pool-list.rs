// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::error::Error;
use veneer::zfs;

fn main() -> Result<(), Box<dyn Error>> {
    let z = zfs::open()?;

    for pool in z.pools()? {
        println!("{}", pool.name());
    }

    Ok(())
}
