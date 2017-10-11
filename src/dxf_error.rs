// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::error;
use std::fmt;
use std::io;
use std::num;

use ::image;

use CodePair;

#[derive(Debug)]
pub enum DxfError {
    IoError(io::Error),
    ImageError(image::ImageError),
    ParseFloatError(num::ParseFloatError),
    ParseIntError(num::ParseIntError),
    ParseError(usize),
    UnexpectedCode(i32),
    UnexpectedCodePair(CodePair, String),
    UnexpectedByte(u8),
    UnexpectedEndOfInput,
    UnexpectedEnumValue(usize),
    UnexpectedEmptySet,
    ExpectedTableType,
    WrongValueType,
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
        match self {
            &DxfError::IoError(ref e) => write!(formatter, "{}", e),
            &DxfError::ImageError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseFloatError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseIntError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseError(ref o) => write!(formatter, "there was a general parsing error at line/offset {}", o),
            &DxfError::UnexpectedCode(c) => write!(formatter, "an unexpected code '{}' was encountered", c),
            &DxfError::UnexpectedCodePair(ref cp, ref s) => write!(formatter, "the code pair '{:?}' was not expected at this time: {}", cp, s),
            &DxfError::UnexpectedByte(ref b) => write!(formatter, "the byte '0x{:02x}' was not expected at this time", b),
            &DxfError::UnexpectedEndOfInput => write!(formatter, "the input unexpectedly ended before the drawing was completely loaded"),
            &DxfError::UnexpectedEnumValue(ref o) => write!(formatter, "the specified enum value does not fall into the expected range at line/offset {}", o),
            &DxfError::UnexpectedEmptySet => write!(formatter, "the set was not expected to be empty"),
            &DxfError::ExpectedTableType => write!(formatter, "a 2/<table-type> code pair was expected"),
            &DxfError::WrongValueType => write!(formatter, "the CodePairValue does not contain the requested type"),
            &DxfError::InvalidBinaryFile => write!(formatter, "the binary file is invalid"),
            &DxfError::WrongItemType => write!(formatter, "the specified item type is not correct"),
        }
    }
}

impl error::Error for DxfError {
    fn description(&self) -> &str {
        match self {
            &DxfError::IoError(ref e) => e.description(),
            &DxfError::ImageError(ref e) => e.description(),
            &DxfError::ParseFloatError(ref e) => e.description(),
            &DxfError::ParseIntError(ref e) => e.description(),
            &DxfError::ParseError(_) => "there was a general parsing error",
            &DxfError::UnexpectedCode(_) => "an unexpected code was encountered",
            &DxfError::UnexpectedCodePair(_, _) => "an unexpected code pair was encountered",
            &DxfError::UnexpectedByte(_) => "an unexpected byte was encountered",
            &DxfError::UnexpectedEndOfInput => "the input unexpectedly ended before the drawing was completely loaded",
            &DxfError::UnexpectedEnumValue(_) => "the specified enum value does not fall into the expected range",
            &DxfError::UnexpectedEmptySet => "the set was not expected to be empty",
            &DxfError::ExpectedTableType => "a 2/<table-type> code pair was expected",
            &DxfError::WrongValueType => "the CodePairValue does not contain the requested type",
            &DxfError::InvalidBinaryFile => "the binary file is invalid",
            &DxfError::WrongItemType => "the specified item type is not correct",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match self {
            &DxfError::IoError(ref e) => Some(e),
            &DxfError::ImageError(ref e) => Some(e),
            &DxfError::ParseFloatError(ref e) => Some(e),
            &DxfError::ParseIntError(ref e) => Some(e),
            _ => None,
        }
    }
}
