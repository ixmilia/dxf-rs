// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use {CodePair, CodePairValue, DxfError, DxfResult, ExpectedType};

use code_pair_value::un_escape_ascii_to_unicode;
use helper_functions::*;
use std::io::Read;

pub(crate) struct CodePairIter<T: Read> {
    reader: T,
    first_line: String,
    read_first_line: bool,
    read_as_text: bool,
    read_text_as_utf8: bool,
    binary_detection_complete: bool,
    offset: usize,
}

impl<T: Read> CodePairIter<T> {
    pub fn new(reader: T, first_line: String) -> Self {
        CodePairIter {
            reader,
            first_line,
            read_first_line: false,
            read_as_text: true,
            read_text_as_utf8: false,
            binary_detection_complete: false,
            offset: 0,
        }
    }
    pub fn read_as_utf8(&mut self) {
        self.read_text_as_utf8 = true;
    }
    fn detect_binary_or_text_file(&mut self) -> DxfResult<()> {
        match &*self.first_line {
            "AutoCAD Binary DXF" => {
                // swallow the next two bytes
                assert_or_err!(
                    try_option_io_result_into_err!(read_u8(&mut self.reader)),
                    0x1A,
                    18
                );
                assert_or_err!(
                    try_option_io_result_into_err!(read_u8(&mut self.reader)),
                    0x00,
                    19
                );
                self.read_as_text = false;
                self.offset = 20;
            }
            _ => {
                self.read_as_text = true;
                self.offset = 1;
            }
        }
        self.binary_detection_complete = true;
        Ok(())
    }
    fn read_code_pair_text(&mut self) -> Option<DxfResult<CodePair>> {
        // Read code.  If no line is available, fail gracefully.
        let code_line = if self.read_first_line {
            self.offset += 1;
            match read_line(&mut self.reader) {
                Some(Ok(v)) => v,
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        } else {
            self.read_first_line = true;

            // .clone() is fine because it'll only ever be called once and the only valid
            // values that might be cloned are: "0" and "999"; all others are errors.
            self.first_line.clone()
        };
        let code_line = code_line.trim();
        if code_line.is_empty() {
            // might be an empty file only containing a newline
            return None;
        }

        let code_offset = self.offset;
        let code = try_into_option!(parse_i32(String::from(code_line), code_offset));

        // Read value.  If no line is available die horribly.
        self.offset += 1;
        let value_line = match read_line(&mut self.reader) {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Some(Err(e)),
            None => return Some(Err(DxfError::UnexpectedEndOfInput)),
        };

        // construct the value pair
        let expected_type = match ExpectedType::get_expected_type(code) {
            Some(t) => t,
            None => return Some(Err(DxfError::UnexpectedEnumValue(self.offset))),
        };
        let value = match expected_type {
            ExpectedType::Boolean => {
                CodePairValue::Boolean(try_into_option!(parse_i16(value_line, self.offset)))
            }
            ExpectedType::Integer => {
                CodePairValue::Integer(try_into_option!(parse_i32(value_line, self.offset)))
            }
            ExpectedType::Long => {
                CodePairValue::Long(try_into_option!(parse_i64(value_line, self.offset)))
            }
            ExpectedType::Short => {
                CodePairValue::Short(try_into_option!(parse_i16(value_line, self.offset)))
            }
            ExpectedType::Double => {
                CodePairValue::Double(try_into_option!(parse_f64(value_line, self.offset)))
            }
            ExpectedType::Str => {
                let value_line = if self.read_text_as_utf8 {
                    value_line
                } else {
                    un_escape_ascii_to_unicode(&value_line)
                };
                let value_line = CodePairValue::un_escape_string(&value_line);
                CodePairValue::Str(value_line.into_owned())
            }
        };

        Some(Ok(CodePair::new(code, value, code_offset)))
    }
    fn read_code_pair_binary(&mut self) -> Option<DxfResult<CodePair>> {
        // Read code.  If no data is available, fail gracefully.
        let mut code = match read_u8(&mut self.reader) {
            Some(Ok(c)) => i32::from(c),
            Some(Err(e)) => return Some(Err(DxfError::IoError(e))),
            None => return None,
        };
        self.offset += 1;

        // If reading a larger code and no data is available, die horribly.
        if code == 255 {
            code = i32::from(try_from_dxf_result!(read_i16(&mut self.reader)));
            self.offset += 2;
        }

        // Read value.  If no data is available die horribly.
        let expected_type = match ExpectedType::get_expected_type(code) {
            Some(t) => t,
            None => return Some(Err(DxfError::UnexpectedEnumValue(self.offset))),
        };
        let (value, read_bytes) = match expected_type {
            ExpectedType::Boolean => (
                CodePairValue::Boolean(try_from_dxf_result!(read_i16(&mut self.reader))),
                2,
            ),
            ExpectedType::Integer => (
                CodePairValue::Integer(try_from_dxf_result!(read_i32(&mut self.reader))),
                4,
            ),
            ExpectedType::Long => (
                CodePairValue::Long(try_from_dxf_result!(read_i64(&mut self.reader))),
                8,
            ),
            ExpectedType::Short => (
                CodePairValue::Short(try_from_dxf_result!(read_i16(&mut self.reader))),
                2,
            ),
            ExpectedType::Double => (
                CodePairValue::Double(try_from_dxf_result!(read_f64(&mut self.reader))),
                8,
            ),
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
                (
                    CodePairValue::Str(CodePairValue::un_escape_string(&s).into_owned()),
                    s.len(),
                )
            }
        };
        self.offset += read_bytes;

        Some(Ok(CodePair::new(code, value, self.offset)))
    }
}

impl<T: Read> Iterator for CodePairIter<T> {
    type Item = DxfResult<CodePair>;
    fn next(&mut self) -> Option<DxfResult<CodePair>> {
        loop {
            if !self.binary_detection_complete {
                match self.detect_binary_or_text_file() {
                    Ok(_) => (),
                    Err(e) => return Some(Err(e)),
                }
            }

            let pair = if self.read_as_text {
                self.read_code_pair_text()
            } else {
                self.read_code_pair_binary()
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
