// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::error;
use std::fmt;
use std::io;
use std::num;

use CodePair;

#[derive(Debug)]
pub enum DxfError {
    IoError(io::Error),
    ParseFloatError(num::ParseFloatError),
    ParseIntError(num::ParseIntError),
    ParseError,
    UnexpectedCode(i32),
    UnexpectedCodePair(CodePair, String),
    UnexpectedByte(u8),
    UnexpectedEndOfInput,
    UnexpectedEnumValue,
    UnexpectedEmptySet,
    ExpectedTableType,
    WrongValueType,
}

impl From<io::Error> for DxfError {
    fn from(ioe: io::Error) -> DxfError {
        DxfError::IoError(ioe)
    }
}

impl fmt::Display for DxfError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DxfError::IoError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseFloatError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseIntError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseError => write!(formatter, "there was a general parsing error"),
            &DxfError::UnexpectedCode(c) => write!(formatter, "an unexpected code '{}' was encountered", c),
            &DxfError::UnexpectedCodePair(ref cp, ref s) => write!(formatter, "the code pair '{:?}' was not expected at this time: {}", cp, s),
            &DxfError::UnexpectedByte(ref b) => write!(formatter, "the byte '{:x}' was not expected at this time", b),
            &DxfError::UnexpectedEndOfInput => write!(formatter, "the input unexpectedly ended before the drawing was completely loaded"),
            &DxfError::UnexpectedEnumValue => write!(formatter, "the specified enum value does not fall into the expected range"),
            &DxfError::UnexpectedEmptySet => write!(formatter, "the set was not expected to be empty"),
            &DxfError::ExpectedTableType => write!(formatter, "a 2/<table-type> code pair was expected"),
            &DxfError::WrongValueType => write!(formatter, "the CodePairValue does not contain the requested type"),
        }
    }
}

impl error::Error for DxfError {
    fn description(&self) -> &str {
        match self {
            &DxfError::IoError(ref e) => e.description(),
            &DxfError::ParseFloatError(ref e) => e.description(),
            &DxfError::ParseIntError(ref e) => e.description(),
            &DxfError::ParseError => "there was a general parsing error",
            &DxfError::UnexpectedCode(_) => "an unexpected code was encountered",
            &DxfError::UnexpectedCodePair(_, _) => "an unexpected code pair was encountered",
            &DxfError::UnexpectedByte(_) => "an unexpected byte was encountered",
            &DxfError::UnexpectedEndOfInput => "the input unexpectedly ended before the drawing was completely loaded",
            &DxfError::UnexpectedEnumValue => "the specified enum value does not fall into the expected range",
            &DxfError::UnexpectedEmptySet => "the set was not expected to be empty",
            &DxfError::ExpectedTableType => "a 2/<table-type> code pair was expected",
            &DxfError::WrongValueType => "the CodePairValue does not contain the requested type",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match self {
            &DxfError::IoError(ref e) => Some(e),
            &DxfError::ParseFloatError(ref e) => Some(e),
            &DxfError::ParseIntError(ref e) => Some(e),
            _ => None,
        }
    }
}
