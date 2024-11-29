use std::cmp::Ordering;
use std::io;
use std::io::Read;
use std::time::Duration as StdDuration;

use byteorder::{ByteOrder, LittleEndian};

use chrono::prelude::*;
use chrono::Duration as ChronoDuration;

use encoding_rs::Encoding;

use uuid::Uuid;

use enum_primitive::FromPrimitive;

use crate::enums::*;
use crate::tables::Layer;
use crate::{CodePair, Color, DxfError, DxfResult};

pub(crate) fn verify_code(pair: &CodePair, expected: i32) -> DxfResult<()> {
    if expected == pair.code {
        Ok(())
    } else {
        Err(DxfError::UnexpectedCode(pair.code, pair.offset))
    }
}

pub(crate) fn as_bool(v: i16) -> bool {
    v == 1
}

fn f64_to_adjusted_duration(f: f64) -> ChronoDuration {
    let days_since_dublin = f - 2_415_020.0; // julian dublin offset, e.g., December 31, 1899 12:00AM
    let secs_per_day = 24i64 * 60 * 60;
    let seconds = days_since_dublin * secs_per_day as f64;
    // functions consuming this need to use 1900/01/01 instead of 1899/12/31 as a base
    // so we counter the extra day and leap second here
    ChronoDuration::seconds(seconds as i64 - secs_per_day + 1)
}

fn epoch<T>(timezone: &T) -> DateTime<T>
where
    T: TimeZone,
{
    // this will never fail; unwrap is ok
    timezone.with_ymd_and_hms(1900, 1, 1, 0, 0, 0).unwrap()
}

fn as_datetime<T>(timezone: &T, date: f64) -> DateTime<T>
where
    T: TimeZone,
{
    // dates are represented as the fractional number of days elapsed since December 31, 1899.
    let epoch = epoch(timezone);
    let duration = if date == 0.0 {
        ChronoDuration::seconds(0)
    } else {
        let duration = f64_to_adjusted_duration(date);
        match ChronoDuration::seconds(0).cmp(&duration) {
            Ordering::Less => duration,
            _ => ChronoDuration::seconds(0),
        }
    };
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
    assert_eq!(
        Local.with_ymd_and_hms(1999, 12, 31, 21, 58, 35).unwrap(),
        as_datetime_local(2_451_544.915_682_87)
    );
}

#[test]
fn datetime_out_of_bounds_test() {
    // these values are out of bounds for acceptable dates
    let values = vec![2305814.964456019, 1799402.631122685];
    for value in values {
        assert_eq!(
            Local.with_ymd_and_hms(1900, 1, 1, 0, 0, 0).unwrap(),
            as_datetime_local(value)
        )
    }
}

fn as_double<T>(timezone: &T, date: DateTime<T>) -> f64
where
    T: TimeZone,
{
    let epoch = epoch(timezone);
    let duration = date.signed_duration_since(epoch);
    (duration.num_seconds() as f64 / 24.0 / 60.0 / 60.0) + 2_415_021f64
}

pub(crate) fn as_double_local(date: DateTime<Local>) -> f64 {
    as_double(&Local, date)
}

pub(crate) fn as_double_utc(date: DateTime<Utc>) -> f64 {
    as_double(&Utc, date)
}

#[test]
#[allow(clippy::float_cmp)]
fn as_double_conversion_test() {
    // from AutoDesk spec: 2451544.91568287[04] = 31 December 1999, 9:58:35PM
    assert_eq!(
        2_451_544.915_682_870_4,
        as_double_local(Local.with_ymd_and_hms(1999, 12, 31, 21, 58, 35).unwrap())
    );
}

pub(crate) fn duration_as_double(duration: StdDuration) -> f64 {
    duration.as_secs() as f64
}

pub(crate) fn as_duration(d: f64) -> StdDuration {
    StdDuration::from_secs(d as u64)
}

pub(crate) fn as_uuid(s: String) -> Uuid {
    let mut reconstructed = String::new();
    let s = if s.starts_with('{') && s.ends_with('}') {
        // reconstruct the string without the braces
        for c in s.chars().skip(1).take(s.len() - 2) {
            reconstructed.push(c);
        }

        reconstructed.as_str()
    } else {
        s.as_str()
    };
    match Uuid::parse_str(s) {
        Ok(uuid) => uuid,
        Err(_) => Uuid::nil(),
    }
}

#[test]
fn parse_regular_and_windows_style_uuids_test() {
    let _regular = as_uuid(String::from("a2a7a23e-975b-4b54-968c-150d4c32a9b6"));
    let _windows = as_uuid(String::from("{a2a7a23e-975b-4b54-968c-150d4c32a9b6}"));
}

#[test]
fn parse_empty_uuid_test() {
    let _empty = as_uuid(String::from(""));
}

pub(crate) fn as_i16(b: bool) -> i16 {
    if b {
        1
    } else {
        0
    }
}

pub(crate) fn uuid_string(u: &Uuid) -> String {
    format!("{}", u)
}

pub(crate) fn combine_points_2<F, T>(
    v1: &mut Vec<f64>,
    v2: &mut Vec<f64>,
    result: &mut Vec<T>,
    comb: F,
) where
    F: Fn(f64, f64, f64) -> T,
{
    for (x, y) in v1.drain(..).zip(v2.drain(..)) {
        result.push(comb(x, y, 0.0));
    }
    v1.clear();
    v2.clear();
}

pub(crate) fn combine_points_3<F, T>(
    v1: &mut Vec<f64>,
    v2: &mut Vec<f64>,
    v3: &mut Vec<f64>,
    result: &mut Vec<T>,
    comb: F,
) where
    F: Fn(f64, f64, f64) -> T,
{
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
#[allow(clippy::float_cmp)]
fn parse_f64_test() {
    assert_eq!(2.5, parse_f64("  2.5 ".to_string(), 0).unwrap());
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

pub(crate) fn read_line<T>(
    reader: &mut T,
    allow_bom: bool,
    encoding: &'static Encoding,
) -> DxfResult<String>
where
    T: Read + ?Sized,
{
    let mut bytes = vec![];
    let mut skipping_bom = false;
    let reader_bytes = reader.bytes();
    for (i, b) in reader_bytes.enumerate() {
        let b = match b {
            Ok(b) => b,
            Err(e) => return Err(DxfError::IoError(e)),
        };
        match (i, b) {
            (0, 0xEF) if allow_bom => {
                skipping_bom = true;
            }
            (1, 0xBB) | (2, 0xBF) if skipping_bom => (), // skip UTF-8 BOM
            _ => {
                if b == b'\n' {
                    break;
                }
                bytes.push(b);
            }
        }
    }

    let mut result = match encoding.decode(&bytes) {
        (result, _, false) => String::from(&*result),
        (_, _, true) => return Err(DxfError::MalformedString),
    };

    if result.ends_with('\r') {
        result.pop();
    }

    Ok(result)
}

pub(crate) fn read_u8<T: Read + ?Sized>(reader: &mut T) -> Option<io::Result<u8>> {
    let mut buf = [0];
    let size = match reader.read(&mut buf) {
        Ok(v) => v,
        Err(e) => return Some(Err(e)),
    };
    match size {
        0 => None,
        _ => Some(Ok(buf[0])),
    }
}

// safely unwrap an Option<io::Result<T>>
macro_rules! try_from_option_io_result {
    ($expr : expr) => {
        match $expr {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(DxfError::IoError(e)),
            None => return Err(DxfError::UnexpectedEndOfInput),
        }
    };
}

// used to turn Result<T> into Option<Result<T>>.
macro_rules! try_into_option {
    ($expr : expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        }
    };
}

// safely unwrap an Option<DxfResult<T>>
macro_rules! try_from_dxf_result {
    ($expr : expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        }
    };
}

// safely unwrap an Option<io::Result<T>> into Err()
macro_rules! try_option_io_result_into_err {
    ($expr : expr) => {
        match $expr {
            Some(Ok(v)) => v,
            Some(Err(e)) => return Err(DxfError::IoError(e)),
            None => return Err(DxfError::UnexpectedEndOfInput),
        }
    };
}

// verifies that an actual value matches the expected value
macro_rules! assert_or_err {
    ($actual: expr, $expected: expr, $offset: expr) => {
        let actual = $actual;
        if actual != $expected {
            return Err(DxfError::UnexpectedByte($expected, $offset));
        }
    };
}

// returns the next CodePair that's not 0, or bails out early
macro_rules! next_pair {
    ($expr : expr) => {
        match $expr.next() {
            Some(Ok(pair @ CodePair { code: 0, .. })) => {
                $expr.put_back(Ok(pair));
                return Ok(true);
            }
            Some(Ok(pair)) => pair,
            Some(Err(e)) => return Err(e),
            None => return Ok(true),
        }
    };
}

// Matches an enum value or returns the default
macro_rules! enum_from_number {
    ($enum: ident, $default: ident, $fn: ident, $expr: expr) => {
        match $enum::$fn($expr) {
            Some(v) => v,
            None => $enum::$default,
        }
    };
}

// Used to safely access the last element in a Vec<T>
macro_rules! vec_last {
    ($expr : expr) => {
        match $expr.len() {
            0 => return Err(DxfError::UnexpectedEmptySet),
            l => &mut $expr[l - 1],
        }
    };
}

pub(crate) fn read_u8_strict<T: Read>(reader: &mut T) -> DxfResult<u8> {
    let u = try_from_option_io_result!(read_u8(reader));
    Ok(u)
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

#[allow(clippy::many_single_char_names)]
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

#[allow(clippy::many_single_char_names)]
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

pub(crate) fn parse_hex_string(data: &str, bytes: &mut Vec<u8>, offset: usize) -> DxfResult<()> {
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
        } else {
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

#[cfg(test)]
#[allow(dead_code)]
pub mod tests {
    use crate::code_pair_iter::DirectCodePairIter;
    use crate::*;
    use std::io::{BufRead, BufReader, Cursor, Seek, SeekFrom};

    pub fn unwrap_drawing(result: DxfResult<Drawing>) -> Drawing {
        match result {
            Ok(drawing) => drawing,
            Err(e) => panic!("unable to load drawing: {:?}: {}", e, e),
        }
    }

    pub fn drawing_from_pairs(pairs: Vec<CodePair>) -> Drawing {
        println!("reading from pairs: {:?}", pairs);
        let iter = DirectCodePairIter::new(pairs);
        let iter = Box::new(iter);
        unwrap_drawing(Drawing::load_from_iter(iter))
    }

    pub fn parse_drawing(s: &str) -> Drawing {
        unwrap_drawing(Drawing::load(&mut s.as_bytes()))
    }

    pub fn from_section_pairs(section: &str, body: Vec<CodePair>) -> Drawing {
        let mut pairs = vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, section),
        ];
        for pair in body {
            pairs.push(pair);
        }
        pairs.push(CodePair::new_str(0, "ENDSEC"));
        pairs.push(CodePair::new_str(0, "EOF"));
        drawing_from_pairs(pairs)
    }

    pub fn from_section(section: &str, body: Vec<CodePair>) -> Drawing {
        let mut pairs = vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, section),
        ];
        for pair in body {
            pairs.push(pair);
        }
        pairs.push(CodePair::new_str(0, "ENDSEC"));
        pairs.push(CodePair::new_str(0, "EOF"));
        drawing_from_pairs(pairs)
    }

    pub fn to_test_string(drawing: &Drawing) -> String {
        let mut buf = Cursor::new(vec![]);
        drawing.save(&mut buf).ok().unwrap();
        buf.seek(SeekFrom::Start(0)).ok().unwrap();
        let reader = BufReader::new(&mut buf);
        let contents = reader
            .lines()
            .map(|l| l.unwrap())
            .fold(String::new(), |a, l| a + l.as_str() + "\r\n");
        println!("{}", contents); // will only be displayed on the console if the test fails
        contents
    }

    pub fn to_binary(drawing: &Drawing) -> Vec<u8> {
        let mut buf = Cursor::new(vec![]);
        drawing.save_binary(&mut buf).ok().unwrap();
        buf.seek(SeekFrom::Start(0)).ok().unwrap();
        buf.into_inner()
    }

    pub fn assert_contains(drawing: &Drawing, contents: String) {
        let actual = to_test_string(drawing);
        assert!(actual.contains(&contents));
    }

    fn try_find_index<T>(superset: &[T], subset: &[T]) -> Option<usize>
    where
        T: PartialEq,
    {
        let min_index = 0usize;
        let max_index = superset.len() - subset.len();
        for candidate_base_index in min_index..=max_index {
            let mut test_index = 0;
            while test_index < subset.len() {
                if superset[candidate_base_index + test_index] != subset[test_index] {
                    break;
                }
                test_index += 1;
            }
            if test_index == subset.len() {
                return Some(candidate_base_index);
            }
        }

        None
    }

    pub fn assert_vec_contains<T>(actual: &[T], expected: &[T])
    where
        T: PartialEq,
    {
        let actual_index = try_find_index(actual, expected);
        assert!(actual_index.is_some());
    }

    pub fn assert_contains_pairs(drawing: &Drawing, expected: Vec<CodePair>) {
        let actual = drawing.code_pairs().ok().unwrap();
        println!("checking pairs:");
        for pair in &actual {
            println!("{:?}", pair);
        }
        let actual_index = try_find_index(&actual, &expected);
        assert!(actual_index.is_some());
    }

    pub fn assert_not_contains(drawing: &Drawing, contents: String) {
        let actual = to_test_string(drawing);
        assert!(!actual.contains(&contents));
    }

    pub fn assert_not_contains_pairs(drawing: &Drawing, not_expected: Vec<CodePair>) {
        let actual = drawing.code_pairs().ok().unwrap();
        println!("checking pairs:");
        for pair in &actual {
            println!("{:?}", pair);
        }
        let actual_index = try_find_index(&actual, &not_expected);
        assert!(actual_index.is_none());
    }
}
