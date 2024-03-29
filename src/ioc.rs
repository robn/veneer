// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crate::nvpair::{self, PairList};
use crate::sys::{self, ZFSCommand};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Result as IOResult;
use std::os::raw::c_ulong;
use std::path::Path;

#[derive(Debug)]
pub struct Handle {
    dev: File,
    cmd: ZFSCommand,
    buf: [u8; 262144],
}

#[derive(Debug)]
pub struct IterState {
    pub name: CString,
    pub list: PairList,
    pub cookie: u64,
}

type IOCResult = Result<(), Box<dyn Error>>;
type IOCResultList = Result<PairList, Box<dyn Error>>;
type IOCResultIter = Result<IterState, Box<dyn Error>>;

impl Handle {
    // open the control device node. you only need this if its not on /dev/zfs
    pub fn open_dev<P: AsRef<Path>>(path: P) -> IOResult<Handle> {
        Ok(Handle {
            dev: File::open(path)?,
            cmd: Default::default(),
            buf: [0; 262144],
        })
    }

    // open the control device via /dev/zfs
    pub fn open() -> IOResult<Handle> {
        Handle::open_dev("/dev/zfs")
    }

    // most of the zfs ioctls have a common form: fill out a couple of details
    // inside the (enormous, mostly obsolete) command structure, submit it,
    // then explode the returned nvlist. this is nice for us, as we can
    // implement a good chunk of them with some simple helpers

    // helper: reset the handle state ready for the next command
    fn reset(&mut self) {
        self.cmd = Default::default();
        self.cmd.nvlist_dst = self.buf.as_ptr();
        self.cmd.nvlist_dst_size = self.buf.len() as u64;
    }

    // helper: invoke the command
    fn invoke(&mut self, req: c_ulong) -> IOCResult {
        sys::zfs_ioctl(&mut self.dev, req, &mut self.cmd)?;
        Ok(())
    }

    // helper: invoke, explode the result list and return it
    fn invoke_list(&mut self, req: c_ulong) -> IOCResultList {
        self.invoke(req)?;
        let nvbuf = &self.buf[0..self.cmd.nvlist_dst_size as usize];
        Ok(nvpair::parse(nvbuf)?)
    }

    // helper: reset, setup named object, invoke, return nvlist
    fn ioc_name_list(&mut self, req: c_ulong, cname: &CStr) -> IOCResultList {
        self.reset();
        let name = cname.to_bytes_with_nul();
        self.cmd.name[..name.len()].copy_from_slice(&name);
        self.invoke_list(req)
    }

    // helper: reset, setup named object+cookie, invoke, return name+nvlist+cookie
    fn ioc_name_list_cookie(&mut self, req: c_ulong, cname: &CStr, cookie: u64) -> IOCResultIter {
        self.reset();
        let name = cname.to_bytes_with_nul();
        self.cmd.name[..name.len()].copy_from_slice(&name);
        self.cmd.cookie = cookie;
        let list = self.invoke_list(req)?;
        Ok(IterState {
            name: CStr::from_bytes_until_nul(&self.cmd.name)?.into(),
            list,
            cookie: self.cmd.cookie,
        })
    }

    // global ioctls

    // get top-level config for all pools (like label contents or zpool.cache)
    pub fn pool_configs(&mut self) -> IOCResultList {
        self.reset();
        self.invoke_list(sys::ZFS_IOC_POOL_CONFIGS)
    }

    // per-pool ioctls

    // get pool stats (iostat counters, config, features, real mixed bag)
    pub fn pool_stats(&mut self, pool: &CStr) -> IOCResultList {
        self.ioc_name_list(sys::ZFS_IOC_POOL_STATS, pool)
    }

    // get pool properties (like zpool get)
    pub fn pool_get_props(&mut self, pool: &CStr) -> IOCResultList {
        self.ioc_name_list(sys::ZFS_IOC_POOL_GET_PROPS, pool)
    }

    // per-dataset ioctls

    // get dataset properties (like zfs get)
    pub fn objset_stats(&mut self, objset: &CStr) -> IOCResultList {
        self.ioc_name_list(sys::ZFS_IOC_OBJSET_STATS, objset)
    }

    // dataset iterator ioctls
    pub fn dataset_list_next(&mut self, dataset: &CStr, cookie: u64) -> IOCResultIter {
        self.ioc_name_list_cookie(sys::ZFS_IOC_DATASET_LIST_NEXT, dataset, cookie)
    }
}
