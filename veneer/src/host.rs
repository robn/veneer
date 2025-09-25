// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crate::{Handle, Pool};

use std::error::Error;
use std::rc::Rc;

pub struct Host(Rc<Handle>);

impl Host {
    pub(crate) fn open() -> Result<Host, Box<dyn Error>> {
        Ok(Host(Rc::new(Handle::open()?)))
    }

    pub fn pools(&self) -> Result<Vec<Pool>, Box<dyn Error>> {
        Ok(self
            .0
            .get_config()?
            .keys()
            .map(|p| Pool::new(self.0.clone(), p.into()))
            .collect())
    }
}
