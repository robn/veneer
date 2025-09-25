// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2025, Rob Norris <robn@despairlabs.com>

use crate::util::AutoString;
use crate::{Dataset, Handle, Vdev};

use std::error::Error;
use std::io::Error as IOError;
use std::io::ErrorKind as IOErrorKind;
use std::rc::Rc;

pub struct Pool {
    handle: Rc<Handle>,
    name: AutoString,
}

impl Pool {
    pub(crate) fn new(handle: Rc<Handle>, name: AutoString) -> Pool {
        Pool { handle, name }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn root_vdev(&self) -> Result<Vdev, Box<dyn Error>> {
        let pl = self.handle.get_pool(&self.name)?;
        let vl = pl
            .get_list("vdev_tree")
            .ok_or_else(|| IOError::from(IOErrorKind::NotFound))?;
        Vdev::new(self.handle.clone(), self.name.clone(), vl)
    }

    pub fn datasets(&self) -> Result<Vec<Dataset>, Box<dyn Error>> {
        Ok(self
            .handle
            .get_dataset_list()?
            .iter()
            .filter(|ds| {
                ds.to_bytes().starts_with(self.name.as_bytes())
                    && (ds.to_bytes().len() == self.name.as_bytes().len()
                        || ds.to_bytes()[self.name.as_bytes().len()] == b'/')
            })
            .map(|ds| Dataset::new(self.handle.clone(), ds.into()))
            .collect())
    }
}
