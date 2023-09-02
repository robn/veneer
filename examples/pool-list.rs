// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

// XXX convert to high-level api, when we have one

use std::error::Error;
use veneer::ioc;

fn main() -> Result<(), Box<dyn Error>> {
    let mut ioc = ioc::Handle::open()?;

    let configs = ioc.pool_configs()?;
    for pool in configs.keys() {
        println!("{:?}", pool);
    }

    Ok(())
}
