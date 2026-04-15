// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2025, Rob Norris <robn@despairlabs.com>

use crate::error::{Error, ResponseError};
use crate::{Dataset, Handle, Vdev};

use anystring::AnyString;
use std::rc::Rc;

pub struct Pool {
    handle: Rc<Handle>,
    name: AnyString,
}

impl Pool {
    pub(crate) fn new(handle: Rc<Handle>, name: AnyString) -> Pool {
        Pool { handle, name }
    }

    pub fn name(&self) -> &AnyString {
        &self.name
    }

    pub fn root_vdev(&self) -> Result<Vdev, Error> {
        let pl = self.handle.get_pool(&self.name)?;
        let vl = pl
            .get_list("vdev_tree")
            .ok_or_else(|| ResponseError::MissingField {
                field: "vdev_tree".into(),
            })?;
        Vdev::new(self.handle.clone(), self.name.clone(), vl)
    }

    pub fn datasets(&self) -> Result<Vec<Dataset>, Error> {
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
