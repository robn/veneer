// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use std::cell::OnceCell;
use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::fmt;

// this badly-named thing takes a CString and does a lossy conversion to String on demand, which is
// what you want almost always
#[derive(Clone)]
pub(crate) struct AutoString(CString, OnceCell<String>);

impl PartialEq for AutoString {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for AutoString {}

impl Ord for AutoString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for AutoString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Default for AutoString {
    fn default() -> Self {
        AutoString(CString::new([]).unwrap(), OnceCell::new())
    }
}

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

impl From<AutoString> for CString {
    fn from(s: AutoString) -> Self {
        s.0.clone()
    }
}