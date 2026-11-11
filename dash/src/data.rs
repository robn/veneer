// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use anystring::AnyString;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, LazyLock, RwLock};
use std::thread;
use std::time::Duration;
use veneer::{Error, VdevState};

#[derive(Clone, Debug, Default)]
pub(crate) struct Data {
    pub pools: BTreeSet<AnyString>,
    pub pool_data: BTreeMap<AnyString, PoolData>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PoolData {
    //pub name: AnyString,
    pub state: VdevState,
    pub size: u64,
    pub alloc: u64,
    pub read: u64,
    pub write: u64,
    pub _wat: String,
}

static DATA: LazyLock<Arc<RwLock<Data>>> = LazyLock::new(|| Arc::new(RwLock::new(Data::default())));

fn fetch() -> Result<Data, Error> {
    let z = veneer::open()?;

    #[allow(clippy::mutable_key_type)]
    let mut pools = BTreeSet::new();

    #[allow(clippy::mutable_key_type)]
    let mut pool_data = BTreeMap::new();

    for pool in z.pools()? {
        let root = pool.root_vdev()?;
        let stats = root.stats()?;

        pools.insert(pool.name().into());
        pool_data.insert(
            pool.name().into(),
            PoolData {
                //name: pool.name().into(),
                state: stats.state,
                size: stats.space,
                alloc: stats.alloc,
                read: stats.bytes[1],
                write: stats.bytes[2],
                _wat: format!("{:?}", stats),
            },
        );
    }

    Ok(Data { pools, pool_data })
}

pub(crate) fn get() -> Data {
    DATA.read().unwrap().clone()
}

pub(crate) fn start() {
    let lock = Arc::clone(&DATA);
    thread::spawn(move || loop {
        let data = fetch().unwrap();
        *lock.write().unwrap() = data;
        thread::sleep(Duration::from_secs(1));
    });
}
