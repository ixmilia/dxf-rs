// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::borrow::Cow;
use std::fmt;
use std::fmt::{
    Debug,
    Formatter,
};

/// Contains the data portion of a `CodePair`.
#[derive(PartialEq)]
pub enum CodePairValue {
    Boolean(i16),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

// internal visibility only
impl CodePairValue {
    pub(crate) fn escape_string<'a>(val: &'a String) -> Cow<'a, String> {
        fn needs_escaping(c: char) -> bool {
            let c = c as u8;
            c <= 0x1F || c == 0x5E
        }

        if let Some(first) = val.find(needs_escaping) {
            let mut result = String::from(&val[0..first]);
            result.reserve(val.len() - first);
            let rest = val[first..].chars();
            for c in rest {
                if needs_escaping(c) {
                    result.push('^');
                    match c as u8 {
                        0x00 => result.push('@'),
                        0x01 => result.push('A'),
                        0x02 => result.push('B'),
                        0x03 => result.push('C'),
                        0x04 => result.push('D'),
                        0x05 => result.push('E'),
                        0x06 => result.push('F'),
                        0x07 => result.push('G'),
                        0x08 => result.push('H'),
                        0x09 => result.push('I'),
                        0x0A => result.push('J'),
                        0x0B => result.push('K'),
                        0x0C => result.push('L'),
                        0x0D => result.push('M'),
                        0x0E => result.push('N'),
                        0x0F => result.push('O'),
                        0x10 => result.push('P'),
                        0x11 => result.push('Q'),
                        0x12 => result.push('R'),
                        0x13 => result.push('S'),
                        0x14 => result.push('T'),
                        0x15 => result.push('U'),
                        0x16 => result.push('V'),
                        0x17 => result.push('W'),
                        0x18 => result.push('X'),
                        0x19 => result.push('Y'),
                        0x1A => result.push('Z'),
                        0x1B => result.push('['),
                        0x1C => result.push('\\'),
                        0x1D => result.push(']'),
                        0x1E => result.push('^'),
                        0x1F => result.push('_'),
                        0x5E => result.push(' '),
                        _ => panic!("this should never happen"),
                    }
                }
                else {
                    result.push(c);
                }
            }

            Cow::Owned(result)
        }
        else {
            Cow::Borrowed(val)
        }
    }
    pub(crate) fn un_escape_string<'a>(val: &'a String) -> Cow<'a, String> {
        fn needs_un_escaping(c: char) -> bool {
            c == '^'
        }

        if let Some(first) = val.find(needs_un_escaping) {
            let mut result = String::from(&val[0..first]);
            result.reserve(val.len() - first);
            let rest = val[first..].chars();
            let mut do_escape = false;
            for c in rest {
                match c {
                    '^' if !do_escape => do_escape = true,
                    _ => {
                        if do_escape {
                            do_escape = false;
                            let c = match c {
                                '@' => 0x00,
                                'A' => 0x01,
                                'B' => 0x02,
                                'C' => 0x03,
                                'D' => 0x04,
                                'E' => 0x05,
                                'F' => 0x06,
                                'G' => 0x07,
                                'H' => 0x08,
                                'I' => 0x09,
                                'J' => 0x0A,
                                'K' => 0x0B,
                                'L' => 0x0C,
                                'M' => 0x0D,
                                'N' => 0x0E,
                                'O' => 0x0F,
                                'P' => 0x10,
                                'Q' => 0x11,
                                'R' => 0x12,
                                'S' => 0x13,
                                'T' => 0x14,
                                'U' => 0x15,
                                'V' => 0x16,
                                'W' => 0x17,
                                'X' => 0x18,
                                'Y' => 0x19,
                                'Z' => 0x1A,
                                '[' => 0x1B,
                                '\\' => 0x1C,
                                ']' => 0x1D,
                                '^' => 0x1E,
                                '_' => 0x1F,
                                ' ' => '^' as u8,
                                _ => c as u8, // invalid escape sequence, just keep the character
                            };

                            result.push(c as char);
                        }
                        else {
                            result.push(c);
                        }
                    }
                }
            }

            Cow::Owned(result)
        }
        else {
            Cow::Borrowed(val)
        }
    }
}

impl Clone for CodePairValue {
    fn clone(&self) -> Self {
        match self {
            &CodePairValue::Boolean(b) => CodePairValue::Boolean(b),
            &CodePairValue::Integer(i) => CodePairValue::Integer(i),
            &CodePairValue::Long(l) => CodePairValue::Long(l),
            &CodePairValue::Short(s) => CodePairValue::Short(s),
            &CodePairValue::Double(d) => CodePairValue::Double(d),
            &CodePairValue::Str(ref s) => CodePairValue::Str(String::from(s.as_str())),
        }
    }
}

impl Debug for CodePairValue {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            &CodePairValue::Boolean(s) => write!(formatter, "{}", s),
            &CodePairValue::Integer(i) => write!(formatter, "{: >9}", i),
            &CodePairValue::Long(l) => write!(formatter, "{}", l),
            &CodePairValue::Short(s) => write!(formatter, "{: >6}", s),
            &CodePairValue::Double(d) => write!(formatter, "{}", format_f64(d)),
            &CodePairValue::Str(ref s) => write!(formatter, "{}", CodePairValue::escape_string(s)),
        }
    }
}

/// Formats an `f64` value with up to 12 digits of precision, ensuring at least one trailing digit after the decimal.
fn format_f64(val: f64) -> String {
    // format with 12 digits of precision
    let mut val = format!("{:.12}", val);

    // trim trailing zeros
    while val.ends_with('0') {
        val.pop();
    }

    // ensure it doesn't end with a decimal
    if val.ends_with('.') {
        val.push('0');
    }

    val
}
