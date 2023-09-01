// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::fs::File;
use std::path::Path;
use std::io::Result as IOResult;
use std::error::Error;
use std::ffi::{CStr, CString};
use crate::sys::{self, ZFSCommand, ZFSIoctl};
use crate::nvpair::{self, List as NVList};

#[derive(Debug)]
pub struct Handle {
    dev: File,
    cmd: ZFSCommand,
    buf: [u8; 262144],
}

#[derive(Debug)]
pub struct IterState {
    pub name: CString,
    pub nvlist: NVList,
    pub cookie: u64,
}

type IOCResult = Result<(), Box<dyn Error>>;
type IOCResultNV = Result<NVList, Box<dyn Error>>;
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
    fn invoke(&mut self, ioctl: ZFSIoctl) -> IOCResult {
        ioctl.ioctl(&mut self.dev, &mut self.cmd)?;
        Ok(())
    }

    // helper: invoke, explode the result nvlist and return it
    fn invoke_nv(&mut self, ioctl: ZFSIoctl) -> IOCResultNV {
        self.invoke(ioctl)?;
        let nvbuf = &self.buf[0..self.cmd.nvlist_dst_size as usize];
        Ok(nvpair::parse(nvbuf)?)
    }

    // helper: reset, setup named object, invoke, return nvlist
    fn ioc_name_nv(&mut self, ioctl: ZFSIoctl, cname: &CStr) -> IOCResultNV {
        self.reset();
        let name = cname.to_bytes_with_nul();
        self.cmd.name[..name.len()].copy_from_slice(&name);
        self.invoke_nv(ioctl)
    }

    // helper: reset, setup named object+cookie, invoke, return name+nvlist+cookie
    fn ioc_name_nv_cookie(&mut self, ioctl: ZFSIoctl, cname: &CStr, cookie: u64) -> IOCResultIter {
        self.reset();
        let name = cname.to_bytes_with_nul();
        self.cmd.name[..name.len()].copy_from_slice(&name);
        self.cmd.cookie = cookie;
        let nvlist = self.invoke_nv(ioctl)?;
        Ok(IterState {
            name: CStr::from_bytes_until_nul(&self.cmd.name)?.into(),
            nvlist,
            cookie: self.cmd.cookie,
        })
    }


    // global ioctls

    // get top-level config for all pools (like label contents or zpool.cache)
    pub fn pool_configs(&mut self) -> IOCResultNV {
        self.reset();
        self.invoke_nv(sys::ZFS_IOC_POOL_CONFIGS)
    }


    // per-pool ioctls

    // get pool stats (iostat counters, config, features, real mixed bag)
    pub fn pool_stats(&mut self, pool: &CStr) -> IOCResultNV {
        self.ioc_name_nv(sys::ZFS_IOC_POOL_STATS, pool)
    }

    // get pool properties (like zpool get)
    pub fn pool_get_props(&mut self, pool: &CStr) -> IOCResultNV {
        self.ioc_name_nv(sys::ZFS_IOC_POOL_GET_PROPS, pool)
    }


    // per-dataset ioctls

    // get dataset properties (like zfs get)
    pub fn objset_stats(&mut self, objset: &CStr) -> IOCResultNV {
        self.ioc_name_nv(sys::ZFS_IOC_OBJSET_STATS, objset)
    }


    // dataset iterator ioctls
    pub fn dataset_list_next(&mut self, dataset: &CStr, cookie: u64) -> IOCResultIter {
        self.ioc_name_nv_cookie(sys::ZFS_IOC_DATASET_LIST_NEXT, dataset, cookie)
    }
}
