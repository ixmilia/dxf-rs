// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate byteorder;

use std::fmt;
use std::fmt::{Debug, Formatter};

use self::byteorder::{BigEndian, ByteOrder};

use {CodePairValue, DxfError, DxfResult};

use helper_functions::parse_hex_string;

/// The basic primitive of a DXF file; a code indicating the type of the data contained, and the
/// data itself.
#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct CodePair {
    pub code: i32,
    pub value: CodePairValue,
    pub offset: usize,
}

impl CodePair {
    pub fn new(code: i32, val: CodePairValue, offset: usize) -> Self {
        CodePair {
            code: code,
            value: val,
            offset: offset,
        }
    }
    pub fn new_str(code: i32, val: &str) -> Self {
        CodePair::new(code, CodePairValue::Str(val.to_string()), 0)
    }
    pub fn new_string(code: i32, val: &String) -> Self {
        CodePair::new(code, CodePairValue::Str(val.clone()), 0)
    }
    pub fn new_i16(code: i32, val: i16) -> Self {
        CodePair::new(code, CodePairValue::Short(val), 0)
    }
    pub fn new_f64(code: i32, val: f64) -> Self {
        CodePair::new(code, CodePairValue::Double(val), 0)
    }
    pub fn new_i64(code: i32, val: i64) -> Self {
        CodePair::new(code, CodePairValue::Long(val), 0)
    }
    pub fn new_i32(code: i32, val: i32) -> Self {
        CodePair::new(code, CodePairValue::Integer(val), 0)
    }
    pub fn new_bool(code: i32, val: bool) -> Self {
        CodePair::new(code, CodePairValue::Boolean(if val { 1 } else { 0 }), 0)
    }
    pub fn assert_bool(&self) -> DxfResult<bool> {
        match self.value {
            CodePairValue::Boolean(s) => Ok(s != 0),
            _ => Err(DxfError::WrongValueType(self.offset)),
        }
    }
    pub fn assert_i64(&self) -> DxfResult<i64> {
        match self.value {
            CodePairValue::Long(l) => Ok(l),
            _ => Err(DxfError::WrongValueType(self.offset)),
        }
    }
    pub fn assert_i32(&self) -> DxfResult<i32> {
        match self.value {
            CodePairValue::Integer(i) => Ok(i),
            _ => Err(DxfError::WrongValueType(self.offset)),
        }
    }
    pub fn assert_f64(&self) -> DxfResult<f64> {
        match self.value {
            CodePairValue::Double(f) => Ok(f),
            _ => Err(DxfError::WrongValueType(self.offset)),
        }
    }
    pub fn assert_string(&self) -> DxfResult<String> {
        match self.value {
            CodePairValue::Str(ref s) => Ok(s.clone()),
            _ => Err(DxfError::WrongValueType(self.offset)),
        }
    }
    pub fn assert_i16(&self) -> DxfResult<i16> {
        match self.value {
            CodePairValue::Boolean(s) => Ok(s),
            CodePairValue::Short(s) => Ok(s),
            _ => Err(DxfError::WrongValueType(self.offset)),
        }
    }
}

impl CodePair {
    pub(crate) fn as_handle(&self) -> DxfResult<u32> {
        let mut bytes = vec![];
        parse_hex_string(&self.assert_string()?, &mut bytes, self.offset)?;
        while bytes.len() < 4 {
            bytes.insert(0, 0);
        }
        Ok(BigEndian::read_u32(&bytes))
    }
}

impl Debug for CodePair {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}/{:?}", self.code, &self.value)
    }
}

impl PartialEq for CodePair {
    fn eq(&self, other: &CodePair) -> bool {
        // not comparing offsets
        self.code == other.code && self.value == other.value
    }
}

#[cfg(test)]
mod tests {
    use CodePair;

    #[test]
    fn as_handle() {
        assert_eq!(0x00, CodePair::new_str(0, "0").as_handle().unwrap());
        assert_eq!(0x01, CodePair::new_str(0, "1").as_handle().unwrap());
        assert_eq!(0xABCD, CodePair::new_str(0, "ABCD").as_handle().unwrap());
    }
}
