// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

#[macro_use] extern crate enum_primitive;

pub mod enums;
mod header_generated;

use self::header_generated::*;

use std::io;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::iter::Peekable;

include!("expected_type.rs");

mod helper_functions;
use helper_functions::*;

////////////////////////////////////////////////////////////////////////////////
//                                                              DxfCodePairValue
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub enum DxfCodePairValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

////////////////////////////////////////////////////////////////////////////////
//                                                                   DxfCodePair
////////////////////////////////////////////////////////////////////////////////
pub struct DxfCodePair {
    code: i32,
    value: DxfCodePairValue,
}

impl DxfCodePair {
    pub fn new(code: i32, val: DxfCodePairValue) -> DxfCodePair {
        DxfCodePair { code: code, value: val }
    }
    pub fn new_str(code: i32, val: &str) -> DxfCodePair {
        DxfCodePair::new(code, DxfCodePairValue::Str(val.to_string()))
    }
    pub fn new_string(code: i32, val: &String) -> DxfCodePair {
        DxfCodePair::new(code, DxfCodePairValue::Str(val.clone()))
    }
    pub fn new_short(code: i32, val: i16) -> DxfCodePair {
        DxfCodePair::new(code, DxfCodePairValue::Short(val))
    }
    pub fn new_double(code: i32, val: f64) -> DxfCodePair {
        DxfCodePair::new(code, DxfCodePairValue::Double(val))
    }
    pub fn new_long(code: i32, val: i64) -> DxfCodePair {
        DxfCodePair::new(code, DxfCodePairValue::Long(val))
    }
    pub fn new_bool(code: i32, val: bool) -> DxfCodePair {
        DxfCodePair::new(code, DxfCodePairValue::Boolean(val))
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                          DxfCodePairAsciiIter
////////////////////////////////////////////////////////////////////////////////
struct DxfCodePairAsciiIter<T>
    where T: BufRead
{
    reader: T,
}

// Used to turn Result into Option<io::Result<T>>
macro_rules! try_option {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
        }
    )
}

impl<T: BufRead> Iterator for DxfCodePairAsciiIter<T> {
    type Item = io::Result<DxfCodePair>;
    fn next(&mut self) -> Option<io::Result<DxfCodePair>> {
        // Read code.  If no line is available, fail gracefully.
        let mut code_line = String::new();
        match self.reader.read_line(&mut code_line) {
            Ok(_) => (),
            Err(_) => return None,
        }
        let code_line = code_line.trim();
        if code_line.is_empty() { return None; }
        let code = try_option!(code_line.parse::<i32>());

        // Read value.  If no line is available die horribly.
        let mut value_line = String::new();
        try_option!(self.reader.read_line(&mut value_line));
        trim_trailing_newline(&mut value_line);

        // construct the value pair
        let value = match try_option!(get_expected_type(code)) {
            ExpectedType::Boolean => DxfCodePairValue::Boolean(try_option!(parse_bool(value_line))),
            ExpectedType::Integer => DxfCodePairValue::Integer(try_option!(parse_int(value_line))),
            ExpectedType::Long => DxfCodePairValue::Long(try_option!(parse_long(value_line))),
            ExpectedType::Short => DxfCodePairValue::Short(try_option!(parse_short(value_line))),
            ExpectedType::Double => DxfCodePairValue::Double(try_option!(parse_double(value_line))),
            ExpectedType::Str => DxfCodePairValue::Str(value_line), // TODO: un-escape
        };

        Some(Ok(DxfCodePair {
            code: code,
            value: value,
        }))
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                        DxfCodePairAsciiWriter
////////////////////////////////////////////////////////////////////////////////
pub struct DxfCodePairAsciiWriter<T>
    where T: Write {
    writer: T,
}

impl<T: Write> DxfCodePairAsciiWriter<T> {
    pub fn write_code_pair(&mut self, pair: &DxfCodePair) -> io::Result<()> {
        try!(self.writer.write_fmt(format_args!("{: >3}\r\n", pair.code)));
        let str_val = match &pair.value {
            &DxfCodePairValue::Boolean(b) => String::from(if b { "1" } else { "0" }),
            &DxfCodePairValue::Integer(i) => format!("{}", i),
            &DxfCodePairValue::Long(l) => format!("{}", l),
            &DxfCodePairValue::Short(s) => format!("{}", s),
            &DxfCodePairValue::Double(d) => format!("{:.12}", d), // TODO: use proper precision
            &DxfCodePairValue::Str(ref s) => s.clone(), // TODO: escape
        };
        try!(self.writer.write_fmt(format_args!("{}\r\n", str_val.as_str())));
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                     DxfHeader
////////////////////////////////////////////////////////////////////////////////
// implementation is in `header_generated.rs`
impl DxfHeader {
    pub fn read<I>(peekable: &mut Peekable<I>) -> io::Result<DxfHeader>
        where I: Iterator<Item = io::Result<DxfCodePair>>
    {
        let mut header = DxfHeader::new();
        loop {
            match peekable.peek() {
                Some(&Ok(DxfCodePair { code: 9, value: _ })) => {
                    let pair = peekable.next().unwrap().ok().unwrap(); // unwrap() and ok() calls are valid due to the match above
                    let last_header_variable = string_value(&pair.value);
                    loop {
                        match peekable.peek() {
                            Some(&Ok(DxfCodePair { code: c, value: _ })) if c == 0 || c == 9 => break, // 0/ENDSEC or a new header variable
                            Some(&Ok(_)) => {
                                let pair = peekable.next().unwrap().ok().unwrap(); // unwrap() and ok() calls are valid due to the match above
                                try!(header.set_header_value(last_header_variable.as_str(), &pair));
                            },
                            Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "unable to read header variable value")),
                            None => break,
                        }
                    }
                },
                Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "unable to read header")),
                _ => break
            }
        }

        Ok(header)
    }
    pub fn write<T>(&self, writer: &mut DxfCodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write
    {
        try!(writer.write_code_pair(&DxfCodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&DxfCodePair::new_str(2, "HEADER")));
        try!(self.write_code_pairs(writer));
        try!(writer.write_code_pair(&DxfCodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                       DxfFile
////////////////////////////////////////////////////////////////////////////////
pub struct DxfFile {
    pub header: DxfHeader,
}

// Used to turn Result<T> into ::Result<T>
macro_rules! try_result {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    )
}

impl DxfFile {
    pub fn new() -> DxfFile {
        DxfFile {
            header: DxfHeader::new(),
        }
    }
    pub fn read<T>(reader: &mut T) -> io::Result<DxfFile>
        where T: Read
    {
        let buf_reader = BufReader::new(reader);
        DxfFile::load(buf_reader)
    }
    pub fn load<T>(reader: T) -> io::Result<DxfFile>
        where T: BufRead {
        let reader = DxfCodePairAsciiIter { reader: reader };
        let mut peekable = reader.peekable();
        let mut file = DxfFile::new();
        match DxfFile::read_sections(&mut file, &mut peekable) {
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            _ => (),
        }
        match peekable.next() {
            Some(Ok(DxfCodePair { code: 0, value: DxfCodePairValue::Str(ref s) })) if s == "EOF" => Ok(file),
            Some(Ok(DxfCodePair { code: c, value: v })) => Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/EOF but got {}/{:?}", c, v))),
            Some(Err(e)) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            None => Ok(file), //Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/EOF but got nothing"))), // n.b., this is probably fine
        }
    }
    pub fn parse(s: &str) -> io::Result<DxfFile> {
        let data = String::from(s);
        let bytes = data.as_bytes();
        DxfFile::load(bytes)
    }
    pub fn write<T>(&self, writer: &mut T) -> io::Result<()>
        where T: Write {
        let mut writer = DxfCodePairAsciiWriter { writer: writer };
        try!(self.header.write(&mut writer));
        // TODO: write other sections
        try!(writer.write_code_pair(&DxfCodePair::new_str(0, "EOF")));
        Ok(())
    }
    pub fn to_string(&self) -> io::Result<String> {
        use std::io::Cursor;
        let mut buf = Cursor::new(vec![]);
        try!(self.write(&mut buf));
        try!(buf.seek(SeekFrom::Start(0)));
        let reader = BufReader::new(&mut buf);
        Ok(reader.lines().map(|l| l.unwrap() + "\r\n").collect())
    }
    fn read_sections<I>(file: &mut DxfFile, peekable: &mut Peekable<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<DxfCodePair>> {
        loop {
            match peekable.peek() {
                Some(&Ok(DxfCodePair { code: 0, value: DxfCodePairValue::Str(_) })) => {
                    let pair = peekable.next().unwrap().ok().unwrap(); // consume 0/SECTION.  unwrap() and ok() calls are valid due to the match above
                    if string_value(&pair.value).as_str() == "EOF" { break; }
                    if string_value(&pair.value).as_str() != "SECTION" { return Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/SECTION, got 0/{}", string_value(&pair.value).as_str()))); }
                    match peekable.peek() {
                        Some(&Ok(DxfCodePair { code: 2, value: DxfCodePairValue::Str(_) })) => {
                            let pair = peekable.next().unwrap().ok().unwrap(); // consume 2/<section-name>.  unwrap() and ok() calls are valid due to the match above
                            match string_value(&pair.value).as_str() {
                                "HEADER" => file.header = try!(header_generated::DxfHeader::read(peekable)),
                                // TODO: read other sections
                                _ => DxfFile::swallow_section(peekable),
                            }

                            let mut swallow_endsec = false;
                            match peekable.peek() {
                                Some(&Ok(DxfCodePair { code: 0, value: DxfCodePairValue::Str(ref s) })) if s == "ENDSEC" => swallow_endsec = true,
                                _ => (), // expected 0/ENDSEC
                            }

                            if swallow_endsec {
                                peekable.next();
                            }
                        },
                        _ => (), // expected 2/<section-name>
                    }
                },
                _ => break,
            }
        }

        Ok(())
    }
    fn swallow_section<I>(peekable: &mut Peekable<I>)
        where I: Iterator<Item = io::Result<DxfCodePair>> {
        loop {
            let mut quit = false;
            match peekable.peek() {
                Some(&Ok(DxfCodePair { code: 0, value: DxfCodePairValue::Str(ref s) })) if s == "ENDSEC" => quit = true,
                _ => (),
            }

            if quit {
                return;
            }
            else {
                peekable.next();
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                      DxfPoint
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq)]
pub struct DxfPoint {
    x: f64,
    y: f64,
    z: f64,
}

impl DxfPoint {
    pub fn new(x: f64, y: f64, z: f64) -> DxfPoint {
        DxfPoint{
            x: x,
            y: y,
            z: z,
        }
    }
    pub fn origin() -> DxfPoint {
        DxfPoint::new(0.0, 0.0, 0.0)
    }
    pub fn set(&mut self, pair: &DxfCodePair) -> io::Result<()> {
        match pair.code {
            10 => self.x = double_value(&pair.value),
            20 => self.y = double_value(&pair.value),
            30 => self.z = double_value(&pair.value),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unexpected code for DxfPoint: {}", pair.code))),
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                     DxfVector
////////////////////////////////////////////////////////////////////////////////
pub struct DxfVector {
    x: f64,
    y: f64,
    z: f64,
}

impl DxfVector {
    pub fn new(x: f64, y: f64, z: f64) -> DxfVector {
        DxfVector {
            x: x,
            y: y,
            z: z,
        }
    }
    pub fn zero() -> DxfVector {
        DxfVector::new(0.0, 0.0, 0.0)
    }
    pub fn x_axis() -> DxfVector {
        DxfVector::new(1.0, 0.0, 0.0)
    }
    pub fn y_axis() -> DxfVector {
        DxfVector::new(0.0, 1.0, 0.0)
    }
    pub fn z_axis() -> DxfVector {
        DxfVector::new(0.0, 0.0, 1.0)
    }
    pub fn set(&mut self, pair: &DxfCodePair) -> io::Result<()> {
        match pair.code {
            10 => self.x = double_value(&pair.value),
            20 => self.y = double_value(&pair.value),
            30 => self.z = double_value(&pair.value),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unexpected code for DxfVector: {}", pair.code))),
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                      DxfColor
////////////////////////////////////////////////////////////////////////////////
pub struct DxfColor {
    raw_value: i16,
}

impl DxfColor {
    pub fn is_by_layer(&self) -> bool {
        self.raw_value == 256
    }
    pub fn is_by_entity(&self) -> bool {
        self.raw_value == 257
    }
    pub fn is_by_block(&self) -> bool {
        self.raw_value == 0
    }
    pub fn is_turned_off(&self) -> bool {
        self.raw_value < 0
    }
    pub fn set_by_layer(&mut self) {
        self.raw_value = 256
    }
    pub fn set_by_block(&mut self) {
        self.raw_value = 0
    }
    pub fn set_by_entity(&mut self) {
        self.raw_value = 257
    }
    pub fn turn_off(&mut self) {
        self.raw_value = -1
    }
    pub fn is_index(&self) -> bool {
        self.raw_value >= 1 && self.raw_value <= 255
    }
    pub fn index(&self) -> Option<u8> {
        if self.is_index() {
            Some(self.raw_value as u8)
        }
        else {
            None
        }
    }
    pub fn get_raw_value(&self) -> i16 {
        self.raw_value
    }
    pub fn from_raw_value(val: i16) -> DxfColor {
        DxfColor { raw_value: val }
    }
    pub fn by_layer() -> DxfColor {
        DxfColor { raw_value: 256 }
    }
    pub fn by_block() -> DxfColor {
        DxfColor { raw_value: 0 }
    }
    pub fn by_entity() -> DxfColor {
        DxfColor { raw_value: 257 }
    }
    pub fn from_index(i: u8) -> DxfColor {
        DxfColor { raw_value: i as i16 }
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                 DxfLineWeight
////////////////////////////////////////////////////////////////////////////////
pub struct DxfLineWeight {
    raw_value: i16,
}

impl DxfLineWeight {
    pub fn from_raw_value(v: i16) -> DxfLineWeight {
        DxfLineWeight { raw_value: v }
    }
    pub fn by_block() -> DxfLineWeight {
        DxfLineWeight::from_raw_value(-1)
    }
    pub fn by_layer() -> DxfLineWeight {
        DxfLineWeight::from_raw_value(-2)
    }
    pub fn get_raw_value(&self) -> i16 {
        self.raw_value
    }
}
