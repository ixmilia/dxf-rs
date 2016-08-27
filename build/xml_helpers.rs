// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xmltree;
use self::xmltree::Element;

use ::{get_expected_type, get_reader_function};

pub fn attr(element: &Element, name: &str) -> String {
    match &element.attributes.get(name) {
        &Some(v) => v.clone(),
        &None => String::new(),
    }
}

pub fn allow_multiples(element: &Element) -> bool {
    attr(element, "AllowMultiples") == "true"
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

pub fn generate_reader(element: &Element) -> bool {
    attr(&element, "GenerateReader") != "false"
}

pub fn generate_writer(element: &Element) -> bool {
    attr(&element, "GenerateWriter") != "false"
}

pub fn get_field_reader(element: &Element) -> String {
    let expected_type = get_expected_type(code(&element)).ok().unwrap();
    let reader_fun = get_reader_function(&expected_type);
    let mut read_converter = attr(&element, "ReadConverter");
    if read_converter.is_empty() {
        read_converter = String::from("{}");
    }
    let read_cmd = format!("{reader}(&pair.value)", reader=reader_fun);
    read_converter.replace("{}", read_cmd.as_str())
}

pub fn name(element: &Element) -> String {
    attr(element, "Name")
}
