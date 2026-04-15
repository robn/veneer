// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use std::sync::OnceLock;
use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::fmt;

// AnyString is a thing that will take a String or a CString and do a lossy conversion to the other
// kind on demand, which is almost always what you want for messing with C stuff from Rust.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Canonical {
    #[default]
    String,
    CString,
}

#[derive(Clone, Debug, Default)]
pub struct AnyString {
    canonical: Canonical,
    s: OnceLock<String>,
    c: OnceLock<CString>,
}

impl AnyString {
    // create from a String
    pub fn from_string(s: String) -> AnyString {
        AnyString {
            canonical: Canonical::String,
            s: s.into(),
            c: OnceLock::new(),
        }
    }

    // create from a CString
    pub fn from_c_string(c: CString) -> AnyString {
        AnyString {
            canonical: Canonical::CString,
            s: OnceLock::new(),
            c: c.into(),
        }
    }

    // create from references to same
    pub fn from_str(s: &str) -> AnyString {
        Self::from_string(s.into())
    }
    pub fn from_c_str(c: &CStr) -> AnyString {
        Self::from_c_string(c.into())
    }

    // get a ref to the stored string, if it exists
    pub fn get_str_inner(&self) -> Option<&str> {
        self.s.get().map(|s| s.as_str())
    }

    // get a ref to the stored C string, if it exists
    pub fn get_c_str_inner(&self) -> Option<&CStr> {
        self.c.get().map(|c| c.as_c_str())
    }

    // convert the stored string to a C string, or return the existing one
    pub fn get_str(&self) -> &str {
        self.s
            .get_or_init(|| self.c.get().map_or_else(
                || String::default(),
                |c| c.to_string_lossy().into_owned()
            )).as_str()
    }

    // convert the stored string to a C string, or return the existing one
    pub fn get_c_str(&self) -> &CStr {
        self.c
            .get_or_init(|| self.s.get().map_or_else(
                || CString::default(),
                |s| {
                    let v = s.as_bytes().to_vec();
                    // safe, because the source string can't have nulls
                    // in it by definition
                    unsafe {
                        CString::from_vec_unchecked(v)
                    }
                }
            )).as_c_str()
    }

    // get a new string
    pub fn to_string(&self) -> String {
        self.get_str().to_string()
    }
    // get a new cstring
    pub fn to_c_string(&self) -> CString {
        CString::from(self.get_c_str())
    }

    // which version is the canonical version, to avoid lossy conversion when
    // necessary
    pub fn canonical(&self) -> Canonical {
        self.canonical
    }
    pub fn is_str_canonical(&self) -> bool {
        self.canonical == Canonical::String
    }
    pub fn is_c_str_canonical(&self) -> bool {
        self.canonical == Canonical::CString
    }

    // original bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self.canonical {
            Canonical::String => self.s.get().unwrap().as_bytes(),
            Canonical::CString => self.c.get().unwrap().to_bytes(),
        }
    }
}

// various conversion traits
impl AsRef<str> for AnyString {
    fn as_ref(&self) -> &str {
        self.get_str()
    }
}
impl AsRef<CStr> for AnyString {
    fn as_ref(&self) -> &CStr {
        self.get_c_str()
    }
}
impl AsRef<[u8]> for AnyString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<&AnyString> for AnyString {
    fn from(a: &AnyString) -> Self {
        a.clone()
    }
}

impl From<&String> for AnyString {
    fn from(s: &String) -> Self {
        Self::from_string(s.clone())
    }
}
impl From<&str> for AnyString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<&CString> for AnyString {
    fn from(c: &CString) -> Self {
        Self::from_c_string(c.clone())
    }
}
impl From<&CStr> for AnyString {
    fn from(c: &CStr) -> Self {
        Self::from_c_str(c)
    }
}

impl From<AnyString> for String {
    fn from(s: AnyString) -> Self {
        s.to_string()
    }
}
impl From<&AnyString> for String {
    fn from(s: &AnyString) -> Self {
        s.to_string()
    }
}
impl From<AnyString> for CString {
    fn from(s: AnyString) -> Self {
        s.to_c_string()
    }
}
impl From<&AnyString> for CString {
    fn from(s: &AnyString) -> Self {
        s.to_c_string()
    }
}


// comparisons. compare the canonical versions if they are the same, otherwise compare the raw
// bytes. there's pros and cons to converting before or not, but I don't think there's a good
// answer and honestly at this point, you shouldn't be doing this really
impl PartialEq for AnyString {
    fn eq(&self, other: &Self) -> bool {
        if self.canonical == other.canonical {
            match self.canonical {
                Canonical::String => self.s.get().unwrap().eq(other.s.get().unwrap()),
                Canonical::CString => self.c.get().unwrap().eq(other.c.get().unwrap()),
            }
        } else {
            self.as_bytes().eq(other.as_bytes())
        }
    }
}

impl Eq for AnyString {}

impl Ord for AnyString {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.canonical == other.canonical {
            match self.canonical {
                Canonical::String => self.s.get().unwrap().cmp(other.s.get().unwrap()),
                Canonical::CString => self.c.get().unwrap().cmp(other.c.get().unwrap()),
            }
        } else {
            self.as_bytes().cmp(other.as_bytes())
        }
    }
}

impl PartialOrd for AnyString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.canonical == other.canonical {
            match self.canonical {
                Canonical::String => self.s.get().unwrap().partial_cmp(other.s.get().unwrap()),
                Canonical::CString => self.c.get().unwrap().partial_cmp(other.c.get().unwrap()),
            }
        } else {
            self.as_bytes().partial_cmp(other.as_bytes())
        }
    }
}

// display the proper string form always
impl fmt::Display for AnyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S_HELLO: &'static str = &"hello world";
    const C_HELLO: &'static CStr = &c"hello world";

    fn assert_inner(a: &AnyString, s: Option<&str>, c: Option<&CStr>) {
        assert_eq!(a.get_str_inner(), s);
        assert_eq!(a.get_c_str_inner(), c);
    }

    #[test]
    fn from_string() {
        let a = AnyString::from_string(S_HELLO.into());
        assert_inner(&a, Some(S_HELLO), None);
    }

    #[test]
    fn from_c_string() {
        let a = AnyString::from_c_string(C_HELLO.into());
        assert_inner(&a, None, Some(C_HELLO));
    }

    #[test]
    fn get_str_cached() {
        let a = AnyString::from_string(S_HELLO.into());
        assert_inner(&a, Some(S_HELLO), None);
        assert_eq!(a.get_str(), S_HELLO);
        assert_inner(&a, Some(S_HELLO), None);
    }

    #[test]
    fn get_str_converted() {
        let a = AnyString::from_c_string(C_HELLO.into());
        assert_inner(&a, None, Some(C_HELLO));
        assert_eq!(a.get_str(), S_HELLO);
        assert_inner(&a, Some(S_HELLO), Some(C_HELLO));
    }

    #[test]
    fn get_c_str_cached() {
        let a = AnyString::from_c_string(C_HELLO.into());
        assert_inner(&a, None, Some(C_HELLO));
        assert_eq!(a.get_c_str(), C_HELLO);
        assert_inner(&a, None, Some(C_HELLO));
    }

    #[test]
    fn get_c_str_converted() {
        let a = AnyString::from_string(S_HELLO.into());
        assert_inner(&a, Some(S_HELLO), None);
        assert_eq!(a.get_c_str(), C_HELLO);
        assert_inner(&a, Some(S_HELLO), Some(C_HELLO));
    }
}
