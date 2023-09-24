// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::ffi::CStr;

#[derive(Debug, Clone, Copy)]
pub enum VdevType {
    Root,
    Mirror,
    Replacing,
    Raidz,
    Draid,
    DraidSpare,
    Disk,
    File,
    Missing,
    Hole,
    Spare,
    Log,
    L2cache,
    Indirect,
    Unknown,
}

impl<T: ?Sized + AsRef<CStr>> From<&T> for VdevType {
    fn from(s: &T) -> Self {
        match s.as_ref().to_string_lossy().as_ref() {
            "root" => VdevType::Root,
            "mirror" => VdevType::Mirror,
            "replacing" => VdevType::Replacing,
            "raidz" => VdevType::Raidz,
            "draid" => VdevType::Draid,
            "dspare" => VdevType::DraidSpare,
            "disk" => VdevType::Disk,
            "file" => VdevType::File,
            "missing" => VdevType::Missing,
            "hole" => VdevType::Hole,
            "spare" => VdevType::Spare,
            "log" => VdevType::Log,
            "l2cache" => VdevType::L2cache,
            "indirect" => VdevType::Indirect,
            _ => VdevType::Unknown,
        }
    }
}
