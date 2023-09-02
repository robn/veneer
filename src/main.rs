// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

pub mod ioc;
mod nvpair;
mod sys;

use std::error::Error;
use std::ffi::CString;
use std::io::Error as IOError;

#[macro_use]
extern crate num_derive;

/*
fn print_dataset(dataset: &CStr, stats: Vec<Pair>) -> Result<(), Box<dyn Error>> {
    if let nvpair::PairData::List(ref l) = stats.list.pairs[&CString::new("used")?] {
        if let nvpair::PairData::UInt64(ref u) = l.pairs[&CString::new("value")?] {
            println!("{:?} {:?}", dataset, u);
        }
    }
    Ok(())
}
*/

fn main() -> Result<(), Box<dyn Error>> {
    let mut ioc = ioc::Handle::open()?;

    let configs = ioc.pool_configs()?;
    for pool in configs.keys() {
        println!("{:?}", pool);

        /*
        let stats = ioc.pool_stats(pool)?;
        println!("{:#?}", stats);

        let props = ioc.pool_get_props(pool)?;
        println!("{:#?}", props);

        let stats = ioc.objset_stats(pool)?;
        println!("{:#?}", stats);
        */

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
