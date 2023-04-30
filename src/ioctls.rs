use std::os::raw::c_int;
use std::ptr::null;
use derivative::Derivative;
use iocuddle::{Ioctl, WriteRead};

// include/sys/fs/zfs.h
const ZFS_MAX_DATASET_NAME_LEN: usize = 256;

// include/os/linux/spl/sys/sysmacros.h
const MAXNAMELEN: usize = 256;
const MAXPATHLEN: usize = 4096;

// dmu_object_stats_t
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub struct DMUObjectStats {
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
pub struct DMUReplayRecordBegin {
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
pub struct ZInjectRecord {
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
pub struct Share {
    exportdata: u64,
    sharedata:  u64,
    sharetype:  u64,
    sharemax:   u64,
}

// zfs_stat_t
#[repr(C)]
#[derive(Derivative,Debug)]
#[derivative(Default)]
pub struct Stat {
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
pub struct Command {
    // nvlist-based
    #[derivative(Default(value="[0; MAXPATHLEN]"))]
    pub name:               [u8; MAXPATHLEN],
    #[derivative(Default(value="null()"))]
    nvlist_src:         *const u8,
    nvlist_src_size:    u64,
    #[derivative(Default(value="null()"))]
    pub nvlist_dst:         *const u8,
    pub nvlist_dst_size:    u64,

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
    cookie:             u64,
    objset_type:        u64,
    perm_action:        u64,
    history_len:        u64,
    history_offset:     u64,
    obj:                u64,
    iflags:             u64,
    share:              Share,
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
    stat:               Stat,
    zoneid:             u64,
}

macro_rules! zfs_ioctl {
    ($name:ident, $id:expr) => {
        pub const $name: Ioctl<WriteRead, &Command> =
            unsafe { Ioctl::classic($id) };
    }
}

zfs_ioctl!(ZFS_IOC_POOL_CONFIGS, 0x5a04);
zfs_ioctl!(ZFS_IOC_POOL_STATS,   0x5a05);
