// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2025, Rob Norris <robn@despairlabs.com>

mod handle;
mod sys;

pub use handle::Handle;

error_set::error_set! {
    Error := {
        NVParseError(nvpair::ParseError),
        NameParseError(std::ffi::FromBytesUntilNulError),
        IOError(std::io::Error),
    }
}
