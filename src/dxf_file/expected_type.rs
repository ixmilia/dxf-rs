// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

enum ExpectedType {
    Boolean,
    Integer,
    Long,
    Short,
    Double,
    Str,
}

fn get_expected_type(code: i32) -> ExpectedType {
    match code {
        0...9 => ExpectedType::Str,
        10...39 => ExpectedType::Double,
        40...59 => ExpectedType::Double,
        60...79 => ExpectedType::Short,
        90...99 => ExpectedType::Integer,
        100...102 => ExpectedType::Str,
        105 => ExpectedType::Str,
        110...119 => ExpectedType::Double,
        120...129 => ExpectedType::Double,
        130...139 => ExpectedType::Double,
        140...149 => ExpectedType::Double,
        160...169 => ExpectedType::Long,
        170...179 => ExpectedType::Short,
        210...239 => ExpectedType::Double,
        270...279 => ExpectedType::Short,
        280...289 => ExpectedType::Short,
        290...299 => ExpectedType::Boolean,
        300...309 => ExpectedType::Str,
        310...319 => ExpectedType::Str,
        320...329 => ExpectedType::Str,
        330...369 => ExpectedType::Str,
        370...379 => ExpectedType::Short,
        380...389 => ExpectedType::Short,
        390...399 => ExpectedType::Str,
        400...409 => ExpectedType::Short,
        410...419 => ExpectedType::Str,
        420...429 => ExpectedType::Integer,
        430...439 => ExpectedType::Str,
        440...449 => ExpectedType::Integer,
        450...459 => ExpectedType::Long,
        460...469 => ExpectedType::Double,
        470...479 => ExpectedType::Str,
        480...481 => ExpectedType::Str,
        999 => ExpectedType::Str,
        1000...1009 => ExpectedType::Str,
        1010...1059 => ExpectedType::Double,
        1060...1070 => ExpectedType::Short,
        1071 => ExpectedType::Integer,
        _ => panic!("unsupported code {}", code),
    }
}

#[allow(dead_code)] // only used in build.rs
fn get_reader_function(typ: &ExpectedType) -> &str {
    match typ {
        &ExpectedType::Boolean => "bool_value",
        &ExpectedType::Integer => "int_value",
        &ExpectedType::Long => "long_value",
        &ExpectedType::Short => "short_value",
        &ExpectedType::Double => "double_value",
        &ExpectedType::Str => "string_value",
    }
}
