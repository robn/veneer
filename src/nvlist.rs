use std::error::Error;
use std::ffi::{CStr,CString};
use std::collections::HashMap;
use num_traits::FromPrimitive;

// data_type_t from include/sys/nvpair.h
#[derive(Debug,FromPrimitive)]
#[allow(non_camel_case_types)]
enum DataType {
    BOOLEAN       = 1,
    BYTE          = 2,
    INT16         = 3,
    UINT16        = 4,
    INT32         = 5,
    UINT32        = 6,
    INT64         = 7,
    UINT64        = 8,
    STRING        = 9,
    BYTE_ARRAY    = 10,
    INT16_ARRAY   = 11,
    UINT16_ARRAY  = 12,
    INT32_ARRAY   = 13,
    UINT32_ARRAY  = 14,
    INT64_ARRAY   = 15,
    UINT64_ARRAY  = 16,
    STRING_ARRAY  = 17,
    HRTIME        = 18,
    NVLIST        = 19,
    NVLIST_ARRAY  = 20,
    BOOLEAN_VALUE = 21,
    INT8          = 22,
    UINT8         = 23,
    BOOLEAN_ARRAY = 24,
    INT8_ARRAY    = 25,
    UINT8_ARRAY   = 26,
    DOUBLE        = 27,
}

#[derive(Debug)]
pub enum UnpackError {
    InvalidEncoding,
    InvalidEndian,
    UnterminatedString,
    UnknownPairType(DataType),
}

impl Error for UnpackError {
    fn description(&self) -> &str {
        match *self {
            UnpackError::InvalidEncoding    => "invalid encoding",
            UnpackError::InvalidEndian      => "invalid endian",
            UnpackError::UnterminatedString => "unterminated string",
            UnpackError::UnknownPairType(_) => "unknown pair type",
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
        }
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
        /*
        println!("----------");
        println!("pairs={}, buf={}", pairs.len(), buf.len());
        */

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

        /*
        println!("len={} name_len={} value_len={} nelems={} typ={}",
            len, name_len, value_len, nelems, typ);
        */

        let name = cstring_at!(pbuf, 12);
        //println!("{:?}", name);

        let voff = 12 + align(name_len as usize);
        let vbuf = &pbuf[voff..];

        /*
        println!("value offset: {:08x}", voff);
        hexdump::hexdump(&vbuf);
        */

        let nvtype = FromPrimitive::from_i32(typ).unwrap(); // XXX handle unknown

        let data = match nvtype {
            DataType::BOOLEAN       => Data::Boolean,
            /*
            DataType::BYTE          => todo!(), 
            DataType::INT16         => todo!(), 
            DataType::UINT16        => todo!(), 
            DataType::INT32         => todo!(), 
            DataType::UINT32        => todo!(), 
            DataType::INT64         => todo!(), 
            */

            DataType::UINT64        => Data::UInt64(int_at!(vbuf, 0, u64)),
            DataType::STRING        => Data::String(cstring_at!(vbuf, 0)),

            /*
            DataType::BYTE_ARRAY    => todo!(), 
            DataType::INT16_ARRAY   => todo!(), 
            DataType::UINT16_ARRAY  => todo!(), 
            DataType::INT32_ARRAY   => todo!(), 
            DataType::UINT32_ARRAY  => todo!(), 
            DataType::INT64_ARRAY  => todo!(), 
            */
            DataType::UINT64_ARRAY   => {
                let mut v = vec![];
                for elem in 0..nelems {
                    v.push(int_at!(vbuf, elem as usize * 8, u64));
                }
                Data::UInt64Array(v)
            },
            /*
            DataType::STRING_ARRAY  => todo!(), 
            DataType::HRTIME        => todo!(), 
            */

            DataType::NVLIST        => {
                let l;
                (l, buf) = unpack_list(buf, true)?;
                Data::List(l)
            },
            DataType::NVLIST_ARRAY  => {
                let mut v = vec![];
                for _ in 0..nelems {
                    let l;
                    (l, buf) = unpack_list(buf, true)?;
                    v.push(l);
                }
                Data::ListArray(v)
            },

            /*
            DataType::BOOLEAN_VALUE => todo!(), 
            DataType::INT8          => todo!(), 
            DataType::UINT8         => todo!(), 
            DataType::BOOLEAN_ARRAY => todo!(), 
            DataType::INT8_ARRAY    => todo!(), 
            DataType::UINT8_ARRAY   => todo!(), 
            DataType::DOUBLE        => todo!(), 
            */

            t                       => return Err(UnpackError::UnknownPairType(t)),
        };

        //println!("{:?}", data);

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
