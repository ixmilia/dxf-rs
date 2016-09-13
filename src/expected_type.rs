// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

#[derive(PartialEq)]
pub enum ExpectedType {
    Boolean,
    Integer,
    Long,
    Short,
    Double,
    Str,
}

pub fn get_expected_type(code: i32) -> ::std::option::Option<ExpectedType> {
    match code {
        0...9 => Some(ExpectedType::Str),
        10...39 => Some(ExpectedType::Double),
        40...59 => Some(ExpectedType::Double),
        60...79 => Some(ExpectedType::Short),
        90...99 => Some(ExpectedType::Integer),
        100...102 => Some(ExpectedType::Str),
        105 => Some(ExpectedType::Str),
        110...119 => Some(ExpectedType::Double),
        120...129 => Some(ExpectedType::Double),
        130...139 => Some(ExpectedType::Double),
        140...149 => Some(ExpectedType::Double),
        160...169 => Some(ExpectedType::Long),
        170...179 => Some(ExpectedType::Short),
        210...239 => Some(ExpectedType::Double),
        270...279 => Some(ExpectedType::Short),
        280...289 => Some(ExpectedType::Short),
        290...299 => Some(ExpectedType::Boolean),
        300...309 => Some(ExpectedType::Str),
        310...319 => Some(ExpectedType::Str),
        320...329 => Some(ExpectedType::Str),
        330...369 => Some(ExpectedType::Str),
        370...379 => Some(ExpectedType::Short),
        380...389 => Some(ExpectedType::Short),
        390...399 => Some(ExpectedType::Str),
        400...409 => Some(ExpectedType::Short),
        410...419 => Some(ExpectedType::Str),
        420...429 => Some(ExpectedType::Integer),
        430...439 => Some(ExpectedType::Str),
        440...449 => Some(ExpectedType::Integer),
        450...459 => Some(ExpectedType::Long),
        460...469 => Some(ExpectedType::Double),
        470...479 => Some(ExpectedType::Str),
        480...481 => Some(ExpectedType::Str),
        999 => Some(ExpectedType::Str),
        1000...1009 => Some(ExpectedType::Str),
        1010...1059 => Some(ExpectedType::Double),
        1060...1070 => Some(ExpectedType::Short),
        1071 => Some(ExpectedType::Integer),
        _ => None,
    }
}

#[allow(dead_code)] // only used in build.rs
pub fn get_reader_function(typ: &ExpectedType) -> &str {
    match typ {
        &ExpectedType::Boolean => "assert_bool",
        &ExpectedType::Integer => "assert_i32",
        &ExpectedType::Long => "assert_i64",
        &ExpectedType::Short => "assert_i16",
        &ExpectedType::Double => "assert_f64",
        &ExpectedType::Str => "assert_string",
    }
}

#[allow(dead_code)] // only used in build.rs
pub fn get_code_pair_type(typ: ExpectedType) -> String {
    match typ {
        ExpectedType::Boolean => String::from("bool"),
        ExpectedType::Integer => String::from("i32"),
        ExpectedType::Long => String::from("i64"),
        ExpectedType::Short => String::from("i16"),
        ExpectedType::Double => String::from("f64"),
        ExpectedType::Str => String::from("string"),
    }
}
