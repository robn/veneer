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

macro_rules! zfs_ioctl {
    ($name:ident, $id:expr) => {
	#[allow(unused)]
        pub(crate) const $name: Ioctl<WriteRead, &ZFSCommand> =
            unsafe { Ioctl::classic($id) };
    }
}

zfs_ioctl!(ZFS_IOC_POOL_CREATE,             0x5a00);
zfs_ioctl!(ZFS_IOC_POOL_DESTROY,            0x5a01);
zfs_ioctl!(ZFS_IOC_POOL_IMPORT,             0x5a02);
zfs_ioctl!(ZFS_IOC_POOL_EXPORT,             0x5a03);
zfs_ioctl!(ZFS_IOC_POOL_CONFIGS,            0x5a04);
zfs_ioctl!(ZFS_IOC_POOL_STATS,              0x5a05);
zfs_ioctl!(ZFS_IOC_POOL_TRYIMPORT,          0x5a06);
zfs_ioctl!(ZFS_IOC_POOL_SCAN,               0x5a07);
zfs_ioctl!(ZFS_IOC_POOL_FREEZE,             0x5a08);
zfs_ioctl!(ZFS_IOC_POOL_UPGRADE,            0x5a09);
zfs_ioctl!(ZFS_IOC_POOL_GET_HISTORY,        0x5a0a);
zfs_ioctl!(ZFS_IOC_VDEV_ADD,                0x5a0b);
zfs_ioctl!(ZFS_IOC_VDEV_REMOVE,             0x5a0c);
zfs_ioctl!(ZFS_IOC_VDEV_SET_STATE,          0x5a0d);
zfs_ioctl!(ZFS_IOC_VDEV_ATTACH,             0x5a0e);
zfs_ioctl!(ZFS_IOC_VDEV_DETACH,             0x5a0f);
zfs_ioctl!(ZFS_IOC_VDEV_SETPATH,            0x5a10);
zfs_ioctl!(ZFS_IOC_VDEV_SETFRU,             0x5a11);
zfs_ioctl!(ZFS_IOC_OBJSET_STATS,            0x5a12);
zfs_ioctl!(ZFS_IOC_OBJSET_ZPLPROPS,         0x5a13);
zfs_ioctl!(ZFS_IOC_DATASET_LIST_NEXT,       0x5a14);
zfs_ioctl!(ZFS_IOC_SNAPSHOT_LIST_NEXT,      0x5a15);
zfs_ioctl!(ZFS_IOC_SET_PROP,                0x5a16);
zfs_ioctl!(ZFS_IOC_CREATE,                  0x5a17);
zfs_ioctl!(ZFS_IOC_DESTROY,                 0x5a18);
zfs_ioctl!(ZFS_IOC_ROLLBACK,                0x5a19);
zfs_ioctl!(ZFS_IOC_RENAME,                  0x5a1a);
zfs_ioctl!(ZFS_IOC_RECV,                    0x5a1b);
zfs_ioctl!(ZFS_IOC_SEND,                    0x5a1c);
zfs_ioctl!(ZFS_IOC_INJECT_FAULT,            0x5a1d);
zfs_ioctl!(ZFS_IOC_CLEAR_FAULT,             0x5a1e);
zfs_ioctl!(ZFS_IOC_INJECT_LIST_NEXT,        0x5a1f);
zfs_ioctl!(ZFS_IOC_ERROR_LOG,               0x5a20);
zfs_ioctl!(ZFS_IOC_CLEAR,                   0x5a21);
zfs_ioctl!(ZFS_IOC_PROMOTE,                 0x5a22);
zfs_ioctl!(ZFS_IOC_SNAPSHOT,                0x5a23);
zfs_ioctl!(ZFS_IOC_DSOBJ_TO_DSNAME,         0x5a24);
zfs_ioctl!(ZFS_IOC_OBJ_TO_PATH,             0x5a25);
zfs_ioctl!(ZFS_IOC_POOL_SET_PROPS,          0x5a26);
zfs_ioctl!(ZFS_IOC_POOL_GET_PROPS,          0x5a27);
zfs_ioctl!(ZFS_IOC_SET_FSACL,               0x5a28);
zfs_ioctl!(ZFS_IOC_GET_FSACL,               0x5a29);
zfs_ioctl!(ZFS_IOC_SHARE,                   0x5a2a);
zfs_ioctl!(ZFS_IOC_INHERIT_PROP,            0x5a2b);
zfs_ioctl!(ZFS_IOC_SMB_ACL,                 0x5a2c);
zfs_ioctl!(ZFS_IOC_USERSPACE_ONE,           0x5a2d);
zfs_ioctl!(ZFS_IOC_USERSPACE_MANY,          0x5a2e);
zfs_ioctl!(ZFS_IOC_USERSPACE_UPGRADE,       0x5a2f);
zfs_ioctl!(ZFS_IOC_HOLD,                    0x5a30);
zfs_ioctl!(ZFS_IOC_RELEASE,                 0x5a31);
zfs_ioctl!(ZFS_IOC_GET_HOLDS,               0x5a32);
zfs_ioctl!(ZFS_IOC_OBJSET_RECVD_PROPS,      0x5a33);
zfs_ioctl!(ZFS_IOC_VDEV_SPLIT,              0x5a34);
zfs_ioctl!(ZFS_IOC_NEXT_OBJ,                0x5a35);
zfs_ioctl!(ZFS_IOC_DIFF,                    0x5a36);
zfs_ioctl!(ZFS_IOC_TMP_SNAPSHOT,            0x5a37);
zfs_ioctl!(ZFS_IOC_OBJ_TO_STATS,            0x5a38);
zfs_ioctl!(ZFS_IOC_SPACE_WRITTEN,           0x5a39);
zfs_ioctl!(ZFS_IOC_SPACE_SNAPS,             0x5a3a);
zfs_ioctl!(ZFS_IOC_DESTROY_SNAPS,           0x5a3b);
zfs_ioctl!(ZFS_IOC_POOL_REGUID,             0x5a3c);
zfs_ioctl!(ZFS_IOC_POOL_REOPEN,             0x5a3d);
zfs_ioctl!(ZFS_IOC_SEND_PROGRESS,           0x5a3e);
zfs_ioctl!(ZFS_IOC_LOG_HISTORY,             0x5a3f);
zfs_ioctl!(ZFS_IOC_SEND_NEW,                0x5a40);
zfs_ioctl!(ZFS_IOC_SEND_SPACE,              0x5a41);
zfs_ioctl!(ZFS_IOC_CLONE,                   0x5a42);
zfs_ioctl!(ZFS_IOC_BOOKMARK,                0x5a43);
zfs_ioctl!(ZFS_IOC_GET_BOOKMARKS,           0x5a44);
zfs_ioctl!(ZFS_IOC_DESTROY_BOOKMARKS,       0x5a45);
zfs_ioctl!(ZFS_IOC_RECV_NEW,                0x5a46);
zfs_ioctl!(ZFS_IOC_POOL_SYNC,               0x5a47);
zfs_ioctl!(ZFS_IOC_CHANNEL_PROGRAM,         0x5a48);
zfs_ioctl!(ZFS_IOC_LOAD_KEY,                0x5a49);
zfs_ioctl!(ZFS_IOC_UNLOAD_KEY,              0x5a4a);
zfs_ioctl!(ZFS_IOC_CHANGE_KEY,              0x5a4b);
zfs_ioctl!(ZFS_IOC_REMAP,                   0x5a4c);
zfs_ioctl!(ZFS_IOC_POOL_CHECKPOINT,         0x5a4d);
zfs_ioctl!(ZFS_IOC_POOL_DISCARD_CHECKPOINT, 0x5a4e);
zfs_ioctl!(ZFS_IOC_POOL_INITIALIZE,         0x5a4f);
zfs_ioctl!(ZFS_IOC_POOL_TRIM,               0x5a50);
zfs_ioctl!(ZFS_IOC_REDACT,                  0x5a51);
zfs_ioctl!(ZFS_IOC_GET_BOOKMARK_PROPS,      0x5a52);
zfs_ioctl!(ZFS_IOC_WAIT,                    0x5a53);
zfs_ioctl!(ZFS_IOC_WAIT_FS,                 0x5a54);
