// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2025, Rob Norris <robn@despairlabs.com>

use nvpair::PairList;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};

use crate::error::{Error, ResponseError};
use veneer_ioctl::Handle as IOCHandle;

pub struct Handle {
    ioc: RefCell<IOCHandle>,
}

impl Handle {
    pub(crate) fn open() -> Result<Handle, Error> {
        Ok(Handle {
            ioc: RefCell::new(IOCHandle::open()?),
        })
    }

    pub(crate) fn get_config(&self) -> Result<PairList, Error> {
        Ok(self.ioc.borrow_mut().pool_configs()?)
    }

    pub(crate) fn get_pool(&self, name: impl AsRef<CStr>) -> Result<PairList, Error> {
        Ok(self.ioc.borrow_mut().pool_stats(name.as_ref())?)
    }

    pub(crate) fn get_vdev(
        &self,
        name: impl AsRef<CStr>,
        guid: u64,
    ) -> Result<Option<PairList>, Error> {
        let plist = self.get_pool(name)?;
        let top = plist
            .get_list("vdev_tree")
            .ok_or_else(|| ResponseError::MissingField {
                field: "vdev_tree".into(),
            })?;

        let mut vds: VecDeque<&PairList> = VecDeque::new();
        vds.push_back(top);

        while let Some(vd) = vds.pop_front() {
            if let Some(vguid) = vd.get_u64("guid") {
                if vguid == guid {
                    return Ok(Some(vd.clone()));
                }

                vd.get_list_slice("children")
                    .into_iter()
                    .flatten()
                    .for_each(|cvd| vds.push_back(cvd));
            }
        }

        Ok(None)
    }

    pub(crate) fn get_dataset(&self, name: impl AsRef<CStr>) -> Result<PairList, Error> {
        Ok(self.ioc.borrow_mut().objset_stats(name.as_ref())?)
    }

    pub(crate) fn get_dataset_list(&self) -> Result<Vec<CString>, Error> {
        let mut list: Vec<CString> = vec![];

        for pool in self.get_config()?.keys() {
            let _ = self.get_dataset(pool)?;
            list.push(pool.into());

            let mut stack: Vec<(CString, u64)> = vec![(pool.into(), 0)];
            while let Some((name, cookie)) = stack.pop() {
                let next = self
                    .ioc
                    .borrow_mut()
                    .dataset_list_next(&name, cookie)
                    .and_then(|is| Ok(Some(is)))
                    .or_else(|e| match e {
                        veneer_ioctl::Error::IOError(ref ioe) => match ioe.raw_os_error() {
                            Some(3) => Ok(None),
                            _ => Err(e),
                        },
                        _ => Err(e),
                    })?;
                if let Some(is) = next {
                    list.push(is.name.clone());
                    stack.push((name, is.cookie));
                    stack.push((is.name, 0));
                }
            }
        }

        Ok(list)
    }
}
