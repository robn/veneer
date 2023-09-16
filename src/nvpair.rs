// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use desert::FromBytesLE;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, Read};

// data_type_t from include/sys/nvpair.h
#[derive(Debug, FromPrimitive)]
enum PairType {
    Boolean = 1,
    Byte = 2,
    Int16 = 3,
    UInt16 = 4,
    Int32 = 5,
    UInt32 = 6,
    Int64 = 7,
    UInt64 = 8,
    String = 9,
    ByteArray = 10,
    Int16Array = 11,
    UInt16Array = 12,
    Int32Array = 13,
    UInt32Array = 14,
    Int64Array = 15,
    UInt64Array = 16,
    StringArray = 17,
    HiResTime = 18,
    NVList = 19,
    NVListArray = 20,
    BooleanValue = 21,
    Int8 = 22,
    UInt8 = 23,
    BooleanArray = 24,
    Int8Array = 25,
    UInt8Array = 26,
    Double = 27,
}

#[derive(Debug)]
pub enum PairValue {
    Boolean,
    Byte(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    String(CString),
    ByteArray(Vec<u8>),
    Int16Array(Vec<i16>),
    UInt16Array(Vec<u16>),
    Int32Array(Vec<i32>),
    UInt32Array(Vec<i32>),
    Int64Array(Vec<i64>),
    UInt64Array(Vec<u64>),
    StringArray(Vec<CString>),
    HiResTime(i64), // XXX hrtime_t -> longlong_t -> i64
    List(PairList),
    ListArray(Vec<PairList>),
    BooleanValue(bool),
    Int8(i8),
    UInt8(u8),
    BooleanArray(Vec<bool>),
    Int8Array(Vec<i8>),
    UInt8Array(Vec<u8>),
    Double(f64),
}

#[derive(Debug)]
pub struct Pair(CString, PairValue);

impl From<Pair> for (CString, PairValue) {
    fn from(pair: Pair) -> Self {
        (pair.0, pair.1)
    }
}

impl Pair {
    pub fn key(&self) -> &CStr {
        self.0.as_c_str()
    }
    pub fn value(&self) -> &PairValue {
        &self.1
    }

    pub fn as_list(&self) -> Option<&PairList> {
        match self.1 {
            PairValue::List(ref l) => Some(l),
            _ => None,
        }
    }

    pub fn as_list_slice(&self) -> Option<&[PairList]> {
        match self.1 {
            PairValue::ListArray(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn as_u64_slice(&self) -> Option<&[u64]> {
        match self.1 {
            PairValue::UInt64Array(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn to_u64(&self) -> Option<u64> {
        match self.1 {
            PairValue::UInt64(n) => Some(n),
            _ => None,
        }
    }

    pub fn to_c_string(&self) -> Option<CString> {
        match self.1 {
            PairValue::String(ref s) => Some(s.clone()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct PairList(Vec<Pair>);

impl PairList {
    pub fn pairs(&self) -> impl Iterator<Item = &Pair> {
        self.0.iter()
    }
    pub fn keys(&self) -> impl Iterator<Item = &CStr> {
        self.0.iter().map(|p| p.0.as_ref())
    }

    pub fn get<T>(&self, t: T) -> Option<&Pair>
    where
        T: Into<Vec<u8>>,
    {
        CString::new(t)
            .ok()
            .and_then(|key| self.pairs().filter(|p| p.0 == key).next())
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidEncoding,
    InvalidEndian,
    ShortRead,
    UnterminatedString,
    UnknownPairType(i32),
    IOError(io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // XXX get the values in
            ParseError::InvalidEncoding => f.write_str("invalid encoding"),
            ParseError::InvalidEndian => f.write_str("invalid endian"),
            ParseError::ShortRead => f.write_str("short read"),
            ParseError::UnterminatedString => f.write_str("unterminated string"),
            ParseError::UnknownPairType(_) => f.write_str("unknown pair type"),
            ParseError::IOError(_) => f.write_str("io error"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::IOError(e)
    }
}

impl From<core::ffi::FromBytesUntilNulError> for ParseError {
    fn from(_: core::ffi::FromBytesUntilNulError) -> Self {
        ParseError::UnterminatedString
    }
}

#[derive(PartialEq, Debug)]
pub enum Encoding {
    Native,
    XDR,
}

#[derive(PartialEq, Debug)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug)]
pub struct Parser;

#[inline(always)]
fn align(n: usize) -> usize {
    (n + 7) & !7
}

pub fn parse<R: Read>(mut r: R) -> Result<PairList, ParseError> {
    let mut buf: Vec<u8> = vec![];
    r.read_to_end(&mut buf)?;
    Parser::new().parse(&buf)
}

impl Parser {
    pub fn new() -> Parser {
        Parser
    }

    fn parse<'a>(&'a self, buf: &'a [u8]) -> Result<PairList, ParseError> {
        let encoding = match buf[0] {
            0 => Encoding::Native,
            1 => Encoding::XDR,
            _ => return Err(ParseError::InvalidEncoding),
        };
        let endian = match buf[1] {
            0 => Endian::Big,
            1 => Endian::Little,
            _ => return Err(ParseError::InvalidEndian),
        };

        assert_eq!(encoding, Encoding::Native);
        assert_eq!(endian, Endian::Little);

        let lbuf = &buf[4..];

        let (version, lbuf) = self.parse_int::<i32>(&lbuf)?;
        let (flags, lbuf) = self.parse_int::<u32>(&lbuf)?;

        assert_eq!(version, 0); // NV_VERSION
        assert_eq!(flags, 1); // XXX NV_UNIQUE_NAME|NV_UNIQUE_NAME_TYPE

        let (l, _) = self.parse_nvlist(&lbuf)?;
        Ok(l)
    }

    fn parse_int<'a, T>(&'a self, buf: &'a [u8]) -> Result<(T, &[u8]), ParseError>
    where
        T: FromBytesLE,
    {
        let s = std::mem::size_of::<T>();
        if buf.len() < s {
            return Err(ParseError::ShortRead);
        }
        let v = T::from_bytes_le(&buf).unwrap().1;
        Ok((v, &buf[s..]))
    }

    fn parse_string<'a>(&'a self, buf: &'a [u8]) -> Result<(CString, &[u8]), ParseError> {
        let cstr = CStr::from_bytes_until_nul(buf)?;
        let s = align(cstr.to_bytes_with_nul().len());
        Ok((cstr.into(), &buf[s..]))
    }

    fn parse_nvlist<'a>(&'a self, buf: &'a [u8]) -> Result<(PairList, &[u8]), ParseError> {
        let mut pairs = vec![];
        let mut nbuf = buf;
        loop {
            nbuf = match self.parse_pair(nbuf)? {
                (Some(pair), buf) => {
                    pairs.push(pair);
                    buf
                }
                (None, buf) => return Ok((PairList(pairs), buf)),
            }
        }
    }

    fn parse_pair<'a>(&'a self, buf: &'a [u8]) -> Result<(Option<Pair>, &[u8]), ParseError> {
        let (len, buf) = self.parse_int::<i32>(&buf)?;
        if len == 0 {
            return Ok((None, buf));
        }

        let (buf, mut nbuf) = buf.split_at((len - 4) as usize);

        let (_, buf) = self.parse_int::<i16>(&buf)?; // name_len
        let (_, buf) = self.parse_int::<i16>(&buf)?; // nvp_reserve

        let (nelems, buf) = self.parse_int::<i32>(&buf)?;
        let (ityp, buf) = self.parse_int::<i32>(&buf)?;

        let (name, buf) = self.parse_string(&buf)?;

        let typ: PairType =
            FromPrimitive::from_i32(ityp).ok_or(ParseError::UnknownPairType(ityp))?;

        //println!("name {:?} nelems {:?} typ {:?}", name, nelems, typ);

        let data = match typ {
            PairType::Boolean => PairValue::Boolean,

            PairType::Byte => todo!(),
            PairType::Int16 => todo!(),
            PairType::UInt16 => todo!(),
            PairType::Int32 => todo!(),
            PairType::UInt32 => todo!(),
            PairType::Int64 => todo!(),

            PairType::UInt64 => PairValue::UInt64(self.parse_int::<u64>(&buf)?.0),
            PairType::String => PairValue::String(self.parse_string(&buf)?.0),

            PairType::ByteArray => todo!(),
            PairType::Int16Array => todo!(),
            PairType::UInt16Array => todo!(),
            PairType::Int32Array => todo!(),
            PairType::UInt32Array => todo!(),
            PairType::Int64Array => todo!(),

            PairType::UInt64Array => {
                let mut v = vec![];
                let mut pbuf = buf;
                for _ in 0..nelems {
                    let n;
                    (n, pbuf) = self.parse_int::<u64>(&pbuf)?;
                    v.push(n);
                }
                PairValue::UInt64Array(v)
            }

            PairType::StringArray => todo!(),
            PairType::HiResTime => todo!(),

            // embedded nvlists start at the "next" pair position, rather than at the "value"
            // position of this pair. the real "next" pair follows after the nvlist
            PairType::NVList => {
                let (l, pbuf) = self.parse_nvlist(&nbuf)?;
                nbuf = pbuf;
                PairValue::List(l)
            }
            PairType::NVListArray => {
                let mut v = vec![];
                let mut pbuf = nbuf;
                for _ in 0..nelems {
                    let l;
                    (l, pbuf) = self.parse_nvlist(&pbuf)?;
                    v.push(l);
                }
                nbuf = pbuf;
                PairValue::ListArray(v)
            }

            PairType::BooleanValue => todo!(),
            PairType::Int8 => todo!(),
            PairType::UInt8 => todo!(),
            PairType::BooleanArray => todo!(),
            PairType::Int8Array => todo!(),
            PairType::UInt8Array => todo!(),
            PairType::Double => todo!(),
        };

        Ok((Some(Pair(name, data)), nbuf))
    }
}
