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
    let normalized_read_cmd = if element.name == "Pointer" { format!("as_u32({})?", read_cmd) } else { read_cmd };
    read_converter.replace("{}", &normalized_read_cmd)
}

pub fn get_methods_for_pointer_access(pointer: &Element) -> String {
    let mut fun = String::new();
    let typ = attr(&pointer, "Type");
    let sub_type = attr(&pointer, "SubType");
    let normalized_field_name = format!("__{}_handle", name(&pointer));
    let return_type = match (allow_multiples(&pointer), typ.is_empty()) {
        (true, true) => String::from("Vec<DrawingItem<'a>>"),
        (true, false) => format!("Vec<&'a {}>", typ),
        (false, true) => String::from("Option<DrawingItem<'a>>"),
        (false, false) => format!("Option<&'a {}>", typ),
    };
    fun.push_str(&format!("    pub fn get_{name}<'a>(&self, drawing: &'a Drawing) -> {return_type} {{\n", name=name(&pointer), return_type=return_type));
    if !typ.is_empty() {
        if allow_multiples(&pointer) {
            if !sub_type.is_empty() {
                fun.push_str(&format!("        self.{field}.iter().filter_map(|&h| {{\n", field=normalized_field_name));
                fun.push_str(&format!("            match drawing.get_item_by_handle(h) {{\n"));
                fun.push_str(&format!("                Some(DrawingItem::{typ}(val)) => {{\n", typ=typ));
                fun.push_str(&format!("                    match val.specific {{\n"));
                fun.push_str(&format!("                        {typ}Type::{sub_type}(_) => Some(val),\n", typ=typ, sub_type=sub_type));
                fun.push_str(&format!("                        _ => None,\n"));
                fun.push_str(&format!("                    }}\n"));
                fun.push_str(&format!("                }},\n"));
                fun.push_str(&format!("                _ => None,\n"));
                fun.push_str(&format!("            }}\n"));
                fun.push_str("        }).collect()\n");
            }
            else {
                fun.push_str(&format!("        self.{field}.iter().filter_map(|&h| {{\n", field=normalized_field_name));
                fun.push_str(&format!("            match drawing.get_item_by_handle(h) {{\n"));
                fun.push_str(&format!("                Some(DrawingItem::{typ}(val)) => Some(val),\n", typ=typ));
                fun.push_str(&format!("                _ => None,\n"));
                fun.push_str(&format!("            }}\n"));
                fun.push_str("        }).collect()\n");
            }
        }
        else {
            fun.push_str(&format!("        match drawing.get_item_by_handle(self.{field}) {{\n", field=normalized_field_name));
            if !sub_type.is_empty() {
                fun.push_str(&format!("            Some(DrawingItem::{typ}(val)) => {{\n", typ=typ));
                fun.push_str("                match val.specific {\n");
                fun.push_str(&format!("                    {typ}Type::{sub_type}(_) => Some(val),\n", typ=typ, sub_type=sub_type));
                fun.push_str("                    _ => None,\n");
                fun.push_str("                }\n");
                fun.push_str("            },\n");
            }
            else {
                fun.push_str(&format!("            Some(DrawingItem::{typ}(val)) => Some(val),\n", typ=typ));
            }
            fun.push_str("            _ => None,\n");
            fun.push_str("        }\n");
        }
    }
    else {
        if allow_multiples(&pointer) {
            fun.push_str(&format!("        self.{field}.iter().filter_map(|&h| drawing.get_item_by_handle(h)).collect()\n", field=normalized_field_name));
        }
        else {
            fun.push_str(&format!("        drawing.get_item_by_handle(self.{field})\n", field=normalized_field_name));
        }
    }

    fun.push_str("    }\n");
    fun
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
