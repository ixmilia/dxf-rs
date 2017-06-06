// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xmltree;
use self::xmltree::Element;
use other_helpers::*;
use ::ExpectedType;

pub fn attr(element: &Element, name: &str) -> String {
    match &element.attributes.get(name) {
        &Some(v) => v.clone(),
        &None => String::new(),
    }
}

pub fn allow_multiples(element: &Element) -> bool {
    attr(element, "AllowMultiples") == "true"
}

pub fn comment(element: &Element) -> String {
    attr(element, "Comment")
}

pub fn code(element: &Element) -> i32 {
    attr(element, "Code").parse::<i32>().unwrap()
}

pub fn codes(element: &Element) -> Vec<i32> {
    let code_overrides = attr(&element, "CodeOverrides");
    if code_overrides.is_empty() {
        return vec![code(&element)];
    }
    else {
        return code_overrides.split(",").map(|c| c.parse::<i32>().unwrap()).collect::<Vec<_>>();
    }
}

pub fn default_value(element: &Element) -> String {
    attr(&element, "DefaultValue")
}

pub fn disable_writing_default(element: &Element) -> bool {
    attr(&element, "DisableWritingDefault") == "true"
}

pub fn generate_reader(element: &Element) -> bool {
    attr(&element, "GenerateReader") != "false"
}

pub fn generate_writer(element: &Element) -> bool {
    attr(&element, "GenerateWriter") != "false"
}

pub fn get_field_reader(element: &Element) -> String {
    let expected_type = ExpectedType::get_expected_type(code(&element)).unwrap();
    let reader_fun = get_reader_function(&expected_type);
    let mut read_converter = attr(&element, "ReadConverter");
    if read_converter.is_empty() {
        read_converter = String::from("{}");
    }
    let read_cmd = format!("pair.value.{}()?", reader_fun);
    read_converter.replace("{}", &read_cmd)
}

pub fn min_version(element: &Element) -> String {
    attr(&element, "MinVersion")
}

pub fn max_version(element: &Element) -> String {
    attr(&element, "MaxVersion")
}

pub fn name(element: &Element) -> String {
    attr(element, "Name")
}

pub fn typ(element: &Element) -> String {
    attr(element, "Type")
}

pub fn write_condition(element: &Element) -> String {
    attr(element, "WriteCondition")
}
