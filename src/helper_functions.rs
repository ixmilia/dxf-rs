// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate chrono;
use self::chrono::*;

extern crate uuid;
use self::uuid::Uuid;

use enum_primitive::FromPrimitive;

use ::{Color, DxfError, DxfResult};
use ::enums::*;
use ::tables::Layer;

#[doc(hidden)]
pub fn verify_code(expected: i32, actual: i32) -> DxfResult<()> {
    if expected == actual {
        Ok(())
    }
    else {
        Err(DxfError::UnexpectedCode(actual))
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
pub fn as_u32(s: String) -> DxfResult<u32> {
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
            _ => return Err(DxfError::ParseError),
        }
    }

    Ok(result)
}

#[doc(hidden)]
pub fn as_handle(h: u32) -> String {
    format!("{:X}", h)
}

#[doc(hidden)]
pub fn as_uuid(s: String) -> DxfResult<Uuid> {
    match Uuid::parse_str(s.as_str()) {
        Ok(uuid) => Ok(uuid),
        Err(_) => Err(DxfError::ParseError),
    }
}

#[doc(hidden)]
pub fn as_i16(b: bool) -> i16 {
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
pub fn parse_bool(s: String) -> DxfResult<bool> {
    match parse_i16(s) {
        Ok(0) => Ok(false),
        Ok(_) => Ok(true),
        Err(x) => Err(x),
    }
}

#[doc(hidden)]
pub fn parse_f64(s: String) -> DxfResult<f64> {
    match s.trim().parse::<f64>() {
        Ok(d) => Ok(d),
        Err(e) => Err(DxfError::ParseFloatError(e)),
    }
}

#[doc(hidden)]
pub fn parse_i32(s: String) -> DxfResult<i32> {
    match s.trim().parse::<i32>() {
        Ok(i) => Ok(i),
        Err(e) => Err(DxfError::ParseIntError(e)),
    }
}

#[doc(hidden)]
pub fn parse_i64(s: String) -> DxfResult<i64> {
    match s.trim().parse::<i64>() {
        Ok(l) => Ok(l),
        Err(e) => Err(DxfError::ParseIntError(e)),
    }
}

#[doc(hidden)]
pub fn parse_i16(s: String) -> DxfResult<i16> {
    match s.trim().parse::<i16>() {
        Ok(s) => Ok(s),
        Err(e) => Err(DxfError::ParseIntError(e)),
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
