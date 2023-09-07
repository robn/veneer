// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::cell::OnceCell;
use std::ffi::{CStr, CString};
use std::fmt;

// this badly-named thing takes a CString and does a lossy conversion to String on demand, which is
// what you want almost always
pub(crate) struct AutoString(CString, OnceCell<String>);

impl fmt::Display for AutoString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.1.get_or_init(|| self.0.to_string_lossy().to_string())
        )
    }
}

impl From<CString> for AutoString {
    fn from(s: CString) -> Self {
        AutoString(s, OnceCell::new())
    }
}

impl From<&CStr> for AutoString {
    fn from(s: &CStr) -> Self {
        AutoString(s.into(), OnceCell::new())
    }
}

impl From<AutoString> for String {
    fn from(s: AutoString) -> Self {
        s.to_string()
    }
}
