// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

// XXX this and other structures like it in fs/zfs.h can be extended with
//     new versions, but not reduced. so we need to initialise to zero, and
//     make sure we don't overrun, but its ok to come up short

// vdev_stat_t
#[repr(C)]
#[derive(Debug, Default)]
pub struct VdevStats {
    pub timestamp: u64, // hrtime_t
    pub state: u64,     // vdev_state_t
    pub aux: u64,       // vdev_aux_t
    pub alloc: u64,
    pub space: u64,
    pub dspace: u64,
    pub rsize: u64,
    pub esize: u64,
    pub ops: [u64; 6],   // VS_ZIO_TYPES
    pub bytes: [u64; 6], // VS_ZIO_TYPES
    pub read_errors: u64,
    pub write_errors: u64,
    pub checksum_errors: u64,
    pub initialize_errors: u64,
    pub self_healed: u64,
    pub scan_removing: u64,
    pub scan_processed: u64,
    pub fragmentation: u64,
    pub initialize_bytes_done: u64,
    pub initialize_bytes_est: u64,
    pub initialize_state: u64,       // vdev_initializing_state_t
    pub initialize_action_time: u64, // time_t
    pub checkpoint_space: u64,
    pub resilver_deferred: u64,
    pub slow_ios: u64,
    pub trim_errors: u64,
    pub trim_notsup: u64,
    pub trim_bytes_done: u64,
    pub trim_bytes_est: u64,
    pub trim_state: u64,       // vdev_trim_state_t
    pub trim_action_time: u64, // time_t
    pub rebuild_processed: u64,
    pub configured_ashift: u64,
    pub logical_ashift: u64,
    pub physical_ashift: u64,
    pub noalloc: u64,
    pub pspace: u64,
}

impl From<&[u64]> for VdevStats {
    fn from(s: &[u64]) -> Self {
        let count = std::cmp::min(
            s.len(),
            std::mem::size_of::<VdevStats>() / std::mem::size_of::<u64>(),
        );
        let mut vs = VdevStats::default();
        unsafe {
            std::ptr::copy_nonoverlapping(
                s.as_ptr(),
                std::ptr::addr_of_mut!(vs) as *mut u64,
                count,
            );
        }
        vs
    }
}
