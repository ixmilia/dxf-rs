// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Write;

extern crate byteorder;
use self::byteorder::{
    LittleEndian,
    WriteBytesExt,
};

use ::{
    CodePair,
    CodePairValue,
    DxfResult,
};

#[doc(hidden)]
pub struct CodePairWriter<T>
    where T: Write {

    writer: T,
    as_ascii: bool,
}

impl<T: Write> CodePairWriter<T> {
    pub fn new_ascii_writer(writer: T) -> Self {
        CodePairWriter {
            writer: writer,
            as_ascii: true,
        }
    }
    pub fn new_binary_writer(writer: T) -> Self {
        CodePairWriter {
            writer: writer,
            as_ascii: false,
        }
    }
    pub fn write_prelude(&mut self) -> DxfResult<()> {
        match self.as_ascii {
            true => (),
            false => {
                self.writer.write_fmt(format_args!("AutoCAD Binary DXF\r\n"))?;
                self.writer.write_u8(0x1A)?;
                self.writer.write_u8(0x00)?;
            },
        }

        Ok(())
    }
    pub fn write_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        match self.as_ascii {
            true => self.write_ascii_code_pair(pair),
            false => self.write_binary_code_pair(pair),
        }
    }
    fn write_ascii_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        self.writer.write_fmt(format_args!("{: >3}\r\n", pair.code))?;
        self.writer.write_fmt(format_args!("{:?}\r\n", &pair.value))?;
        Ok(())
    }
    fn write_binary_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        // write code
        if pair.code >= 255 {
            self.writer.write_u8(255)?;
            self.writer.write_i16::<LittleEndian>(pair.code as i16)?;
        }
        else {
            self.writer.write_u8(pair.code as u8)?;
        }

        // write value
        match &pair.value {
            &CodePairValue::Boolean(s) => self.writer.write_i16::<LittleEndian>(s)?,
            &CodePairValue::Integer(i) => self.writer.write_i32::<LittleEndian>(i)?,
            &CodePairValue::Long(l) => self.writer.write_i64::<LittleEndian>(l)?,
            &CodePairValue::Short(s) => self.writer.write_i16::<LittleEndian>(s)?,
            &CodePairValue::Double(d) => self.writer.write_f64::<LittleEndian>(d)?,
            &CodePairValue::Str(ref s) => {
                for &b in CodePairValue::escape_string(s).as_bytes() {
                    self.writer.write_u8(b)?;
                }

                self.writer.write_u8(0)?;
            },
        }

        Ok(())
    }
}
