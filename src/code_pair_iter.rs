use crate::{CodePair, CodePairValue, DxfError, DxfResult, ExpectedType};

use crate::code_pair_value::un_escape_ascii_to_unicode;
use crate::helper_functions::*;
use encoding_rs::Encoding;
use std::io::Read;

pub(crate) struct CodePairIter<T: Read> {
    reader: T,
    string_encoding: &'static Encoding,
    first_line: String,
    read_first_line: bool,
    read_as_text: bool,
    is_post_r13_binary: bool,
    returned_binary_pair: bool,
    binary_detection_complete: bool,
    offset: usize,
}

impl<T: Read> CodePairIter<T> {
    pub fn new(reader: T, string_encoding: &'static Encoding, first_line: String) -> Self {
        CodePairIter {
            reader,
            string_encoding,
            first_line,
            read_first_line: false,
            read_as_text: true,
            is_post_r13_binary: false,
            returned_binary_pair: false,
            binary_detection_complete: false,
            offset: 0,
        }
    }
    pub fn read_as_utf8(&mut self) {
        self.string_encoding = encoding_rs::UTF_8;
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
            match read_line(&mut self.reader, true, encoding_rs::WINDOWS_1252) {
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
        let value_line = match read_line(&mut self.reader, false, self.string_encoding) {
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
                let value_line = if self.string_encoding == encoding_rs::WINDOWS_1252 {
                    un_escape_ascii_to_unicode(&value_line)
                } else {
                    value_line
                };
                let value_line = CodePairValue::un_escape_string(&value_line);
                CodePairValue::Str(value_line.into_owned())
            }
            ExpectedType::Binary => {
                let mut data = vec![];
                match parse_hex_string(&value_line, &mut data, self.offset) {
                    Ok(()) => CodePairValue::Binary(data),
                    Err(e) => return Some(Err(e)),
                }
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
        if self.is_post_r13_binary {
            // post R13 codes are 2 bytes, read the second byte of the code
            let high_byte = i32::from(try_from_dxf_result!(read_u8_strict(&mut self.reader)));
            code += high_byte << 8;
            self.offset += 1;
        } else if code == 255 {
            // pre R13 codes are either 1 or 3 bytes
            code = i32::from(try_from_dxf_result!(read_i16(&mut self.reader)));
            self.offset += 2;
        }

        // Read value.  If no data is available die horribly.
        let expected_type = match ExpectedType::get_expected_type(code) {
            Some(t) => t,
            None => return Some(Err(DxfError::UnexpectedEnumValue(self.offset))),
        };
        let (value, read_bytes) = match expected_type {
            ExpectedType::Boolean => {
                // after R13 bools are encoded as a single byte
                let (b_value, read_bytes) = if self.is_post_r13_binary {
                    (
                        i16::from(try_from_dxf_result!(read_u8_strict(&mut self.reader))),
                        1,
                    )
                } else {
                    (try_from_dxf_result!(read_i16(&mut self.reader)), 2)
                };
                (CodePairValue::Boolean(b_value), read_bytes)
            }
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
                let mut value = try_from_dxf_result!(self.read_string_binary());
                if !self.returned_binary_pair && code == 0 && value == "" {
                    // If this is the first pair being read and the code is 0, the only valid string value is "SECTION".
                    // If the read value is instead empty, that means the string reader found a single 0x00 byte which
                    // indicates that this is a post R13 binary file where codes are always read as 2 bytes.  The 0x00
                    // byte was really the second byte of {0x00, 0x00}, so we need to do one more string read to catch
                    // the reader up.
                    self.is_post_r13_binary = true;
                    self.offset += 1; // account for the NULL byte that was interpreted as an empty string
                    value = try_from_dxf_result!(self.read_string_binary()); // now read the actual value
                }
                (
                    CodePairValue::Str(CodePairValue::un_escape_string(&value).into_owned()),
                    value.len() + 1, // +1 to account for the NULL terminator
                )
            }
            ExpectedType::Binary => {
                let length = try_from_dxf_result!(read_u8_strict(&mut self.reader)) as usize;
                let mut data = vec![];
                for _ in 0..length {
                    data.push(try_from_dxf_result!(read_u8_strict(&mut self.reader)));
                }

                (CodePairValue::Binary(data), length + 1) // +1 to account for initial length byte
            }
        };
        self.offset += read_bytes;
        self.returned_binary_pair = true;

        Some(Ok(CodePair::new(code, value, self.offset)))
    }
    fn read_string_binary(&mut self) -> DxfResult<String> {
        let mut s = String::new();
        loop {
            match read_u8(&mut self.reader) {
                Some(Ok(0)) => break,
                Some(Ok(c)) => s.push(c as char),
                Some(Err(e)) => return Err(DxfError::IoError(e)),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }

        Ok(s)
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

#[cfg(test)]
mod tests {
    use crate::code_pair_iter::CodePairIter;

    #[test]
    fn read_string_in_binary() {
        // code 0x0001, value 0x41 = "A", NUL
        let data: Vec<u8> = vec![0x01, 0x00, 0x41, 0x00];
        let mut reader = CodePairIter::<&[u8]> {
            reader: data.as_slice(),
            string_encoding: encoding_rs::WINDOWS_1252,
            first_line: String::from("not-important"),
            read_first_line: true,
            read_as_text: false,
            is_post_r13_binary: true,
            returned_binary_pair: true,
            binary_detection_complete: true,
            offset: 0,
        };
        let pair = reader.read_code_pair_binary().unwrap().unwrap();
        assert_eq!(1, pair.code);
        assert_eq!("A", pair.assert_string().expect("should be a string"));
    }

    #[test]
    fn read_binary_chunk_in_binary() {
        // code 0x136, length 2, data [0x01, 0x02]
        let data: Vec<u8> = vec![0x36, 0x01, 0x02, 0x01, 0x02];
        let mut reader = CodePairIter::<&[u8]> {
            reader: data.as_slice(),
            string_encoding: encoding_rs::WINDOWS_1252,
            first_line: String::from("not-important"),
            read_first_line: true,
            read_as_text: false,
            is_post_r13_binary: true,
            returned_binary_pair: true,
            binary_detection_complete: true,
            offset: 0,
        };
        let pair = reader.read_code_pair_binary().unwrap().unwrap();
        assert_eq!(310, pair.code);
        assert_eq!(
            vec![0x01, 0x02],
            pair.assert_binary().expect("should be binary")
        );
    }

    #[test]
    fn read_binary_chunk_in_ascii() {
        let data = "310\r\n0102";
        let mut reader = CodePairIter::<&[u8]> {
            reader: data.as_bytes(),
            string_encoding: encoding_rs::WINDOWS_1252,
            first_line: String::from("not-important"),
            read_first_line: true,
            read_as_text: true,
            is_post_r13_binary: false,
            returned_binary_pair: false,
            binary_detection_complete: true,
            offset: 0,
        };
        let pair = reader.read_code_pair_text().unwrap().unwrap();
        assert_eq!(310, pair.code);
        assert_eq!(
            vec![0x01, 0x02],
            pair.assert_binary().expect("should be binary")
        );
    }
}
