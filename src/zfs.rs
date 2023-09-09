// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crate::ioc;
use crate::nvpair::PairList;
use crate::util::AutoString;
use elsa::FrozenMap;
use std::cell::{OnceCell, RefCell};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::io::Error as IOError;
use std::rc::Rc;

struct Handle {
    ioc: RefCell<ioc::Handle>,
    config: OnceCell<PairList>,
    datasets: FrozenMap<CString, Box<PairList>>,
}

impl Handle {
    fn open() -> Result<Handle, Box<dyn Error>> {
        Ok(Handle {
            ioc: RefCell::new(ioc::Handle::open()?),
            config: OnceCell::default(),
            datasets: FrozenMap::default(),
        })
    }

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

    fn get_dataset(&self, name: &CStr) -> Result<&PairList, Box<dyn Error>> {
        if let Some(p) = self.datasets.get(name) {
            return Ok(p);
        }

        let p = self.ioc.borrow_mut().objset_stats(name)?;
        self.datasets.insert(name.into(), Box::new(p));

        Ok(self.datasets.get(name).unwrap())
    }

    fn get_dataset_list(&self) -> Result<Vec<&CStr>, Box<dyn Error>> {
        let mut list: Vec<&CStr> = vec![];

        for pool in self.get_config()?.keys() {
            let _ = self.get_dataset(pool)?;
            list.push(pool);

            let mut stack: Vec<(CString, u64)> = vec![(pool.into(), 0)];
            while let Some((name, cookie)) = stack.pop() {
                match self.ioc.borrow_mut().dataset_list_next(&name, cookie) {
                    Ok(is) => {
                        stack.push((name, is.cookie));
                        let name = is.name.clone();
                        list.push(match self.datasets.get_key_value(&name) {
                            Some((k, _)) => k,
                            None => {
                                self.datasets.insert(name.clone(), Box::new(is.list));
                                self.datasets.get_key_value(&name).unwrap().0
                            }
                        });
                        stack.push((name, 0));
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
}
