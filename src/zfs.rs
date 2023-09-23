// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crate::ioc;
use crate::nvpair::PairList;
use crate::nvtypes;
use crate::util::AutoString;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::io::Error as IOError;
use std::io::ErrorKind as IOErrorKind;
use std::rc::Rc;

struct Handle {
    ioc: RefCell<ioc::Handle>,
}

impl Handle {
    fn open() -> Result<Handle, Box<dyn Error>> {
        Ok(Handle {
            ioc: RefCell::new(ioc::Handle::open()?),
        })
    }

    fn get_config(&self) -> Result<PairList, Box<dyn Error>> {
        self.ioc.borrow_mut().pool_configs()
    }

    fn get_pool(&self, name: impl AsRef<CStr>) -> Result<PairList, Box<dyn Error>> {
        self.ioc.borrow_mut().pool_stats(name.as_ref())
    }

    fn get_vdev(
        &self,
        name: impl AsRef<CStr>,
        guid: u64,
    ) -> Result<Option<PairList>, Box<dyn Error>> {
        let plist = self.get_pool(name)?;
        let top = plist
            .get("vdev_tree")
            .and_then(|p| p.as_list())
            .ok_or_else(|| IOError::from(IOErrorKind::NotFound))?; // XXX should be impossible, maybe
                                                                   // just panic?

        let mut vds: VecDeque<&PairList> = VecDeque::new();
        vds.push_back(top);

        while let Some(vd) = vds.pop_front() {
            if let Some(vguid) = vd.get("guid").and_then(|p| p.to_u64()) {
                if vguid == guid {
                    return Ok(Some(vd.clone()));
                }

                vd.get("children")
                    .and_then(|p| p.as_list_slice())
                    .into_iter()
                    .flatten()
                    .for_each(|cvd| vds.push_back(cvd));
            }
        }

        Ok(None)
    }

    fn get_dataset(&self, name: impl AsRef<CStr>) -> Result<PairList, Box<dyn Error>> {
        self.ioc.borrow_mut().objset_stats(name.as_ref())
    }

    fn get_dataset_list(&self) -> Result<Vec<CString>, Box<dyn Error>> {
        let mut list: Vec<CString> = vec![];

        for pool in self.get_config()?.keys() {
            let _ = self.get_dataset(pool)?;
            list.push(pool.into());

            let mut stack: Vec<(CString, u64)> = vec![(pool.into(), 0)];
            while let Some((name, cookie)) = stack.pop() {
                match self.ioc.borrow_mut().dataset_list_next(&name, cookie) {
                    Ok(is) => {
                        list.push(is.name.clone());
                        stack.push((name, is.cookie));
                        stack.push((is.name, 0));
                    }
                    Err(e) => {
                        let ioe = e.downcast::<IOError>()?;
                        ioe.raw_os_error().filter(|n| *n == 3).ok_or(ioe)?; // ESRCH
                    }
                }
            }
        }

        Ok(list)
    }
}

pub struct Root(Rc<Handle>);

pub fn open() -> Result<Root, Box<dyn Error>> {
    Root::open()
}

impl Root {
    fn open() -> Result<Root, Box<dyn Error>> {
        Ok(Root(Rc::new(Handle::open()?)))
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

pub struct Pool {
    handle: Rc<Handle>,
    name: AutoString,
}

impl Pool {
    fn new(handle: Rc<Handle>, name: AutoString) -> Pool {
        Pool { handle, name }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn root_vdev(&self) -> Result<Vdev, Box<dyn Error>> {
        let plist = self.handle.get_pool(&self.name)?;
        let guid = plist
            .get("vdev_tree")
            .and_then(|p| p.as_list())
            .and_then(|l| l.get("guid"))
            .and_then(|p| p.to_u64())
            .ok_or_else(|| IOError::from(IOErrorKind::NotFound))?;
        Ok(Vdev::new(self.handle.clone(), self.name.clone(), guid))
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

pub struct Vdev {
    handle: Rc<Handle>,
    pool: AutoString,
    guid: u64,
}

impl Vdev {
    fn new(handle: Rc<Handle>, pool: AutoString, guid: u64) -> Vdev {
        Vdev { handle, pool, guid }
    }

    pub fn guid(&self) -> u64 {
        self.guid
    }

    pub fn children(&self) -> Result<Vec<Vdev>, Box<dyn Error>> {
        Ok(self
            .handle
            .get_vdev(&self.pool, self.guid)?
            .and_then(|l| {
                l.get("children")
                    .and_then(|p| p.as_list_slice())
                    .map(|s| s.to_vec())
            })
            .unwrap_or(vec![])
            .iter()
            .map(|vl| vl.get("guid").and_then(|p| p.to_u64()))
            .flatten()
            .map(|guid| Vdev::new(self.handle.clone(), self.pool.clone(), guid))
            .collect())
    }

    pub fn stats(&self) -> Result<nvtypes::VdevStats, Box<dyn Error>> {
        Ok(self
            .handle
            .get_vdev(&self.pool, self.guid)?
            .and_then(|l| {
                l.get("vdev_stats")
                    .and_then(|p| p.as_u64_slice())
                    .map(|s| nvtypes::VdevStats::from(s))
            })
            .unwrap_or_default())
    }
}

pub struct Dataset {
    handle: Rc<Handle>,
    name: AutoString,
}

impl Dataset {
    fn new(handle: Rc<Handle>, name: AutoString) -> Dataset {
        Dataset { handle, name }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    fn get_prop(&self, prop: &str) -> Result<Option<PairList>, Box<dyn Error>> {
        let dslist = self.handle.get_dataset(&self.name)?;
        Ok(dslist.get(prop).and_then(|p| p.as_list()).cloned())
    }

    pub fn get_prop_u64(&self, prop: &str) -> Result<Option<u64>, Box<dyn Error>> {
        Ok(self
            .get_prop(prop)?
            .and_then(|l| l.get("value").and_then(|p| p.to_u64())))
    }

    pub fn get_prop_string(&self, prop: &str) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self
            .get_prop(prop)?
            .and_then(|l| l.get("value").and_then(|p| p.to_c_string()))
            .map(|cs| cs.to_string_lossy().to_string()))
    }
}
