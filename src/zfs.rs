// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crate::ioc;
use crate::nvpair::PairList;
use crate::util::AutoString;
use std::cell::{OnceCell, RefCell};
use std::error::Error;
use std::ffi::CStr;

pub struct Handle {
    ioc: RefCell<ioc::Handle>,
    config: OnceCell<PairList>,
}

pub fn open() -> Result<Handle, Box<dyn Error>> {
    Ok(Handle {
        ioc: RefCell::new(ioc::Handle::open()?),
        config: OnceCell::default(),
    })
}

impl Handle {
    fn get_config(&self) -> Result<&PairList, Box<dyn Error>> {
        // XXX get_or_try_init [feature "once_cell_try"] would be better
        //self.config.get_or_try_init(|| self.ioc.borrow_mut().pool_configs()?);
        if let Some(c) = self.config.get() {
            return Ok(c);
        }

        let c = self.ioc.borrow_mut().pool_configs()?;
        let _ = self.config.set(c);

        Ok(self.config.get().unwrap())
    }

    pub fn pools(&self) -> Result<Vec<Pool>, Box<dyn Error>> {
        Ok(self.get_config()?.keys().map(|p| Pool::new(p)).collect())
    }
}

pub struct Pool {
    name: AutoString,
}

impl Pool {
    fn new(name: &CStr) -> Pool {
        Pool { name: name.into() }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
