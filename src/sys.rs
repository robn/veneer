// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::ptr::null;
use derivative::Derivative;
use std::io::Error as IOError;
use std::os::fd::AsRawFd;
use std::os::raw::{c_ulong, c_int, c_uint, c_void};

// include/sys/fs/zfs.h
const ZFS_MAX_DATASET_NAME_LEN: usize = 256;

// include/os/linux/spl/sys/sysmacros.h
#[cfg(target_os="linux")] const MAXNAMELEN: usize = 256;
#[cfg(target_os="linux")] const MAXPATHLEN: usize = 4096;

// /usr/include/sys/syslimits.h
// /usr/include/sys/param.h
#[cfg(target_os="freebsd")] const MAXNAMELEN: usize = 256;
#[cfg(target_os="freebsd")] const MAXPATHLEN: usize = 1024; // PATH_MAX

// dmu_object_stats_t
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub(crate) struct DMUObjectStats {
    num_clones:     u64,
    creation_txg:   u64,
    guid:           u64,
    typ:            c_int, // enum dmu_objset_type
    is_snapshot:    u8,
    inconsistent:   u8,
    redacted:       u8,
    #[derivative(Default(value="[0; ZFS_MAX_DATASET_NAME_LEN]"))]
    origin:         [u8; ZFS_MAX_DATASET_NAME_LEN],
}

// struct drr_begin
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub(crate) struct DMUReplayRecordBegin {
    magic:          u64,
    versioninfo:    u64,
    creation_time:  u64,
    typ:            c_int, // enum dmu_objset_type
    flags:          u32,
    toguid:         u64,
    fromguid:       u64,
    #[derivative(Default(value="[0; MAXNAMELEN]"))]
    toname:         [u8; MAXNAMELEN],
}

// zinject_record_t
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub(crate) struct ZInjectRecord {
    objset:     u64,
    object:     u64,
    start:      u64,
    end:        u64,
    guid:       u64,
    level:      u32,
    error:      u32,
    typ:        u64,
    freq:       u32,
    failfast:   u32,
    #[derivative(Default(value="[0; MAXNAMELEN]"))]
    func:       [u8; MAXNAMELEN],
    iotype:     u32,
    duration:   i32,
    timer:      u64,
    nlanes:     u64,
    cmd:        u64,
    dvas:       u64,
}

// zfs_share_t
#[repr(C)]
#[derive(Default,Debug)]
pub(crate) struct ZFSShare {
    exportdata: u64,
    sharedata:  u64,
    sharetype:  u64,
    sharemax:   u64,
}

// zfs_stat_t
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub(crate) struct ZFSStat {
    gen:    u64,
    mode:   u64,
    links:  u64,
    #[derivative(Default(value="[0; 2]"))]
    ctime:  [u64; 2],
}

// zfs_cmd_t
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub(crate) struct ZFSCommand {
    // nvlist-based
    #[derivative(Default(value="[0; MAXPATHLEN]"))]
    pub name:               [u8; MAXPATHLEN],
    #[derivative(Default(value="null()"))]
    nvlist_src:         *const u8,
    nvlist_src_size:    u64,
    #[derivative(Default(value="null()"))]
    pub nvlist_dst:         *const u8,
    pub nvlist_dst_size:    u64,
    pad2:               i32,

    // legacy
    #[derivative(Default(value="null()"))]
    history:            *const u8,
    #[derivative(Default(value="[0; MAXPATHLEN*2]"))]
    value:              [u8; MAXPATHLEN*2],
    #[derivative(Default(value="[0; MAXNAMELEN]"))]
    string:             [u8; MAXNAMELEN],
    guid:               u64,
    #[derivative(Default(value="null()"))]
    nvlist_conf:        *const u8,
    nvlist_conf_size:   u64,
    pub cookie:             u64,
    objset_type:        u64,
    perm_action:        u64,
    history_len:        u64,
    history_offset:     u64,
    obj:                u64,
    iflags:             u64,
    share:              ZFSShare,
    objset_stats:       DMUObjectStats,
    begin_record:       DMUReplayRecordBegin,
    inject_record:      ZInjectRecord,
    defer_destroy:      u32,
    flags:              i32,
    action_handle:      u64,
    cleanup_fd:         c_int,
    simple:             u8,
    #[derivative(Default(value="[0; 3]"))]
    pad:                [u8; 3],
    sendobj:            u64,
    fromobj:            u64,
    createtxg:          u64,
    stat:               ZFSStat,
    zoneid:             u64,
}

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

#[cfg(not(target_os="freebsd"))]
pub(crate)
fn zfs_ioctl(fd: &mut impl AsRawFd, req: c_ulong, zc: &mut ZFSCommand) -> Result<c_uint, IOError> {
    let r = unsafe { ioctl(fd.as_raw_fd(), 0x5a00+req, zc as *mut _, null::<c_void>()) };
    r.try_into().map_err(|_| IOError::last_os_error())
}

#[cfg(target_os="freebsd")]
pub(crate)
fn zfs_ioctl(fd: &mut impl AsRawFd, req: c_ulong, zc: &mut ZFSCommand) -> Result<c_uint, IOError> {
    #[repr(C)]
    struct iocparm(u32, u64, u64);
    let mut iocp = iocparm(15 as u32, zc as *mut ZFSCommand as u64, std::mem::size_of::<ZFSCommand>() as u64);
    // _IOWR('Z', req, sizeof(iocparm))
    let ncmd: c_ulong = 0xc0000000 + ((std::mem::size_of::<iocparm>() as c_ulong) << 16) + 0x5a00 + req;
    let r = unsafe { ioctl(fd.as_raw_fd(), ncmd, &mut iocp as *mut _, null::<c_void>()) };
    r.try_into().map_err(|_| IOError::last_os_error())
}

macro_rules! ioc {
    ($name:ident, $id:expr) => {
        #[allow(unused)]
        pub(crate) const $name: c_ulong = $id;
    }
}

ioc!(ZFS_IOC_POOL_CREATE,             0x00);
ioc!(ZFS_IOC_POOL_DESTROY,            0x01);
ioc!(ZFS_IOC_POOL_IMPORT,             0x02);
ioc!(ZFS_IOC_POOL_EXPORT,             0x03);
ioc!(ZFS_IOC_POOL_CONFIGS,            0x04);
ioc!(ZFS_IOC_POOL_STATS,              0x05);
ioc!(ZFS_IOC_POOL_TRYIMPORT,          0x06);
ioc!(ZFS_IOC_POOL_SCAN,               0x07);
ioc!(ZFS_IOC_POOL_FREEZE,             0x08);
ioc!(ZFS_IOC_POOL_UPGRADE,            0x09);
ioc!(ZFS_IOC_POOL_GET_HISTORY,        0x0a);
ioc!(ZFS_IOC_VDEV_ADD,                0x0b);
ioc!(ZFS_IOC_VDEV_REMOVE,             0x0c);
ioc!(ZFS_IOC_VDEV_SET_STATE,          0x0d);
ioc!(ZFS_IOC_VDEV_ATTACH,             0x0e);
ioc!(ZFS_IOC_VDEV_DETACH,             0x0f);
ioc!(ZFS_IOC_VDEV_SETPATH,            0x10);
ioc!(ZFS_IOC_VDEV_SETFRU,             0x11);
ioc!(ZFS_IOC_OBJSET_STATS,            0x12);
ioc!(ZFS_IOC_OBJSET_ZPLPROPS,         0x13);
ioc!(ZFS_IOC_DATASET_LIST_NEXT,       0x14);
ioc!(ZFS_IOC_SNAPSHOT_LIST_NEXT,      0x15);
ioc!(ZFS_IOC_SET_PROP,                0x16);
ioc!(ZFS_IOC_CREATE,                  0x17);
ioc!(ZFS_IOC_DESTROY,                 0x18);
ioc!(ZFS_IOC_ROLLBACK,                0x19);
ioc!(ZFS_IOC_RENAME,                  0x1a);
ioc!(ZFS_IOC_RECV,                    0x1b);
ioc!(ZFS_IOC_SEND,                    0x1c);
ioc!(ZFS_IOC_INJECT_FAULT,            0x1d);
ioc!(ZFS_IOC_CLEAR_FAULT,             0x1e);
ioc!(ZFS_IOC_INJECT_LIST_NEXT,        0x1f);
ioc!(ZFS_IOC_ERROR_LOG,               0x20);
ioc!(ZFS_IOC_CLEAR,                   0x21);
ioc!(ZFS_IOC_PROMOTE,                 0x22);
ioc!(ZFS_IOC_SNAPSHOT,                0x23);
ioc!(ZFS_IOC_DSOBJ_TO_DSNAME,         0x24);
ioc!(ZFS_IOC_OBJ_TO_PATH,             0x25);
ioc!(ZFS_IOC_POOL_SET_PROPS,          0x26);
ioc!(ZFS_IOC_POOL_GET_PROPS,          0x27);
ioc!(ZFS_IOC_SET_FSACL,               0x28);
ioc!(ZFS_IOC_GET_FSACL,               0x29);
ioc!(ZFS_IOC_SHARE,                   0x2a);
ioc!(ZFS_IOC_INHERIT_PROP,            0x2b);
ioc!(ZFS_IOC_SMB_ACL,                 0x2c);
ioc!(ZFS_IOC_USERSPACE_ONE,           0x2d);
ioc!(ZFS_IOC_USERSPACE_MANY,          0x2e);
ioc!(ZFS_IOC_USERSPACE_UPGRADE,       0x2f);
ioc!(ZFS_IOC_HOLD,                    0x30);
ioc!(ZFS_IOC_RELEASE,                 0x31);
ioc!(ZFS_IOC_GET_HOLDS,               0x32);
ioc!(ZFS_IOC_OBJSET_RECVD_PROPS,      0x33);
ioc!(ZFS_IOC_VDEV_SPLIT,              0x34);
ioc!(ZFS_IOC_NEXT_OBJ,                0x35);
ioc!(ZFS_IOC_DIFF,                    0x36);
ioc!(ZFS_IOC_TMP_SNAPSHOT,            0x37);
ioc!(ZFS_IOC_OBJ_TO_STATS,            0x38);
ioc!(ZFS_IOC_SPACE_WRITTEN,           0x39);
ioc!(ZFS_IOC_SPACE_SNAPS,             0x3a);
ioc!(ZFS_IOC_DESTROY_SNAPS,           0x3b);
ioc!(ZFS_IOC_POOL_REGUID,             0x3c);
ioc!(ZFS_IOC_POOL_REOPEN,             0x3d);
ioc!(ZFS_IOC_SEND_PROGRESS,           0x3e);
ioc!(ZFS_IOC_LOG_HISTORY,             0x3f);
ioc!(ZFS_IOC_SEND_NEW,                0x40);
ioc!(ZFS_IOC_SEND_SPACE,              0x41);
ioc!(ZFS_IOC_CLONE,                   0x42);
ioc!(ZFS_IOC_BOOKMARK,                0x43);
ioc!(ZFS_IOC_GET_BOOKMARKS,           0x44);
ioc!(ZFS_IOC_DESTROY_BOOKMARKS,       0x45);
ioc!(ZFS_IOC_RECV_NEW,                0x46);
ioc!(ZFS_IOC_POOL_SYNC,               0x47);
ioc!(ZFS_IOC_CHANNEL_PROGRAM,         0x48);
ioc!(ZFS_IOC_LOAD_KEY,                0x49);
ioc!(ZFS_IOC_UNLOAD_KEY,              0x4a);
ioc!(ZFS_IOC_CHANGE_KEY,              0x4b);
ioc!(ZFS_IOC_REMAP,                   0x4c);
ioc!(ZFS_IOC_POOL_CHECKPOINT,         0x4d);
ioc!(ZFS_IOC_POOL_DISCARD_CHECKPOINT, 0x4e);
ioc!(ZFS_IOC_POOL_INITIALIZE,         0x4f);
ioc!(ZFS_IOC_POOL_TRIM,               0x50);
ioc!(ZFS_IOC_REDACT,                  0x51);
ioc!(ZFS_IOC_GET_BOOKMARK_PROPS,      0x52);
ioc!(ZFS_IOC_WAIT,                    0x53);
ioc!(ZFS_IOC_WAIT_FS,                 0x54);
