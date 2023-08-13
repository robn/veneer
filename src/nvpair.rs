use std::io::{self, Read};
use std::iter::Iterator;
use std::fmt;
use std::ffi::{CStr, CString};
use desert::FromBytesLE;
use num_traits::FromPrimitive;

// data_type_t from include/sys/nvpair.h
#[derive(Debug,FromPrimitive)]
enum PairType {
    Boolean      = 1,
    Byte         = 2,
    Int16        = 3,
    UInt16       = 4,
    Int32        = 5,
    UInt32       = 6,
    Int64        = 7,
    UInt64       = 8,
    String       = 9,
    ByteArray    = 10,
    Int16Array   = 11,
    UInt16Array  = 12,
    Int32Array   = 13,
    UInt32Array  = 14,
    Int64Array   = 15,
    UInt64Array  = 16,
    StringArray  = 17,
    HiResTime    = 18,
    NVList       = 19,
    NVListArray  = 20,
    BooleanValue = 21,
    Int8         = 22,
    UInt8        = 23,
    BooleanArray = 24,
    Int8Array    = 25,
    UInt8Array   = 26,
    Double       = 27,
}

#[derive(Debug)]
pub enum PairData {
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
pub struct Pair(CString,PairData);

#[derive(Debug)]
pub struct List {
    pub version: i32,
    pub flags:   u32,
    //pub pairs    Vec<Pair>,
    //pub pairs:   HashMap<CString,Data>,
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
            ParseError::InvalidEncoding    => f.write_str("invalid encoding"),
            ParseError::InvalidEndian      => f.write_str("invalid endian"),
            ParseError::ShortRead          => f.write_str("short read"),
            ParseError::UnterminatedString => f.write_str("unterminated string"),
            ParseError::UnknownPairType(_) => f.write_str("unknown pair type"),
            ParseError::IOError(_)         => f.write_str("io error"),
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
pub struct Parser<R> {
    r: R,
    encoding: Encoding,
    endian: Endian,
}

pub fn parser<R: Read>(mut r: R) -> Result<Parser<R>, ParseError> {
    let mut buf: [u8; 4] = [0; 4];
    r.read_exact(&mut buf)?;

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

    // The top list is implied, so just move the ptr to the first pair.
    r.read_exact(&mut buf)?;
    let version = i32::from_bytes_le(&buf).unwrap().1;
    r.read_exact(&mut buf)?;
    let flags = u32::from_bytes_le(&buf).unwrap().1;

    assert_eq!(version, 0); // NV_VERSION
    assert_eq!(flags, 1);   // XXX NV_UNIQUE_NAME|NV_UNIQUE_NAME_TYPE

    Ok(Parser {
        r,
        encoding,
        endian,
    })
}

impl<R: Read> Parser<R> {
    pub fn iter(self) -> PairIterator<R> {
        PairIterator {
            p: self,
        }
    }
}

#[derive(Debug)]
pub struct PairIterator<R> {
    p: Parser<R>,
}

#[inline(always)]
fn align(n: usize) -> usize {
    (n + 7) & ! 7
}

impl<R: Read> PairIterator<R> {
    fn parse_int<'a, T>(&'a self, buf: &'a [u8]) -> Result<(T,&[u8]),ParseError>
    where
        T: FromBytesLE
    {
        let s = std::mem::size_of::<T>();
        if buf.len() < s { return Err(ParseError::ShortRead) }
        let v = T::from_bytes_le(&buf).unwrap().1;
        Ok((v, &buf[s..]))
    }

    fn parse_string<'a>(&'a self, buf: &'a [u8]) -> Result<(CString,&[u8]),ParseError> {
        let cstr = CStr::from_bytes_until_nul(buf)?;
        let s = align(cstr.to_bytes_with_nul().len());
        Ok((cstr.into(), &buf[s..]))
    }

    fn parse_nvlist<'a>(&'a self, buf: &'a [u8]) -> Result<(List,&[u8]),ParseError> {
        let (version, buf) = self.parse_int::<i32>(buf)?;
        let (flags, mut buf) = self.parse_int::<u32>(buf)?;

        buf = &buf[8..]; // only for embedded nvlists, not root

        //let (pairs, rest) = unpack_pairs(pairs_buf)?;

        Ok((List {
            version: version,
            flags: flags,
        }, buf))
    }

    fn take_pair(&mut self) -> Result<Option<Pair>,ParseError> {
        let len: usize = {
            let mut buf: [u8; 4] = [0; 4];
            self.p.r.read_exact(&mut buf)?;
            let (len, _) = self.parse_int::<u32>(&buf)?;
            len as usize
        };
        if len == 0 {
            return Ok(None);
        }

        let mut buf = vec![0; len-4];
        self.p.r.read_exact(&mut buf)?;

        let (name_len, mut buf) = self.parse_int::<i16>(&buf)?;
        buf = &buf[2..]; // int16_t nvp_reserve

        let (nelems, buf) = self.parse_int::<i32>(&buf)?;
        let (ityp, buf) = self.parse_int::<i32>(&buf)?;

        let (name, buf) = self.parse_string(&buf)?;

        let typ: PairType = FromPrimitive::from_i32(ityp).
            ok_or(ParseError::UnknownPairType(ityp))?;

        println!("name {:?} nelems {:?} typ {:#?}", name, nelems, typ);

        let data = match typ {
            PairType::Boolean       => PairData::Boolean,

            PairType::Byte          => todo!(),
            PairType::Int16         => todo!(),
            PairType::UInt16        => todo!(),
            PairType::Int32         => todo!(),
            PairType::UInt32        => todo!(),
            PairType::Int64         => todo!(),

            PairType::UInt64        => PairData::UInt64(self.parse_int::<u64>(&buf)?.0),
            PairType::String        => PairData::String(self.parse_string(&buf)?.0),

            PairType::ByteArray     => todo!(),
            PairType::Int16Array    => todo!(),
            PairType::UInt16Array   => todo!(),
            PairType::Int32Array    => todo!(),
            PairType::UInt32Array   => todo!(),
            PairType::Int64Array    => todo!(),

            PairType::UInt64Array   => {
                let mut v = vec![];
                for elem in 0..nelems {
                    let (n, buf) = self.parse_int::<u64>(&buf)?;
                    v.push(n);
                }
                PairData::UInt64Array(v)
            },

            PairType::StringArray   => todo!(),
            PairType::HiResTime     => todo!(),

            PairType::NVList      => PairData::List(self.parse_nvlist(&buf)?.0),
            PairType::NVListArray => {
                let mut v = vec![];
                for _ in 0..nelems {
                    let (l, buf) = self.parse_nvlist(&buf)?;
                    v.push(l);
                }
                PairData::ListArray(v)
            },
/*
            PairType::Nvlist        => {
                let l;
                (l, buf) = unpack_list(buf, true)?;
                Data::List(l)
            },
            PairType::NvlistArray  => {
                let mut v = vec![];
                for _ in 0..nelems {
                    let l;
                    (l, buf) = unpack_list(buf, true)?;
                    v.push(l);
                }
                Data::ListArray(v)
            },
*/

            PairType::BooleanValue  => todo!(),
            PairType::Int8          => todo!(),
            PairType::UInt8         => todo!(),
            PairType::BooleanArray  => todo!(),
            PairType::Int8Array     => todo!(),
            PairType::UInt8Array    => todo!(),
            PairType::Double        => todo!(),
        };

        //Ok(Some(Pair { name, nelems, typ }))

        Ok(Some(Pair(name, data)))
    }
}

impl<R: Read> Iterator for PairIterator<R> {
    type Item=Result<Pair,ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.take_pair().transpose()
    }

}
