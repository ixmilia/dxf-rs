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

pub enum DxfCodePairValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

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

fn parse_bool(s: String) -> bool {
    match parse_short(s) {
        0 => false,
        _ => true,
    }
}

fn parse_double(s: String) -> f64 {
    match s.parse::<f64>() {
        Ok(v) => v,
        Err(_) => panic!("Unable to parse double value"),
    }
}

fn parse_int(s: String) -> i32 {
    match s.parse::<i32>() {
        Ok(v) => v,
        Err(_) => panic!("Unable to parse int value"),
    }
}

fn parse_long(s: String) -> i64 {
    match s.parse::<i64>() {
        Ok(v) => v,
        Err(_) => panic!("Unable to parse long value"),
    }
}

fn parse_short(s: String) -> i16 {
    match s.parse::<i16>() {
        Ok(v) => v,
        Err(_) => panic!("Unable to parse short value"),
    }
}

struct DxfCodePairAsciiIter<T>
    where T: BufRead
{
    reader: T,
}

pub struct DxfCodePairAsciiWriter<T>
    where T: Write
{
    writer: T,
}

macro_rules! tryr {
    ($expr : expr) => (match $expr { Ok(v) => v, Err(_) => return None })
}

fn trim_trailing_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

impl<T: BufRead> Iterator for DxfCodePairAsciiIter<T> {
    type Item = DxfCodePair;
    fn next(&mut self) -> Option<DxfCodePair> {
        // TODO: Option<IoResult<DxfCodePair>>?
        // read code
        let mut code_line = String::new();
        tryr!(self.reader.read_line(&mut code_line));
        trim_trailing_newline(&mut code_line);
        let code = tryr!(code_line.trim().parse::<i32>());

        // read value
        let mut value_line = String::new();
        tryr!(self.reader.read_line(&mut value_line));
        trim_trailing_newline(&mut value_line);
        let value = match get_expected_type(code) {
            ExpectedType::Boolean => DxfCodePairValue::Boolean(parse_bool(value_line)),
            ExpectedType::Integer => DxfCodePairValue::Integer(parse_int(value_line)),
            ExpectedType::Long => DxfCodePairValue::Long(parse_long(value_line)),
            ExpectedType::Short => DxfCodePairValue::Short(parse_short(value_line)),
            ExpectedType::Double => DxfCodePairValue::Double(parse_double(value_line)),
            ExpectedType::Str => DxfCodePairValue::Str(value_line),
        };

        Some(DxfCodePair {
            code: code,
            value: value,
        })
    }
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

pub fn read_sections<I>(file: &mut DxfFile, peekable: &mut Peekable<I>)
    where I: Iterator<Item = DxfCodePair>
{
    loop {
        match peekable.peek() {
            Some(&DxfCodePair { code: 0, value: DxfCodePairValue::Str(_) }) => {
                let pair = peekable.next().unwrap(); // consume 0/SECTION
                if string_value(&pair.value).as_str() == "EOF" { return; }
                if string_value(&pair.value).as_str() != "SECTION" { panic!("expected 0/SECTION, got 0/{}", string_value(&pair.value).as_str()); }
                match peekable.peek() {
                    Some(&DxfCodePair { code: 2, value: DxfCodePairValue::Str(_) }) => {
                        let pair = peekable.next().unwrap(); // consume 2/<section-name>
                        match string_value(&pair.value).as_str() {
                            "HEADER" => file.header = header_generated::DxfHeader::read(peekable),
                            // TODO: read other sections
                            _ => swallow_section(peekable),
                        }

                        let mut swallow_endsec = false;
                        match peekable.peek() {
                            Some(&DxfCodePair { code: 0, value: DxfCodePairValue::Str(ref s) }) if s == "ENDSEC" => swallow_endsec = true,
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
}

fn swallow_section<I>(peekable: &mut Peekable<I>)
    where I: Iterator<Item = DxfCodePair>
{
    loop {
        let mut quit = false;
        match peekable.peek() {
            Some(&DxfCodePair { code: 0, value: DxfCodePairValue::Str(ref s) }) if s == "ENDSEC" => quit = true,
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

impl DxfHeader {
    pub fn read<I>(peekable: &mut Peekable<I>) -> DxfHeader
        where I: Iterator<Item = DxfCodePair>
    {
        let mut header = DxfHeader::new();
        let mut last_header_variable;
        loop {
            match peekable.peek() {
                Some(&DxfCodePair { code: 9, value: DxfCodePairValue::Str(_) }) => {
                    last_header_variable = string_value(&peekable.next().unwrap().value);
                    loop {
                        match peekable.peek() {
                            Some(&DxfCodePair { code: c, value: _ }) if c == 0 || c == 9 => break,
                            Some(_) => {
                                let pair = peekable.next().unwrap();
                                header.set_header_value(last_header_variable.as_str(), pair);
                            },
                            None => break,
                        }
                    }
                },
                _ => break
            }
        }

        header
    }
    pub fn write<T>(&self, writer: &mut DxfCodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write
    {
        try!(writer.write_code_pair(&DxfCodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&DxfCodePair::new_str(2, "HEADER")));
        try!(self.write_code_pairs(&self.version, writer));
        try!(writer.write_code_pair(&DxfCodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
}

pub struct DxfFile {
    pub header: DxfHeader,
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
    pub fn load<T: BufRead>(reader: T) -> io::Result<DxfFile>
        where T: BufRead {
        let reader = DxfCodePairAsciiIter { reader: reader };
        let mut peekable = reader.peekable();
        let mut file = DxfFile::new();
        read_sections(&mut file, &mut peekable);
        match peekable.next() {
            Some(DxfCodePair { code: 0, value: DxfCodePairValue::Str(ref s) }) if s == "EOF" => Ok(file),
            Some(_) => panic!("expected 0/EOF but got something else"),
            None => Ok(file), //panic!("expected 0/EOF pair but no more pairs found"), // n.b., this is probably fine
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
}

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
    pub fn set(&mut self, pair: &DxfCodePair) {
        match pair.code {
            10 => self.x = double_value(&pair.value),
            20 => self.y = double_value(&pair.value),
            30 => self.z = double_value(&pair.value),
            _ => panic!("unexpected code value: {}", pair.code)
        }
    }
}

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
    pub fn set(&mut self, pair: &DxfCodePair) {
        match pair.code {
            10 => self.x = double_value(&pair.value),
            20 => self.y = double_value(&pair.value),
            30 => self.z = double_value(&pair.value),
            _ => panic!("unexpected code value: {}", pair.code)
        }
    }
}

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
    pub fn index(&self) -> u8 {
        if self.is_index() {
            self.raw_value as u8
        }
        else {
            panic!("color does not have an index")
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
