// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

// XXX convert to high-level api, when we have one

use std::error::Error;
use std::ffi::CString;
use std::io::Error as IOError;
use veneer::ioc;

fn main() -> Result<(), Box<dyn Error>> {
    let mut ioc = ioc::Handle::open()?;

    let configs = ioc.pool_configs()?;
    for pool in configs.keys() {
        let mut stack: Vec<(CString, u64)> = vec![(pool.into(), 0)];
        while let Some((name, cookie)) = stack.pop() {
            match ioc.dataset_list_next(&name, cookie) {
                Ok(is) => {
                    stack.push((name, is.cookie));
                    let name = is.name.into();
                    println!("{:?}", name);
                    stack.push((name, 0));
                }
                Err(e) => {
                    let ioe = e.downcast::<IOError>()?;
                    ioe.raw_os_error().filter(|n| *n == 3).ok_or(ioe)?; // ESRCH
                }
            }
        }
    }

    Ok(())
}