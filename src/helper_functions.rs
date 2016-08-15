// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate chrono;
use self::chrono::*;

extern crate uuid;
use self::uuid::Uuid;

use enum_primitive::FromPrimitive;

use ::{DxfCodePair, DxfCodePairValue};
use ::enums::*;

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

pub fn as_uuid(s: String) -> Uuid {
    Uuid::parse_str(s.as_str()).unwrap()
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

pub fn default_if_empty(default: &str) -> Box<Fn(&String) -> String> {
    let default = String::from(default);
    Box::new(move |val| if val == "" { default.clone() } else { val.clone() })
}

pub fn ensure_positive_or_default(default: f64) -> Box<Fn(f64) -> f64> {
    Box::new(move |val| if val <= 0.0 { default } else { val })
}

pub fn clipping_from_bool(b: bool) -> DxfXrefClippingBoundaryVisibility {
    DxfXrefClippingBoundaryVisibility::from_i16(if b { 1 } else { 0 }).unwrap()
}

pub fn bool_from_clipping(c: DxfXrefClippingBoundaryVisibility) -> bool {
    c != DxfXrefClippingBoundaryVisibility::NotDisplayedNotPlotted
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
