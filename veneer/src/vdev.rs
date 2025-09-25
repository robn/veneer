// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crate::nvenums::VdevType;
use crate::nvtypes;
use crate::util::AutoString;
use crate::Handle;
use nvpair::PairList;
use std::error::Error;
use std::rc::Rc;

pub struct Vdev {
    handle: Rc<Handle>,
    pool: AutoString,
    id: u64,
    guid: u64,
    typ: VdevType,

    // XXX below are likely parameterised variants, but I want to see what they look like first
    parity: Option<usize>,
    path: Option<AutoString>,
}

impl Vdev {
    pub(crate) fn new(
        handle: Rc<Handle>,
        pool: AutoString,
        vl: &PairList,
    ) -> Result<Vdev, Box<dyn Error>> {
        let id = vl.get_u64("id").unwrap_or(0);
        let guid = vl.get_u64("guid").unwrap_or(0);

        let typ = vl
            .get_c_string("type")
            .map(|ref s| s.into())
            .unwrap_or(VdevType::Unknown);

        let parity = match typ {
            VdevType::Raidz => vl.get_u64("nparity").map(|n| n as usize),
            _ => None,
        };

        let path = match typ {
            VdevType::Disk | VdevType::File => vl.get_c_string("path").map(|ref cs| cs.into()),
            _ => None,
        };

        Ok(Vdev {
            handle,
            pool,
            id,
            guid,
            typ,
            parity,
            path,
        })
    }

    pub fn guid(&self) -> u64 {
        self.guid
    }

    pub fn typ(&self) -> VdevType {
        self.typ
    }

    pub fn name(&self) -> String {
        match self.typ {
            VdevType::Root => self.pool.to_string(),
            VdevType::Mirror => format!("mirror-{}", self.id),
            //VdevType::Replacing,
            VdevType::Raidz => format!("raidz{}-{}", self.id, self.parity.unwrap()),
            //VdevType::Draid,
            //VdevType::DraidSpare,
            VdevType::Disk => self.path.as_ref().unwrap().to_string(),
            VdevType::File => self.path.as_ref().unwrap().to_string(),
            //VdevType::Missing,
            //VdevType::Hole,
            //VdevType::Spare,
            //VdevType::Log,
            //VdevType::L2cache,
            //VdevType::Indirect,
            //VdevType::Unknown,
            _ => self.guid.to_string(),
        }
    }

    pub fn children(&self) -> Result<Vec<Vdev>, Box<dyn Error>> {
        Ok(self
            .handle
            .get_vdev(&self.pool, self.guid)?
            .and_then(|l| l.get_list_slice("children").map(|s| s.to_vec()))
            .unwrap_or(vec![])
            .iter()
            .map(|vl| Vdev::new(self.handle.clone(), self.pool.clone(), vl))
            .flatten()
            .collect())
    }

    pub fn stats(&self) -> Result<nvtypes::VdevStats, Box<dyn Error>> {
        Ok(self
            .handle
            .get_vdev(&self.pool, self.guid)?
            .and_then(|l| {
                l.get_u64_slice("vdev_stats")
                    .map(|s| nvtypes::VdevStats::from(s))
            })
            .unwrap_or_default())
    }
}
