use crate::ExpectedType;

pub fn reader_function(typ: &ExpectedType) -> &str {
    match typ {
        &ExpectedType::Boolean => "assert_bool",
        &ExpectedType::Integer => "assert_i32",
        &ExpectedType::Long => "assert_i64",
        &ExpectedType::Short => "assert_i16",
        &ExpectedType::Double => "assert_f64",
        &ExpectedType::Str => "assert_string",
        &ExpectedType::Binary => "assert_binary",
    }
}

pub fn code_pair_type(typ: &ExpectedType) -> String {
    match typ {
        &ExpectedType::Boolean => String::from("bool"),
        &ExpectedType::Integer => String::from("i32"),
        &ExpectedType::Long => String::from("i64"),
        &ExpectedType::Short => String::from("i16"),
        &ExpectedType::Double => String::from("f64"),
        &ExpectedType::Str => String::from("string"),
        &ExpectedType::Binary => String::from("binary"),
    }
}
