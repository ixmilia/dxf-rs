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

pub fn get_expected_type(code: i32) -> ::std::io::Result<ExpectedType> {
    match code {
        0...9 => Ok(ExpectedType::Str),
        10...39 => Ok(ExpectedType::Double),
        40...59 => Ok(ExpectedType::Double),
        60...79 => Ok(ExpectedType::Short),
        90...99 => Ok(ExpectedType::Integer),
        100...102 => Ok(ExpectedType::Str),
        105 => Ok(ExpectedType::Str),
        110...119 => Ok(ExpectedType::Double),
        120...129 => Ok(ExpectedType::Double),
        130...139 => Ok(ExpectedType::Double),
        140...149 => Ok(ExpectedType::Double),
        160...169 => Ok(ExpectedType::Long),
        170...179 => Ok(ExpectedType::Short),
        210...239 => Ok(ExpectedType::Double),
        270...279 => Ok(ExpectedType::Short),
        280...289 => Ok(ExpectedType::Short),
        290...299 => Ok(ExpectedType::Boolean),
        300...309 => Ok(ExpectedType::Str),
        310...319 => Ok(ExpectedType::Str),
        320...329 => Ok(ExpectedType::Str),
        330...369 => Ok(ExpectedType::Str),
        370...379 => Ok(ExpectedType::Short),
        380...389 => Ok(ExpectedType::Short),
        390...399 => Ok(ExpectedType::Str),
        400...409 => Ok(ExpectedType::Short),
        410...419 => Ok(ExpectedType::Str),
        420...429 => Ok(ExpectedType::Integer),
        430...439 => Ok(ExpectedType::Str),
        440...449 => Ok(ExpectedType::Integer),
        450...459 => Ok(ExpectedType::Long),
        460...469 => Ok(ExpectedType::Double),
        470...479 => Ok(ExpectedType::Str),
        480...481 => Ok(ExpectedType::Str),
        999 => Ok(ExpectedType::Str),
        1000...1009 => Ok(ExpectedType::Str),
        1010...1059 => Ok(ExpectedType::Double),
        1060...1070 => Ok(ExpectedType::Short),
        1071 => Ok(ExpectedType::Integer),
        _ => Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, format!("unsupported code {}", code))),
    }
}

#[allow(dead_code)] // only used in build.rs
pub fn get_reader_function(typ: &ExpectedType) -> &str {
    match typ {
        &ExpectedType::Boolean => "bool_value",
        &ExpectedType::Integer => "int_value",
        &ExpectedType::Long => "long_value",
        &ExpectedType::Short => "short_value",
        &ExpectedType::Double => "double_value",
        &ExpectedType::Str => "string_value",
    }
}

#[allow(dead_code)] // only used in build.rs
pub fn get_code_pair_type(typ: ExpectedType) -> String {
    match typ {
        ExpectedType::Boolean => String::from("bool"),
        ExpectedType::Integer => String::from("int"),
        ExpectedType::Long => String::from("long"),
        ExpectedType::Short => String::from("short"),
        ExpectedType::Double => String::from("double"),
        ExpectedType::Str => String::from("string"),
    }
}
