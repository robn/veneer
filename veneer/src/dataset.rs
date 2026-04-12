// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2025, Rob Norris <robn@despairlabs.com>

use crate::util::AutoString;
use crate::{Error, Handle};
use nvpair::PairList;
use std::rc::Rc;

pub struct Dataset {
    handle: Rc<Handle>,
    name: AutoString,
}

impl Dataset {
    pub(crate) fn new(handle: Rc<Handle>, name: AutoString) -> Dataset {
        Dataset { handle, name }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    fn get_prop(&self, prop: &str) -> Result<Option<PairList>, Error> {
        let dslist = self.handle.get_dataset(&self.name)?;
        Ok(dslist.get_list(prop).cloned())
    }

    pub fn get_prop_u64(&self, prop: &str) -> Result<Option<u64>, Error> {
        Ok(self.get_prop(prop)?.and_then(|l| l.get_u64("value")))
    }

    pub fn get_prop_string(&self, prop: &str) -> Result<Option<String>, Error> {
        Ok(self
            .get_prop(prop)?
            .and_then(|l| l.get_c_string("value"))
            .map(|cs| cs.to_string_lossy().to_string()))
    }
}
