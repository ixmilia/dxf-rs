// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

pub mod enums;
mod header_generated;

extern crate chrono;
use self::chrono::*;

extern crate uuid;
use self::uuid::Uuid;

use self::header_generated::*;
use self::enums::*;
use enum_primitive::FromPrimitive;
use std::io::{BufRead, BufReader, Read};
use std::iter::Peekable;
use std::io::Result as IoResult;

include!("expected_type.rs");

pub enum DxfCodePairValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

pub fn bool_value(value: &DxfCodePairValue) -> bool {
    match value {
        &DxfCodePairValue::Boolean(b) => b,
        _ => panic!("expected bool value"),
    }
}

pub fn long_value(value: &DxfCodePairValue) -> i64 {
    match value {
        &DxfCodePairValue::Long(l) => l,
        _ => panic!("expected long value"),
    }
}

pub fn double_value(value: &DxfCodePairValue) -> f64 {
    match value {
        &DxfCodePairValue::Double(f) => f,
        _ => panic!("expected double value"),
    }
}

pub fn string_value(value: &DxfCodePairValue) -> String {
    match value {
        &DxfCodePairValue::Str(ref s) => s.clone(),
        _ => panic!("expected string value"),
    }
}

pub fn short_value(value: &DxfCodePairValue) -> i16 {
    match value {
        &DxfCodePairValue::Short(s) => s,
        _ => panic!("expected short value"),
    }
}

pub fn verify_code(expected: i32, pair: &DxfCodePair) {
    match pair.code {
        c if expected == c => (),
        _ => panic!("expected code {} but got {}", expected, pair.code),
    }
}

pub fn set_point<T>(point: &mut T, pair: &DxfCodePair)
    where T: SetPoint {
    point.set(pair);
}

pub fn as_bool(v: i16) -> bool {
    v == 1
}

fn f64_to_adjusted_duration(f: f64) -> Duration {
    let days_since_dublin = f - 2415020.0; // julian dublin offset, e.g., December 31, 1899 12:00AM
    let seconds = days_since_dublin * 24.0 * 60.0 * 60.0;
    // functions consuming this need to use 1900/01/01 instead of 1899/12/31 as a base
    // so we counter the extra day and leap second here
    Duration::seconds(seconds as i64)
        - Duration::days(1)
        + Duration::seconds(1)
}

pub fn as_datetime_local(date: f64) -> DateTime<Local> {
    // dates are represented as the fractional number of days elapsed since December 31, 1899.
    if date == 0.0 {
        Local.ymd(1900, 1, 1).and_hms(0, 0, 0)
    }
    else {
        Local.ymd(1900, 1, 1).and_hms(0, 0, 0) + f64_to_adjusted_duration(date)
    }
}

pub fn as_datetime_utc(date: f64) -> DateTime<UTC> {
    // dates are represented as the fractional number of days elapsed since December 31, 1899.
    if date == 0.0 {
        UTC.ymd(1900, 1, 1).and_hms(0, 0, 0)
    }
    else {
        UTC.ymd(1900, 1, 1).and_hms(0, 0, 0) + f64_to_adjusted_duration(date)
    }
}

pub fn as_duration(_d: f64) -> Duration {
    // TODO: preserve double value (i64)
    Duration::days(0)
}

pub fn as_handle(_s: String) -> u32 {
    0 // TODO
}

pub fn as_uuid(s: String) -> Uuid {
    Uuid::parse_str(s.as_str()).unwrap()
}

pub fn clipping_from_bool(b: bool) -> DxfXrefClippingBoundaryVisibility {
    DxfXrefClippingBoundaryVisibility::from_i16(if b { 1 } else { 0 }).unwrap()
}

pub struct DxfCodePair {
    code: i32,
    value: DxfCodePairValue,
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
        /*
        return typeof(string)
        
        */
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

pub fn version_from_string(v: String) -> DxfAcadVersion {
    match v.as_str() {
        "MC0.0" => DxfAcadVersion::Version_1_0,
        "AC1.2" => DxfAcadVersion::Version_1_2,
        "AC1.40" => DxfAcadVersion::Version_1_40,
        "AC1.50" => DxfAcadVersion::Version_2_05,
        "AC2.10" => DxfAcadVersion::Version_2_10,
        "AC2.21" => DxfAcadVersion::Version_2_21,
        "AC2.22" => DxfAcadVersion::Version_2_22,
        "AC1001" => DxfAcadVersion::Version_2_22,
        "AC1002" => DxfAcadVersion::Version_2_5,
        "AC1003" => DxfAcadVersion::Version_2_6,
        "AC1004" => DxfAcadVersion::R9,
        "AC1006" => DxfAcadVersion::R10,
        "AC1009" => DxfAcadVersion::R12,
        "AC1011" => DxfAcadVersion::R13,
        "AC1012" => DxfAcadVersion::R13,
        "AC1014" => DxfAcadVersion::R14,
        "14" => DxfAcadVersion::R14,
        "14.01" => DxfAcadVersion::R14,
        "AC1015" => DxfAcadVersion::R2000,
        "15.0" => DxfAcadVersion::R2000,
        "15.05" => DxfAcadVersion::R2000,
        "15.06" => DxfAcadVersion::R2000,
        "AC1018" => DxfAcadVersion::R2004,
        "16.0" => DxfAcadVersion::R2004,
        "16.1" => DxfAcadVersion::R2004,
        "16.2" => DxfAcadVersion::R2004,
        "AC1021" => DxfAcadVersion::R2007,
        "17.0" => DxfAcadVersion::R2007,
        "17.1" => DxfAcadVersion::R2007,
        "17.2" => DxfAcadVersion::R2007,
        "AC1024" => DxfAcadVersion::R2010,
        "18.0" => DxfAcadVersion::R2010,
        "18.1" => DxfAcadVersion::R2010,
        "18.2" => DxfAcadVersion::R2010,
        "AC1027" => DxfAcadVersion::R2013,
        "19.0" => DxfAcadVersion::R2013,
        "19.1" => DxfAcadVersion::R2013,
        "19.2" => DxfAcadVersion::R2013,
        "19.3" => DxfAcadVersion::R2013,
        _ => panic!("unsupported version {}", v),
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
    pub fn read<T>(reader: &mut T) -> IoResult<DxfFile>
        where T: Read
    {
        let buf_reader = BufReader::new(reader);
        DxfFile::load(buf_reader)
    }
    pub fn load<T: BufRead>(reader: T) -> IoResult<DxfFile>
        where T: BufRead
    {
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
    pub fn parse(s: &str) -> IoResult<DxfFile> {
        let data = String::from(s);
        let bytes = data.as_bytes();
        DxfFile::load(bytes)
    }
}

pub trait SetPoint {
    fn set(&mut self, pair: &DxfCodePair) -> ();
}

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
}

impl SetPoint for DxfPoint {
    fn set(&mut self, pair: &DxfCodePair) {
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
}

impl SetPoint for DxfVector {
    fn set(&mut self, pair: &DxfCodePair) {
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
    _value: i16,
}

impl DxfLineWeight {
    pub fn from_raw_value(v: i16) -> DxfLineWeight {
        DxfLineWeight { _value: v }
    }
    pub fn by_block() -> DxfLineWeight {
        DxfLineWeight::from_raw_value(-1)
    }
    pub fn by_layer() -> DxfLineWeight {
        DxfLineWeight::from_raw_value(-2)
    }
}
