// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate chrono;
use self::chrono::*;

extern crate uuid;
use self::uuid::Uuid;

use std::io;
use enum_primitive::FromPrimitive;

use ::DxfCodePairValue;
use ::enums::*;

pub fn bool_value(value: &DxfCodePairValue) -> bool {
    match value {
        &DxfCodePairValue::Boolean(b) => b,
        _ => panic!("this should never have happened, please file a bug"),
    }
}

pub fn long_value(value: &DxfCodePairValue) -> i64 {
    match value {
        &DxfCodePairValue::Long(l) => l,
        _ => panic!("this should never have happened, please file a bug"),
    }
}

pub fn double_value(value: &DxfCodePairValue) -> f64 {
    match value {
        &DxfCodePairValue::Double(f) => f,
        _ => panic!("this should never have happened, please file a bug"),
    }
}

pub fn string_value(value: &DxfCodePairValue) -> String {
    match value {
        &DxfCodePairValue::Str(ref s) => s.clone(),
        _ => panic!("this should never have happened, please file a bug"),
    }
}

pub fn short_value(value: &DxfCodePairValue) -> i16 {
    match value {
        &DxfCodePairValue::Short(s) => s,
        _ => panic!("this should never have happened, please file a bug"),
    }
}

pub fn verify_code(expected: i32, actual: i32) -> io::Result<()> {
    if expected == actual {
        Ok(())
    }
    else {
        Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected code {} but got {}", expected, actual)))
    }
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

pub fn as_double_local(date: DateTime<Local>) -> f64 {
    let epoch = Local.ymd(1900, 1, 1).and_hms(0, 0, 0);
    let duration = date - epoch;
    (duration.num_seconds() as f64 / 24.0 / 60.0 / 60.0) + 2415021f64
}

pub fn as_double_utc(date: DateTime<UTC>) -> f64 {
    let epoch = UTC.ymd(1900, 1, 1).and_hms(0, 0, 0);
    let duration = date - epoch;
    (duration.num_seconds() as f64 / 24.0 / 60.0 / 60.0) + 2415021f64
}

pub fn duration_as_double(duration: Duration) -> f64 {
    duration.num_seconds() as f64
}

pub fn as_duration(_d: f64) -> Duration {
    unimplemented!()
}

pub fn as_handle(_s: String) -> u32 {
    unimplemented!()
}

pub fn as_uuid(s: String) -> io::Result<Uuid> {
    match Uuid::parse_str(s.as_str()) {
        Ok(uuid) => Ok(uuid),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

pub fn as_short(b: bool) -> i16 {
    if b { 1 } else { 0 }
}

pub fn u32_handle(h: &u32) -> String {
    format!("{:X}", h)
}

pub fn uuid_string(_u: &Uuid) -> String {
    unimplemented!()
}

pub fn default_if_empty(val: &String, default: &str) -> String {
    if val.is_empty() { String::from(default) } else { val.clone() }
}

pub fn ensure_positive_or_default(val: f64, default: f64) -> f64 {
    if val <= 0.0 { default } else { val }
}

pub fn clipping_from_bool(b: bool) -> Option<DxfXrefClippingBoundaryVisibility> {
    DxfXrefClippingBoundaryVisibility::from_i16(if b { 1 } else { 0 })
}

pub fn bool_from_clipping(c: DxfXrefClippingBoundaryVisibility) -> bool {
    c != DxfXrefClippingBoundaryVisibility::NotDisplayedNotPlotted
}

pub fn version_from_string(v: String) -> io::Result<DxfAcadVersion> {
    match v.as_str() {
        "MC0.0" => Ok(DxfAcadVersion::Version_1_0),
        "AC1.2" => Ok(DxfAcadVersion::Version_1_2),
        "AC1.40" => Ok(DxfAcadVersion::Version_1_40),
        "AC1.50" => Ok(DxfAcadVersion::Version_2_05),
        "AC2.10" => Ok(DxfAcadVersion::Version_2_10),
        "AC2.21" => Ok(DxfAcadVersion::Version_2_21),
        "AC2.22" => Ok(DxfAcadVersion::Version_2_22),
        "AC1001" => Ok(DxfAcadVersion::Version_2_22),
        "AC1002" => Ok(DxfAcadVersion::Version_2_5),
        "AC1003" => Ok(DxfAcadVersion::Version_2_6),
        "AC1004" => Ok(DxfAcadVersion::R9),
        "AC1006" => Ok(DxfAcadVersion::R10),
        "AC1009" => Ok(DxfAcadVersion::R12),
        "AC1011" => Ok(DxfAcadVersion::R13),
        "AC1012" => Ok(DxfAcadVersion::R13),
        "AC1014" => Ok(DxfAcadVersion::R14),
        "14" => Ok(DxfAcadVersion::R14),
        "14.01" => Ok(DxfAcadVersion::R14),
        "AC1015" => Ok(DxfAcadVersion::R2000),
        "15.0" => Ok(DxfAcadVersion::R2000),
        "15.05" => Ok(DxfAcadVersion::R2000),
        "15.06" => Ok(DxfAcadVersion::R2000),
        "AC1018" => Ok(DxfAcadVersion::R2004),
        "16.0" => Ok(DxfAcadVersion::R2004),
        "16.1" => Ok(DxfAcadVersion::R2004),
        "16.2" => Ok(DxfAcadVersion::R2004),
        "AC1021" => Ok(DxfAcadVersion::R2007),
        "17.0" => Ok(DxfAcadVersion::R2007),
        "17.1" => Ok(DxfAcadVersion::R2007),
        "17.2" => Ok(DxfAcadVersion::R2007),
        "AC1024" => Ok(DxfAcadVersion::R2010),
        "18.0" => Ok(DxfAcadVersion::R2010),
        "18.1" => Ok(DxfAcadVersion::R2010),
        "18.2" => Ok(DxfAcadVersion::R2010),
        "AC1027" => Ok(DxfAcadVersion::R2013),
        "19.0" => Ok(DxfAcadVersion::R2013),
        "19.1" => Ok(DxfAcadVersion::R2013),
        "19.2" => Ok(DxfAcadVersion::R2013),
        "19.3" => Ok(DxfAcadVersion::R2013),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, format!("unsupported version {}", v))),
    }
}

pub fn string_from_version(v: &DxfAcadVersion) -> String {
    String::from(
        match v {
            &DxfAcadVersion::Version_1_0 => "MC0.0",
            &DxfAcadVersion::Version_1_2 => "AC1.2",
            &DxfAcadVersion::Version_1_40 => "AC1.40",
            &DxfAcadVersion::Version_2_05 => "AC1.50",
            &DxfAcadVersion::Version_2_10 => "AC2.10",
            &DxfAcadVersion::Version_2_21 => "AC2.21",
            &DxfAcadVersion::Version_2_22 => "AC2.22",
            &DxfAcadVersion::Version_2_5 => "AC1002",
            &DxfAcadVersion::Version_2_6 => "AC1003",
            &DxfAcadVersion::R9 => "AC1004",
            &DxfAcadVersion::R10 => "AC1006",
            &DxfAcadVersion::R11 => "AC1009",
            &DxfAcadVersion::R12 => "AC1009",
            &DxfAcadVersion::R13 => "AC1012",
            &DxfAcadVersion::R14 => "AC1014",
            &DxfAcadVersion::R2000 => "AC1015",
            &DxfAcadVersion::R2004 => "AC1018",
            &DxfAcadVersion::R2007 => "AC1021",
            &DxfAcadVersion::R2010 => "AC1024",
            &DxfAcadVersion::R2013 => "AC1027",
    })
}

pub fn parse_bool(s: String) -> io::Result<bool> {
    match parse_short(s) {
        Ok(0) => Ok(false),
        Ok(_) => Ok(true),
        Err(x) => Err(io::Error::new(io::ErrorKind::InvalidData, x)),
    }
}

pub fn parse_double(s: String) -> io::Result<f64> {
    match s.parse::<f64>() {
        Ok(d) => Ok(d),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

pub fn parse_int(s: String) -> io::Result<i32> {
    match s.parse::<i32>() {
        Ok(i) => Ok(i),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

pub fn parse_long(s: String) -> io::Result<i64> {
    match s.parse::<i64>() {
        Ok(l) => Ok(l),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

pub fn parse_short(s: String) -> io::Result<i16> {
    match s.parse::<i16>() {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

pub fn trim_trailing_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
