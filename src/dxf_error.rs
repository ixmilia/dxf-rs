use std::error;
use std::fmt;
use std::io;
use std::num;

use crate::CodePair;

#[derive(Debug)]
pub enum DxfError {
    IoError(io::Error),
    ImageError(image::ImageError),
    ParseFloatError(num::ParseFloatError, usize),
    ParseIntError(num::ParseIntError, usize),
    ParseError(usize),
    UnexpectedCode(i32, usize),
    UnexpectedCodePair(CodePair, String),
    UnexpectedByte(u8, usize),
    UnexpectedEndOfInput,
    UnexpectedEnumValue(usize),
    UnexpectedEmptySet,
    ExpectedTableType(usize),
    WrongValueType(usize),
    InvalidBinaryFile,
    MalformedString,
    WrongItemType,
}

impl From<io::Error> for DxfError {
    fn from(ioe: io::Error) -> DxfError {
        DxfError::IoError(ioe)
    }
}

impl From<::image::ImageError> for DxfError {
    fn from(ie: ::image::ImageError) -> DxfError {
        DxfError::ImageError(ie)
    }
}

impl fmt::Display for DxfError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DxfError::IoError(ref e) => write!(formatter, "{}", e),
            DxfError::ImageError(ref e) => write!(formatter, "{}", e),
            DxfError::ParseFloatError(ref e, o) => write!(formatter, "{} at line/offset {}", e, o),
            DxfError::ParseIntError(ref e, o) => write!(formatter, "{} at line/offset {}", e, o),
            DxfError::ParseError(o) => write!(
                formatter,
                "there was a general parsing error at line/offset {}",
                o
            ),
            DxfError::UnexpectedCode(c, o) => write!(
                formatter,
                "an unexpected code '{}' was encountered at line/offset {}",
                c, o
            ),
            DxfError::UnexpectedCodePair(ref cp, ref s) => write!(
                formatter,
                "the code pair '{:?}' was not expected at this time: {} at line/offset {}",
                cp, s, cp.offset
            ),
            DxfError::UnexpectedByte(ref b, o) => write!(
                formatter,
                "the byte '0x{:02x}' was not expected at this time at line/offset {}",
                b, o
            ),
            DxfError::UnexpectedEndOfInput => write!(
                formatter,
                "the input unexpectedly ended before the drawing was completely loaded"
            ),
            DxfError::UnexpectedEnumValue(o) => write!(
                formatter,
                "the specified enum value does not fall into the expected range at line/offset {}",
                o
            ),
            DxfError::UnexpectedEmptySet => {
                write!(formatter, "the set was not expected to be empty")
            }
            DxfError::ExpectedTableType(o) => write!(
                formatter,
                "a 2/<table-type> code pair was expected at line/offset {}",
                o
            ),
            DxfError::WrongValueType(o) => write!(
                formatter,
                "the CodePairValue does not contain the requested type at line/offset {}",
                o
            ),
            DxfError::InvalidBinaryFile => write!(formatter, "the binary file is invalid"),
            DxfError::MalformedString => write!(formatter, "the string is malformed"),
            DxfError::WrongItemType => write!(formatter, "the specified item type is not correct"),
        }
    }
}

impl error::Error for DxfError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            DxfError::IoError(ref e) => Some(e),
            DxfError::ImageError(ref e) => Some(e),
            DxfError::ParseFloatError(ref e, _) => Some(e),
            DxfError::ParseIntError(ref e, _) => Some(e),
            _ => None,
        }
    }
}
