// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io;
use std::io::Read;

extern crate byteorder;
use self::byteorder::{
    ByteOrder,
    LittleEndian,
};

extern crate chrono;
use self::chrono::*;

extern crate uuid;
use self::uuid::Uuid;

use enum_primitive::FromPrimitive;

use ::{CodePair, Color, DxfError, DxfResult};
use ::enums::*;
use ::tables::Layer;

pub(crate) fn verify_code(pair: &CodePair, expected: i32) -> DxfResult<()> {
    if expected == pair.code {
        Ok(())
    }
    else {
        Err(DxfError::UnexpectedCode(pair.code, pair.offset))
    }
}

pub(crate) fn as_bool(v: i16) -> bool {
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

fn get_epoch<T>(timezone: &T) -> DateTime<T>
    where T: TimeZone {
    timezone.ymd(1900, 1, 1).and_hms(0, 0, 0)
}

fn as_datetime<T>(timezone: &T, date: f64) -> DateTime<T>
    where T: TimeZone {
    // dates are represented as the fractional number of days elapsed since December 31, 1899.
    let epoch = get_epoch(timezone);
    let duration = if date == 0.0 { Duration::seconds(0) } else { f64_to_adjusted_duration(date) };
    epoch + duration
}

pub(crate) fn as_datetime_local(date: f64) -> DateTime<Local> {
    as_datetime(&Local, date)
}

pub(crate) fn as_datetime_utc(date: f64) -> DateTime<Utc> {
    as_datetime(&Utc, date)
}

#[test]
fn as_datetime_conversion_test() {
    // from AutoDesk spec: 2451544.91568287 = 31 December 1999, 9:58:35PM
    assert_eq!(Local.ymd(1999, 12, 31).and_hms(21, 58, 35), as_datetime_local(2451544.91568287));
}

fn as_double<T>(timezone: &T, date: DateTime<T>) -> f64
    where T: TimeZone {
    let epoch = get_epoch(timezone);
    let duration = date.signed_duration_since(epoch);
    (duration.num_seconds() as f64 / 24.0 / 60.0 / 60.0) + 2415021f64
}

pub(crate) fn as_double_local(date: DateTime<Local>) -> f64 {
    as_double(&Local, date)
}

pub(crate) fn as_double_utc(date: DateTime<Utc>) -> f64 {
    as_double(&Utc, date)
}

#[test]
fn as_double_conversion_test() {
    // from AutoDesk spec: 2451544.91568287[04] = 31 December 1999, 9:58:35PM
    assert_eq!(2451544.9156828704, as_double_local(Local.ymd(1999, 12, 31).and_hms(21, 58, 35)));
}

pub(crate) fn duration_as_double(duration: Duration) -> f64 {
    duration.num_seconds() as f64
}

pub(crate) fn as_duration(d: f64) -> Duration {
    Duration::seconds(d as i64)
}

pub(crate) fn as_handle(h: u32) -> String {
    format!("{:X}", h)
}

pub(crate) fn as_uuid(s: String, offset: usize) -> DxfResult<Uuid> {
    let mut reconstructed = String::new();
    let s = if s.starts_with("{") && s.ends_with("}") {
        // reconstruct the string without the braces
        for c in s.chars().skip(1).take(s.len() - 2) {
            reconstructed.push(c);
        }

        reconstructed.as_str()
    }
    else {
        s.as_str()
    };
    match Uuid::parse_str(s) {
        Ok(uuid) => Ok(uuid),
        Err(_) => Err(DxfError::ParseError(offset)),
    }
}

#[test]
fn parse_regular_and_windows_style_uuids_test() {
    let _regular = as_uuid(String::from("a2a7a23e-975b-4b54-968c-150d4c32a9b6"), 0).unwrap();
    let _windows = as_uuid(String::from("{a2a7a23e-975b-4b54-968c-150d4c32a9b6}"), 0).unwrap();
}

pub(crate) fn as_i16(b: bool) -> i16 {
    if b { 1 } else { 0 }
}

pub(crate) fn uuid_string(u: &Uuid) -> String {
    format!("{}", u)
}

pub(crate) fn combine_points_2<F, T>(v1: &mut Vec<f64>, v2: &mut Vec<f64>, result: &mut Vec<T>, comb: F)
    where F: Fn(f64, f64, f64) -> T {
    for (x, y) in v1.drain(..).zip(v2.drain(..)) {
        result.push(comb(x, y, 0.0));
    }
    v1.clear();
    v2.clear();
}

pub(crate) fn combine_points_3<F, T>(v1: &mut Vec<f64>, v2: &mut Vec<f64>, v3: &mut Vec<f64>, result: &mut Vec<T>, comb: F)
    where F: Fn(f64, f64, f64) -> T {
    for (x, (y, z)) in v1.drain(..).zip(v2.drain(..).zip(v3.drain(..))) {
        result.push(comb(x, y, z))
    }
    v1.clear();
    v2.clear();
    v3.clear();
}

pub(crate) fn default_if_empty(val: &mut String, default: &str) {
    if val.is_empty() {
        *val = String::from(default);
    }
}

pub(crate) fn ensure_positive_or_default(val: &mut f64, default: f64) {
    if *val <= 0.0 {
        *val = default
    }
}

pub(crate) fn ensure_positive_or_default_i32(val: &mut i32, default: i32) {
    if *val <= 0 {
        *val = default;
    }
}

pub(crate) fn ensure_positive_or_default_i16(val: &mut i16, default: i16) {
    if *val <= 0 {
        *val = default;
    }
}

pub(crate) fn clipping_from_bool(b: bool) -> XrefClippingBoundaryVisibility {
    XrefClippingBoundaryVisibility::from_i16(if b { 1 } else { 0 }).unwrap() // `1` and `0` will always parse so `.unwrap()` is safe
}

pub(crate) fn bool_from_clipping(c: XrefClippingBoundaryVisibility) -> bool {
    c != XrefClippingBoundaryVisibility::NotDisplayedNotPlotted
}

pub(crate) fn parse_f64(s: String, offset: usize) -> DxfResult<f64> {
    match s.trim().parse::<f64>() {
        Ok(d) => Ok(d),
        Err(e) => Err(DxfError::ParseFloatError(e, offset)),
    }
}

#[test]
fn parse_f64_test() {
    assert_eq!(3.14, parse_f64("  3.14 ".to_string(), 0).unwrap());
}

pub(crate) fn parse_i32(s: String, offset: usize) -> DxfResult<i32> {
    match s.trim().parse::<i32>() {
        Ok(i) => Ok(i),
        Err(e) => Err(DxfError::ParseIntError(e, offset)),
    }
}

#[test]
fn parse_i32_test() {
    assert_eq!(2, parse_i32("  2 ".to_string(), 0).unwrap());
}

pub(crate) fn parse_i64(s: String, offset: usize) -> DxfResult<i64> {
    match s.trim().parse::<i64>() {
        Ok(l) => Ok(l),
        Err(e) => Err(DxfError::ParseIntError(e, offset)),
    }
}

#[test]
fn parse_i64_test() {
    assert_eq!(2, parse_i64("  2 ".to_string(), 0).unwrap());
}

pub(crate) fn parse_i16(s: String, offset: usize) -> DxfResult<i16> {
    match s.trim().parse::<f64>() {
        Ok(s) => Ok(s as i16),
        Err(e) => Err(DxfError::ParseFloatError(e, offset)),
    }
}

#[test]
fn parse_i16_test() {
    assert_eq!(2, parse_i16("  2 ".to_string(), 0).unwrap());

    // some files write shorts as a double
    assert_eq!(2, parse_i16(" 2.0 ".to_string(), 0).unwrap());
}

pub(crate) fn read_color_value(layer: &mut Layer, color: i16) -> Color {
    layer.is_layer_on = color >= 0;
    Color::from_raw_value(color.abs())
}

pub(crate) fn read_line<T>(reader: &mut T) -> Option<DxfResult<String>>
    where T: Read + ?Sized {

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

pub(crate) fn read_u8<T: Read + ?Sized>(reader: &mut T) -> Option<io::Result<u8>> {
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

// safely unwrap an Option<io::Result<T>>
macro_rules! try_from_option_io_result {
    ($expr : expr) => (
        match $expr {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(DxfError::IoError(e)),
            None => return Err(DxfError::UnexpectedEndOfInput),
        }
    )
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

// safely unwrap an Option<DxfResult<T>>
macro_rules! try_from_dxf_result {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
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
    ($actual: expr, $expected: expr, $offset: expr) => (
        let actual = $actual;
        if actual != $expected {
            return Err(DxfError::UnexpectedByte($expected, $offset));
        }
    )
}

// returns the next CodePair that's not 0, or bails out early
macro_rules! next_pair {
    ($expr : expr) => (
        match $expr.next() {
            Some(Ok(pair @ CodePair { code: 0, .. })) => {
                $expr.put_back(Ok(pair));
                return Ok(true);
            },
            Some(Ok(pair)) => pair,
            Some(Err(e)) => return Err(e),
            None => return Ok(true),
        }
    )
}

// Matches an enum value or returns the default
macro_rules! enum_from_number {
    ($enum: ident, $default: ident, $fn: ident, $expr: expr) => (
        match $enum::$fn($expr) {
            Some(v) => v,
            None => $enum::$default,
        }
    )
}

// Used to safely access the last element in a Vec<T>
macro_rules! vec_last {
    ($expr : expr) => (
        match $expr.len() {
            0 => return Err(DxfError::UnexpectedEmptySet),
            l => &mut $expr[l - 1],
        }
    )
}

pub(crate) fn read_i16<T: Read>(reader: &mut T) -> DxfResult<i16> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    Ok(LittleEndian::read_i16(&[a, b]))
}

pub(crate) fn read_i32<T: Read>(reader: &mut T) -> DxfResult<i32> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    Ok(LittleEndian::read_i32(&[a, b, c, d]))
}

pub(crate) fn read_i64<T: Read>(reader: &mut T) -> DxfResult<i64> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    let e = try_from_option_io_result!(read_u8(reader));
    let f = try_from_option_io_result!(read_u8(reader));
    let g = try_from_option_io_result!(read_u8(reader));
    let h = try_from_option_io_result!(read_u8(reader));
    Ok(LittleEndian::read_i64(&[a, b, c, d, e, f, g, h]))
}

pub(crate) fn read_f32<T: Read>(reader: &mut T) -> DxfResult<f32> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    Ok(LittleEndian::read_f32(&[a, b, c, d]))
}

pub(crate) fn read_f64<T: Read>(reader: &mut T) -> DxfResult<f64> {
    let a = try_from_option_io_result!(read_u8(reader));
    let b = try_from_option_io_result!(read_u8(reader));
    let c = try_from_option_io_result!(read_u8(reader));
    let d = try_from_option_io_result!(read_u8(reader));
    let e = try_from_option_io_result!(read_u8(reader));
    let f = try_from_option_io_result!(read_u8(reader));
    let g = try_from_option_io_result!(read_u8(reader));
    let h = try_from_option_io_result!(read_u8(reader));
    Ok(LittleEndian::read_f64(&[a, b, c, d, e, f, g, h]))
}

pub(crate) fn parse_hex_string(data: &String, bytes: &mut Vec<u8>, offset: usize) -> DxfResult<()> {
    fn char_to_value(c: char, offset: usize) -> DxfResult<u8> {
        let value = match c {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'A' | 'a' => 10,
            'B' | 'b' => 11,
            'C' | 'c' => 12,
            'D' | 'd' => 13,
            'E' | 'e' => 14,
            'F' | 'f' => 15,
            _ => return Err(DxfError::ParseError(offset)),
        };
        Ok(value)
    }

    let mut complete_byte = data.len() % 2 != 0; // handles strings with an odd number of bytes
    let mut current_byte = 0u8;
    for c in data.chars() {
        let value = char_to_value(c, offset)?;
        if complete_byte {
            let x = current_byte * 16 + value;
            bytes.push(x);
        }
        else {
            current_byte = value;
        }
        complete_byte = !complete_byte;
    }

    Ok(())
}

#[test]
fn parse_hex_string_test() {
    let mut bytes = vec![];
    parse_hex_string(&String::from("012345"), &mut bytes, 0).unwrap();
    assert_eq!(vec![0x01u8, 0x23, 0x45], bytes);
}
