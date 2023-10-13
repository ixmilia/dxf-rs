use crate::ExpectedType;

pub fn reader_function(typ: &ExpectedType) -> &str {
    use ExpectedType::*;
    match *typ {
        Boolean => "assert_bool",
        Integer => "assert_i32",
        Long => "assert_i64",
        Short => "assert_i16",
        Double => "assert_f64",
        Str => "assert_string",
        Binary => "assert_binary",
    }
}

pub fn code_pair_type(typ: &ExpectedType) -> String {
    use ExpectedType::*;
    match *typ {
        Boolean => String::from("bool"),
        Integer => String::from("i32"),
        Long => String::from("i64"),
        Short => String::from("i16"),
        Double => String::from("f64"),
        Str => String::from("string"),
        Binary => String::from("binary"),
    }
}
