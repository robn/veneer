// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use veneer::Error;

pub(crate) trait Loader: Sized {
    fn load() -> impl Future<Output = Result<Self, Error>> + Send;
}
