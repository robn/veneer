use std::io::{self, Read};
use std::iter::Iterator;
use std::fmt;
use std::ffi::CStr;
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
    Uint64Array  = 16,
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
pub enum ParseError {
    InvalidEncoding,
    InvalidEndian,
    UnterminatedString,
    UnknownPairType(i32),
    IOError(std::io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // XXX get the values in
            ParseError::InvalidEncoding    => f.write_str("invalid encoding"),
            ParseError::InvalidEndian      => f.write_str("invalid endian"),
            ParseError::UnterminatedString => f.write_str("unterminated string"),
            ParseError::UnknownPairType(_) => f.write_str("unknown pair type"),
            ParseError::IOError(_)         => f.write_str("io error"),
        }
    }
}


impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::IOError(e)
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
    fn skip_bytes(&mut self, len: usize) -> Result<(),ParseError> {
        if len == 0 {
            return Ok(())
        }
        io::copy(&mut self.p.r.by_ref().take(len as u64), &mut io::sink())?;
        Ok(())
    }

    fn read_int<T>(&mut self) -> Result<T,ParseError>
    where
        T: FromBytesLE
    {
        let s = std::mem::size_of::<T>();
        let mut buf = vec![0; s];
        self.p.r.read_exact(&mut buf)?;
        let v = T::from_bytes_le(&buf).unwrap().1;
        Ok(v)
    }

    fn read_string(&mut self, len: usize) -> Result<String,ParseError> {
        let mut buf = vec![0; len];
        self.p.r.read_exact(&mut buf)?;
        let cstr = CStr::from_bytes_with_nul(&buf).unwrap(); // XXX unterminated error?
        Ok(cstr.to_string_lossy().into())
    }

    fn read_pair(&mut self) -> Result<Option<Pair>,ParseError> {
        let len = self.read_int::<u32>()?;
        if len == 0 {
            return Ok(None);
        }

        let name_len = self.read_int::<i16>()? as usize;
        self.skip_bytes(2)?;

        let nelems = self.read_int::<i32>()? as usize;
        let ityp = self.read_int::<i32>()?;

        let name = self.read_string(name_len)?;
        self.skip_bytes(align(name_len) - name_len);

/*
        let voff = 16 + align!(name_len);
        let vbytes = &self.bytes[voff..];
*/

        let typ = FromPrimitive::from_i32(ityp).
            ok_or(ParseError::UnknownPairType(ityp))?;

        Ok(Some(Pair { name, nelems, typ }))
    }
}

impl<R: Read> Iterator for PairIterator<R> {
    type Item=Result<Pair,ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_pair().transpose()
    }

}

#[derive(Debug)]
pub struct Pair {
    name: String,
    nelems: usize,
    typ: PairType,
}
