// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::error;
use std::fmt;
use std::io;
use std::num;

use image;

use CodePair;

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
            DxfError::WrongItemType => write!(formatter, "the specified item type is not correct"),
        }
    }
}

impl error::Error for DxfError {
    fn description(&self) -> &str {
        match *self {
            DxfError::IoError(ref e) => e.description(),
            DxfError::ImageError(ref e) => e.description(),
            DxfError::ParseFloatError(ref e, _) => e.description(),
            DxfError::ParseIntError(ref e, _) => e.description(),
            DxfError::ParseError(_) => "there was a general parsing error",
            DxfError::UnexpectedCode(_, _) => "an unexpected code was encountered",
            DxfError::UnexpectedCodePair(_, _) => "an unexpected code pair was encountered",
            DxfError::UnexpectedByte(_, _) => "an unexpected byte was encountered",
            DxfError::UnexpectedEndOfInput => {
                "the input unexpectedly ended before the drawing was completely loaded"
            }
            DxfError::UnexpectedEnumValue(_) => {
                "the specified enum value does not fall into the expected range"
            }
            DxfError::UnexpectedEmptySet => "the set was not expected to be empty",
            DxfError::ExpectedTableType(_) => "a 2/<table-type> code pair was expected",
            DxfError::WrongValueType(_) => "the CodePairValue does not contain the requested type",
            DxfError::InvalidBinaryFile => "the binary file is invalid",
            DxfError::WrongItemType => "the specified item type is not correct",
        }
    }
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
