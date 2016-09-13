// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate chrono;
use self::chrono::*;

extern crate uuid;
use self::uuid::Uuid;

use std::io;
use enum_primitive::FromPrimitive;

use ::Color;
use ::enums::*;
use ::tables::Layer;

#[doc(hidden)]
pub fn verify_code(expected: i32, actual: i32) -> io::Result<()> {
    if expected == actual {
        Ok(())
    }
    else {
        Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected code {} but got {}", expected, actual)))
    }
}

#[doc(hidden)]
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

#[doc(hidden)]
pub fn as_datetime_local(date: f64) -> DateTime<Local> {
    // dates are represented as the fractional number of days elapsed since December 31, 1899.
    if date == 0.0 {
        Local.ymd(1900, 1, 1).and_hms(0, 0, 0)
    }
    else {
        Local.ymd(1900, 1, 1).and_hms(0, 0, 0) + f64_to_adjusted_duration(date)
    }
}

#[doc(hidden)]
pub fn as_datetime_utc(date: f64) -> DateTime<UTC> {
    // dates are represented as the fractional number of days elapsed since December 31, 1899.
    if date == 0.0 {
        UTC.ymd(1900, 1, 1).and_hms(0, 0, 0)
    }
    else {
        UTC.ymd(1900, 1, 1).and_hms(0, 0, 0) + f64_to_adjusted_duration(date)
    }
}

#[doc(hidden)]
pub fn as_double_local(date: DateTime<Local>) -> f64 {
    let epoch = Local.ymd(1900, 1, 1).and_hms(0, 0, 0);
    let duration = date - epoch;
    (duration.num_seconds() as f64 / 24.0 / 60.0 / 60.0) + 2415021f64
}

#[doc(hidden)]
pub fn as_double_utc(date: DateTime<UTC>) -> f64 {
    let epoch = UTC.ymd(1900, 1, 1).and_hms(0, 0, 0);
    let duration = date - epoch;
    (duration.num_seconds() as f64 / 24.0 / 60.0 / 60.0) + 2415021f64
}

#[doc(hidden)]
pub fn duration_as_double(duration: Duration) -> f64 {
    duration.num_seconds() as f64
}

#[doc(hidden)]
pub fn as_duration(d: f64) -> Duration {
    Duration::seconds(d as i64)
}

#[doc(hidden)]
pub fn as_u32(s: String) -> io::Result<u32> {
    let mut result = 0;
    for c in s.chars() {
        match c {
            '0' => result = result * 16,
            '1' => result = result * 16 + 1,
            '2' => result = result * 16 + 2,
            '3' => result = result * 16 + 3,
            '4' => result = result * 16 + 4,
            '5' => result = result * 16 + 5,
            '6' => result = result * 16 + 6,
            '7' => result = result * 16 + 7,
            '8' => result = result * 16 + 8,
            '9' => result = result * 16 + 9,
            'A' | 'a' => result = result * 16 + 10,
            'B' | 'b' => result = result * 16 + 11,
            'C' | 'c' => result = result * 16 + 12,
            'D' | 'd' => result = result * 16 + 13,
            'E' | 'e' => result = result * 16 + 14,
            'F' | 'f' => result = result * 16 + 15,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid hex character")),
        }
    }

    Ok(result)
}

#[doc(hidden)]
pub fn as_handle(h: u32) -> String {
    format!("{:X}", h)
}

#[doc(hidden)]
pub fn as_uuid(s: String) -> io::Result<Uuid> {
    match Uuid::parse_str(s.as_str()) {
        Ok(uuid) => Ok(uuid),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[doc(hidden)]
pub fn as_short(b: bool) -> i16 {
    if b { 1 } else { 0 }
}

#[doc(hidden)]
pub fn uuid_string(u: &Uuid) -> String {
    format!("{}", u)
}

#[doc(hidden)]
pub fn default_if_empty(val: &String, default: &str) -> String {
    if val.is_empty() { String::from(default) } else { val.clone() }
}

#[doc(hidden)]
pub fn ensure_positive_or_default(val: f64, default: f64) -> f64 {
    if val <= 0.0 { default } else { val }
}

#[doc(hidden)]
pub fn ensure_positive_or_default_i32(val: i32, default: i32) -> i32 {
    if val <= 0 { default } else { val }
}

#[doc(hidden)]
pub fn ensure_positive_or_default_i16(val: i16, default: i16) -> i16 {
    if val <= 0 { default } else { val }
}

#[doc(hidden)]
pub fn get_writable_linetype_name<'a>(val: &'a str) -> &'a str {
    if val.is_empty() { "CONTINUOUS" } else { val }
}

#[doc(hidden)]
pub fn clipping_from_bool(b: bool) -> Option<XrefClippingBoundaryVisibility> {
    XrefClippingBoundaryVisibility::from_i16(if b { 1 } else { 0 })
}

#[doc(hidden)]
pub fn bool_from_clipping(c: XrefClippingBoundaryVisibility) -> bool {
    c != XrefClippingBoundaryVisibility::NotDisplayedNotPlotted
}

#[doc(hidden)]
pub fn version_from_string(v: String) -> io::Result<AcadVersion> {
    match &*v {
        "MC0.0" => Ok(AcadVersion::Version_1_0),
        "AC1.2" => Ok(AcadVersion::Version_1_2),
        "AC1.40" => Ok(AcadVersion::Version_1_40),
        "AC1.50" => Ok(AcadVersion::Version_2_05),
        "AC2.10" => Ok(AcadVersion::Version_2_10),
        "AC2.21" => Ok(AcadVersion::Version_2_21),
        "AC2.22" => Ok(AcadVersion::Version_2_22),
        "AC1001" => Ok(AcadVersion::Version_2_22),
        "AC1002" => Ok(AcadVersion::Version_2_5),
        "AC1003" => Ok(AcadVersion::Version_2_6),
        "AC1004" => Ok(AcadVersion::R9),
        "AC1006" => Ok(AcadVersion::R10),
        "AC1009" => Ok(AcadVersion::R12),
        "AC1011" => Ok(AcadVersion::R13),
        "AC1012" => Ok(AcadVersion::R13),
        "AC1014" => Ok(AcadVersion::R14),
        "14" => Ok(AcadVersion::R14),
        "14.01" => Ok(AcadVersion::R14),
        "AC1015" => Ok(AcadVersion::R2000),
        "15.0" => Ok(AcadVersion::R2000),
        "15.05" => Ok(AcadVersion::R2000),
        "15.06" => Ok(AcadVersion::R2000),
        "AC1018" => Ok(AcadVersion::R2004),
        "16.0" => Ok(AcadVersion::R2004),
        "16.1" => Ok(AcadVersion::R2004),
        "16.2" => Ok(AcadVersion::R2004),
        "AC1021" => Ok(AcadVersion::R2007),
        "17.0" => Ok(AcadVersion::R2007),
        "17.1" => Ok(AcadVersion::R2007),
        "17.2" => Ok(AcadVersion::R2007),
        "AC1024" => Ok(AcadVersion::R2010),
        "18.0" => Ok(AcadVersion::R2010),
        "18.1" => Ok(AcadVersion::R2010),
        "18.2" => Ok(AcadVersion::R2010),
        "AC1027" => Ok(AcadVersion::R2013),
        "19.0" => Ok(AcadVersion::R2013),
        "19.1" => Ok(AcadVersion::R2013),
        "19.2" => Ok(AcadVersion::R2013),
        "19.3" => Ok(AcadVersion::R2013),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("unsupported version {}", v))),
    }
}

#[doc(hidden)]
pub fn string_from_version(v: &AcadVersion) -> String {
    String::from(
        match v {
            &AcadVersion::Version_1_0 => "MC0.0",
            &AcadVersion::Version_1_2 => "AC1.2",
            &AcadVersion::Version_1_40 => "AC1.40",
            &AcadVersion::Version_2_05 => "AC1.50",
            &AcadVersion::Version_2_10 => "AC2.10",
            &AcadVersion::Version_2_21 => "AC2.21",
            &AcadVersion::Version_2_22 => "AC2.22",
            &AcadVersion::Version_2_5 => "AC1002",
            &AcadVersion::Version_2_6 => "AC1003",
            &AcadVersion::R9 => "AC1004",
            &AcadVersion::R10 => "AC1006",
            &AcadVersion::R11 => "AC1009",
            &AcadVersion::R12 => "AC1009",
            &AcadVersion::R13 => "AC1012",
            &AcadVersion::R14 => "AC1014",
            &AcadVersion::R2000 => "AC1015",
            &AcadVersion::R2004 => "AC1018",
            &AcadVersion::R2007 => "AC1021",
            &AcadVersion::R2010 => "AC1024",
            &AcadVersion::R2013 => "AC1027",
    })
}

#[doc(hidden)]
pub fn parse_bool(s: String) -> io::Result<bool> {
    match parse_short(s) {
        Ok(0) => Ok(false),
        Ok(_) => Ok(true),
        Err(x) => Err(io::Error::new(io::ErrorKind::InvalidData, x)),
    }
}

#[doc(hidden)]
pub fn parse_double(s: String) -> io::Result<f64> {
    match s.parse::<f64>() {
        Ok(d) => Ok(d),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[doc(hidden)]
pub fn parse_int(s: String) -> io::Result<i32> {
    match s.parse::<i32>() {
        Ok(i) => Ok(i),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[doc(hidden)]
pub fn parse_long(s: String) -> io::Result<i64> {
    match s.parse::<i64>() {
        Ok(l) => Ok(l),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[doc(hidden)]
pub fn parse_short(s: String) -> io::Result<i16> {
    match s.parse::<i16>() {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[doc(hidden)]
pub fn trim_trailing_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

#[doc(hidden)]
pub fn read_color_value(layer: &mut Layer, color: i16) -> Color {
    layer.is_layer_on = color >= 0;
    Color::from_raw_value(color.abs())
}
