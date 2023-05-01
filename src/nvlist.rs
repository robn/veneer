use std::error::Error;
use std::ffi::{CStr,CString};
use std::collections::HashMap;

// data_type_t from include/sys/nvpair.h
const DATA_TYPE_BOOLEAN:       i32 = 1;
const DATA_TYPE_BYTE:          i32 = 2;
const DATA_TYPE_INT16:         i32 = 3;
const DATA_TYPE_UINT16:        i32 = 4;
const DATA_TYPE_INT32:         i32 = 5;
const DATA_TYPE_UINT32:        i32 = 6;
const DATA_TYPE_INT64:         i32 = 7;
const DATA_TYPE_UINT64:        i32 = 8;
const DATA_TYPE_STRING:        i32 = 9;
const DATA_TYPE_BYTE_ARRAY:    i32 = 10;
const DATA_TYPE_INT16_ARRAY:   i32 = 11;
const DATA_TYPE_UINT16_ARRAY:  i32 = 12;
const DATA_TYPE_INT32_ARRAY:   i32 = 13;
const DATA_TYPE_UINT32_ARRAY:  i32 = 14;
const DATA_TYPE_INT64_ARRAY:   i32 = 15;
const DATA_TYPE_UINT64_ARRAY:  i32 = 16;
const DATA_TYPE_STRING_ARRAY:  i32 = 17;
const DATA_TYPE_HRTIME:        i32 = 18;
const DATA_TYPE_NVLIST:        i32 = 19;
const DATA_TYPE_NVLIST_ARRAY:  i32 = 20;
const DATA_TYPE_BOOLEAN_VALUE: i32 = 21;
const DATA_TYPE_INT8:          i32 = 22;
const DATA_TYPE_UINT8:         i32 = 23;
const DATA_TYPE_BOOLEAN_ARRAY: i32 = 24;
const DATA_TYPE_INT8_ARRAY:    i32 = 25;
const DATA_TYPE_UINT8_ARRAY:   i32 = 26;
const DATA_TYPE_DOUBLE:        i32 = 27;

#[derive(Debug)]
pub enum UnpackError {
    InvalidEncoding,
    InvalidEndian,
    UnterminatedString,
    UnknownPairType(i32),
    IOError(std::io::Error),
}

impl Error for UnpackError {
    fn description(&self) -> &str {
        match *self {
            UnpackError::InvalidEncoding    => "invalid encoding",
            UnpackError::InvalidEndian      => "invalid endian",
            UnpackError::UnterminatedString => "unterminated string",
            UnpackError::UnknownPairType(_) => "unknown pair type",
            UnpackError::IOError(_)         => "IO error",
        }
    }
}

impl std::fmt::Display for UnpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            UnpackError::InvalidEncoding    => f.write_str("InvalidEncoding"),
            UnpackError::InvalidEndian      => f.write_str("InvalidEndian"),
            UnpackError::UnterminatedString => f.write_str("UnterminatedStringValue"),
            UnpackError::UnknownPairType(_) => f.write_str("UnknownPairType"),
            UnpackError::IOError(_)         => f.write_str("IOError"),
        }
    }
}

impl From<std::io::Error> for UnpackError {
    fn from(e: std::io::Error) -> Self {
        UnpackError::IOError(e)
    }
}

impl From<core::ffi::FromBytesUntilNulError> for UnpackError {
    fn from(_: core::ffi::FromBytesUntilNulError) -> Self {
        UnpackError::UnterminatedString
    }
}

#[derive(PartialEq,Debug)]
pub enum Encoding {
    Native,
    XDR,
}

#[derive(PartialEq,Debug)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug)]
pub enum Data {
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
    HiResTime(i64),             // XXX hrtime_t -> longlong_t -> i64
    List(List),
    ListArray(Vec<List>),
    BooleanValue(bool),
    Int8(i8),
    UInt8(u8),
    BooleanArray(Vec<bool>),
    Int8Array(Vec<i8>),
    UInt8Array(Vec<u8>),
    Double(f64),
}

#[derive(Debug)]
pub struct List {
    pub version: i32,
    pub flags:   u32,
    pub pairs:   HashMap<CString,Data>,
}

#[derive(Debug)]
pub struct Header {
    pub encoding: Encoding,
    pub endian:   Endian,
    pub list:     List,
}

macro_rules! int_at {
    ($slice:expr, $offset:expr, $inttype:ty) => {
        {
            let s = &$slice[$offset..$offset+std::mem::size_of::<$inttype>()];
            <$inttype>::from_le_bytes(s.try_into().unwrap())
        }
    };
}

macro_rules! cstring_at {
    ($slice:expr, $offset:expr) => {
        {
            let s = &$slice[$offset..];
            CStr::from_bytes_until_nul(s)?.into()
        }
    };
}

#[inline(always)]
fn align(n: usize) -> usize {
    (n + 7) & ! 7
}

fn unpack_pairs(mut buf: &[u8]) ->
  Result<(HashMap<CString,Data>, &[u8]), UnpackError> {

    let mut pairs: HashMap<CString,Data> = HashMap::new();

    loop {
        println!("----------");
        println!("pairs={}, buf={}", pairs.len(), buf.len());

        let (lbuf, pbuf);

        (lbuf, buf) = buf.split_at(4);

        let len = {
            let len = int_at!(lbuf, 0, i32);
            if len == 0 {
                return Ok((pairs, buf));
            }
            len - 4
        } as usize;

        (pbuf, buf) = buf.split_at(len);

        let name_len = int_at!(pbuf, 0, i16);
        let nelems   = int_at!(pbuf, 4, i32);
        let typ      = int_at!(pbuf, 8, i32);

        let value_len = (len - name_len as usize - 12) & !0x7;

        println!("len={} name_len={} value_len={} nelems={} typ={}",
            len, name_len, value_len, nelems, typ);

        let name = cstring_at!(pbuf, 12);
        println!("{:?}", name);

        let voff = 12 + align(name_len as usize);
        let vbuf = &pbuf[voff..];

        println!("value offset: {:08x}", voff);
        hexdump::hexdump(&vbuf);

        let data = match typ {
            DATA_TYPE_BOOLEAN       => Data::Boolean,
            /*
            DATA_TYPE_BYTE          => todo!(), 
            DATA_TYPE_INT16         => todo!(), 
            DATA_TYPE_UINT16        => todo!(), 
            DATA_TYPE_INT32         => todo!(), 
            DATA_TYPE_UINT32        => todo!(), 
            DATA_TYPE_INT64         => todo!(), 
            */

            DATA_TYPE_UINT64        => Data::UInt64(int_at!(vbuf, 0, u64)),
            DATA_TYPE_STRING        => Data::String(cstring_at!(vbuf, 0)),

            /*
            DATA_TYPE_BYTE_ARRAY    => todo!(), 
            DATA_TYPE_INT16_ARRAY   => todo!(), 
            DATA_TYPE_UINT16_ARRAY  => todo!(), 
            DATA_TYPE_INT32_ARRAY   => todo!(), 
            DATA_TYPE_UINT32_ARRAY  => todo!(), 
            DATA_TYPE_INT64_ARRAY  => todo!(), 
            */
            DATA_TYPE_UINT64_ARRAY   => {
                let mut v = vec![];
                for elem in 0..nelems {
                    v.push(int_at!(vbuf, elem as usize * 8, u64));
                }
                Data::UInt64Array(v)
            },
            /*
            DATA_TYPE_STRING_ARRAY  => todo!(), 
            DATA_TYPE_HRTIME        => todo!(), 
            */

            DATA_TYPE_NVLIST        => {
                let l;
                (l, buf) = unpack_list(buf, true)?;
                Data::List(l)
            },
            DATA_TYPE_NVLIST_ARRAY  => {
                let mut v = vec![];
                for _ in 0..nelems {
                    let l;
                    (l, buf) = unpack_list(buf, true)?;
                    v.push(l);
                }
                Data::ListArray(v)
            },

            /*
            DATA_TYPE_BOOLEAN_VALUE => todo!(), 
            DATA_TYPE_INT8          => todo!(), 
            DATA_TYPE_UINT8         => todo!(), 
            DATA_TYPE_BOOLEAN_ARRAY => todo!(), 
            DATA_TYPE_INT8_ARRAY    => todo!(), 
            DATA_TYPE_UINT8_ARRAY   => todo!(), 
            DATA_TYPE_DOUBLE        => todo!(), 
            */

            t                       => return Err(UnpackError::UnknownPairType(t)),
        };

        println!("{:?}", data);

        pairs.insert(name, data);
    }
}

fn unpack_list(buf: &[u8], embedded: bool) -> Result<(List, &[u8]), UnpackError> {
    let version = int_at!(buf, 0, i32);
    let flags   = int_at!(buf, 4, u32);

    let pairs_off = if embedded { 0 } else { 8 };
    let pairs_buf = &buf[pairs_off..];

    let (pairs, rest) = unpack_pairs(pairs_buf)?;

    Ok((List {
        version: version,
        flags: flags,
        pairs: pairs,
    }, rest))
}

pub fn unpack(buf: &[u8]) -> Result<Header, UnpackError> {
    let encoding = match buf[0] {
        0 => Encoding::Native,
        1 => Encoding::XDR,
        _ => return Err(UnpackError::InvalidEncoding),
    };
    let endian = match buf[1] {
        0 => Endian::Big,
        1 => Endian::Little,
        _ => return Err(UnpackError::InvalidEndian),
    };

    assert_eq!(encoding, Encoding::Native);
    assert_eq!(endian, Endian::Little);

    let (list, _) = unpack_list(&buf[4..], false)?;

    Ok(Header {
        encoding: encoding,
        endian: endian,
        list: list,
    })
}
