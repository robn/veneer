// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use veneer::VdevState;

#[derive(Clone, Debug, Default)]
pub(crate) struct PoolData {
    pub name: String,
    pub state: VdevState,
    pub size: u64,
    pub alloc: u64,
    pub read: u64,
    pub write: u64,
    pub _wat: String,
}
