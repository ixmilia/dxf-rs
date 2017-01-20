// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate byteorder;
use self::byteorder::{
    ByteOrder,
    LittleEndian,
};

use ::{
    CodePair,
    CodePairValue,
    DxfError,
    DxfResult,
    ExpectedType,
    get_expected_type,
};

use helper_functions::*;
use std::io;
use std::io::Read;

#[doc(hidden)]
pub struct CodePairIter<T: Read> {
    reader: T,
    first_line: String,
    read_first_line: bool,
    read_as_ascii: bool,
    binary_detection_complete: bool,
}

// used to turn Result<T> into Option<Result<T>>.
macro_rules! try_into_option {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        }
    )
}

// safely unwrap an Option<io::Result<T>>
macro_rules! try_from_option_io_result {
    ($expr : expr) => (
        match $expr {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Some(Err(DxfError::IoError(e))),
            None => return None,
        }
    )
}

// safely unwrap an Option<DxfResult<T>>
macro_rules! try_from_option_dxf_result {
    ($expr : expr) => (
        match $expr {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Some(Err(e)),
            None => return Some(Err(DxfError::UnexpectedEndOfInput)),
        }
    )
}

// safely unwrap an Option<io::Result<T>> into Err()
macro_rules! try_option_io_result_into_err {
    ($expr : expr) => (
        match $expr {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(DxfError::IoError(e)),
            None => return Err(DxfError::UnexpectedEndOfInput),
        }
    )
}

// verifies that an actual value matches the expected value
macro_rules! assert_or_err {
    ($actual : expr, $expected : expr) => (
        let actual = $actual;
        if actual != $expected {
            return Err(DxfError::UnexpectedByte($expected));
        }
    )
}

impl<T: Read> CodePairIter<T> {
    pub fn new(reader: T) -> Self {
        CodePairIter {
            reader: reader,
            first_line: String::new(),
            read_first_line: false,
            read_as_ascii: true,
            binary_detection_complete: false,
        }
    }
    fn detect_binary_or_ascii_file(&mut self) -> DxfResult<()> {
        let first_line = read_line(&mut self.reader);
        match first_line {
            Some(Ok(line)) => {
                if line == "AutoCAD Binary DXF" {
                    // swallow the next two bytes
                    assert_or_err!(try_option_io_result_into_err!(read_u8(&mut self.reader)), 0x1A);
                    assert_or_err!(try_option_io_result_into_err!(read_u8(&mut self.reader)), 0x00);
                    self.read_as_ascii = false;
                }
                else {
                    self.first_line = line;
                    self.read_as_ascii = true;
                }
            },
            Some(Err(e)) => return Err(e),
            None => return Err(DxfError::UnexpectedEndOfInput),
        }
        self.binary_detection_complete = true;
        Ok(())
    }
    fn read_code_pair_ascii(&mut self) -> Option<DxfResult<CodePair>> {
        // Read code.  If no line is available, fail gracefully.
        let code_line = match self.read_first_line {
            true => {
                match read_line(&mut self.reader) {
                    Some(Ok(v)) => v,
                    Some(Err(e)) => return Some(Err(e)),
                    None => return None,
                }
            },
            false => {
                self.read_first_line = true;

                // .clone() is fine because it'll only ever be called once and the only valid
                // values that might be cloned are: "0" and "999"; all others are errors.
                self.first_line.clone()
            },
        };
        let code_line = code_line.trim();
        if code_line.is_empty() {
            // might be an empty file only containing a newline
            return None;
        }

        let code = try_into_option!(parse_i32(String::from(code_line)));

        // Read value.  If no line is available die horribly.
        let value_line = match read_line(&mut self.reader) {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Some(Err(e)),
            None => return Some(Err(DxfError::UnexpectedEndOfInput)),
        };

        // construct the value pair
        let expected_type = match get_expected_type(code) {
            Some(t) => t,
            None => return Some(Err(DxfError::UnexpectedEnumValue)),
        };
        let value = match expected_type {
            ExpectedType::Boolean => CodePairValue::Boolean(try_into_option!(parse_i16(value_line))),
            ExpectedType::Integer => CodePairValue::Integer(try_into_option!(parse_i32(value_line))),
            ExpectedType::Long => CodePairValue::Long(try_into_option!(parse_i64(value_line))),
            ExpectedType::Short => CodePairValue::Short(try_into_option!(parse_i16(value_line))),
            ExpectedType::Double => CodePairValue::Double(try_into_option!(parse_f64(value_line))),
            ExpectedType::Str => CodePairValue::Str(CodePairValue::un_escape_string(&value_line).into_owned()),
        };

        Some(Ok(CodePair::new(code, value)))
    }
    fn read_code_pair_binary(&mut self) -> Option<DxfResult<CodePair>> {
        // Read code.  If no data is available, fail gracefully.
        let mut code = match read_u8(&mut self.reader) {
            Some(Ok(c)) => c as i32,
            Some(Err(e)) => return Some(Err(DxfError::IoError(e))),
            None => return None,
        };

        // If reading a larger code and no data is available, die horribly.
        if code == 255 {
            code = try_from_option_dxf_result!(read_i16(&mut self.reader)) as i32;
        }

        // Read value.  If no data is available die horribly.
        let expected_type = match get_expected_type(code) {
            Some(t) => t,
            None => return Some(Err(DxfError::UnexpectedEnumValue)),
        };
        let value = match expected_type {
            ExpectedType::Boolean => CodePairValue::Boolean(try_from_option_dxf_result!(read_i16(&mut self.reader))),
            ExpectedType::Integer => CodePairValue::Integer(try_from_option_dxf_result!(read_i32(&mut self.reader))),
            ExpectedType::Long => CodePairValue::Long(try_from_option_dxf_result!(read_i64(&mut self.reader))),
            ExpectedType::Short => CodePairValue::Short(try_from_option_dxf_result!(read_i16(&mut self.reader))),
            ExpectedType::Double => CodePairValue::Double(try_from_option_dxf_result!(read_f64(&mut self.reader))),
            ExpectedType::Str => {
                let mut s = String::new();
                loop {
                    match read_u8(&mut self.reader) {
                        Some(Ok(0)) => break,
                        Some(Ok(c)) => s.push(c as char),
                        Some(Err(e)) => return Some(Err(DxfError::IoError(e))),
                        None => return Some(Err(DxfError::UnexpectedEndOfInput)),
                    }
                }
                CodePairValue::Str(CodePairValue::un_escape_string(&s).into_owned())
            },
        };

        Some(Ok(CodePair::new(code, value)))
    }
}

impl<T: Read> Iterator for CodePairIter<T> {
    type Item = DxfResult<CodePair>;
    fn next(&mut self) -> Option<DxfResult<CodePair>> {
        loop {
            if !self.binary_detection_complete {
                match self.detect_binary_or_ascii_file() {
                    Ok(_) => (),
                    Err(e) => return Some(Err(e)),
                }
            }

            let pair = match self.read_as_ascii {
                true => self.read_code_pair_ascii(),
                false => self.read_code_pair_binary(),
            };

            match pair {
                Some(Ok(CodePair { code, .. })) if code != 999 => return pair,
                Some(Ok(_)) => (), // a 999 comment code, try again
                Some(Err(_)) => return pair,
                None => return None,
            }
        }
    }
}

fn read_line<T>(reader: &mut T) -> Option<DxfResult<String>>
    where T: Read {

    let mut result = String::new();
    let bytes = reader.bytes();
    for (i, c) in bytes.enumerate() {
        let c = match c {
            Ok(c) => c,
            Err(e) => return Some(Err(DxfError::IoError(e))),
        };
        match (i, c) {
            (0, 0xFE) | (1, 0xFF) => (),
            _ => {
                let c = c as char;
                if c == '\n' { break; }
                result.push(c);
            }
        }
    }

    if result.ends_with('\r') {
        result.pop();
    }

    Some(Ok(result))
}

fn read_u8<T: Read>(reader: &mut T) -> Option<io::Result<u8>> {
    let mut buf = [0];
    let size = match reader.read(&mut buf) {
        Ok(v) => v,
        Err(e) => return Some(Err(e)),
    };
    match size {
        0 => None,
        _ => Some(Ok(buf[0]))
    }
}

fn read_i16<T: Read>(reader: &mut T) -> Option<DxfResult<i16>> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    Some(Ok(LittleEndian::read_i16(&[a, b])))
}

fn read_i32<T: Read>(reader: &mut T) -> Option<DxfResult<i32>> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    Some(Ok(LittleEndian::read_i32(&[a, b, c, d])))
}

fn read_i64<T: Read>(reader: &mut T) -> Option<DxfResult<i64>> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    let e = try_from_option_io_result!(read_u8(reader));
    let f = try_from_option_io_result!(read_u8(reader));
    let g = try_from_option_io_result!(read_u8(reader));
    let h = try_from_option_io_result!(read_u8(reader));
    Some(Ok(LittleEndian::read_i64(&[a, b, c, d, e, f, g, h])))
}

fn read_f64<T: Read>(reader: &mut T) -> Option<DxfResult<f64>> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    let e = try_from_option_io_result!(read_u8(reader));
    let f = try_from_option_io_result!(read_u8(reader));
    let g = try_from_option_io_result!(read_u8(reader));
    let h = try_from_option_io_result!(read_u8(reader));
    Some(Ok(LittleEndian::read_f64(&[a, b, c, d, e, f, g, h])))
}
