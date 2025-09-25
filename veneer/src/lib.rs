// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

mod nvenums;
mod nvtypes;
mod util;

mod dataset;
mod handle;
mod host;
mod pool;
mod vdev;

pub use dataset::Dataset;
pub use handle::Handle;
pub use host::Host;
pub use pool::Pool;
pub use vdev::Vdev;

use std::error::Error;

pub fn open() -> Result<Host, Box<dyn Error>> {
    Host::open()
}
