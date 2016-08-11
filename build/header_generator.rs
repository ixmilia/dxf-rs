// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xml;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use self::xml::reader::{EventReader, XmlEvent};

include!("../src/dxf_file/expected_type.rs");

pub fn generate_header() {
    let variables = gather_variables();
    let mut fun = String::new();
    fun.push_str("use dxf_file::*;\n");
    fun.push_str("use dxf_file::enums::*;\n");
    fun.push_str("use enum_primitive::FromPrimitive;\n");
    fun.push_str("\n");

    fun.push_str("extern crate chrono;\n");
    fun.push_str("use self::chrono::{DateTime, Local, UTC};\n");
    fun.push_str("\n");

    fun.push_str("extern crate time;\n");
    fun.push_str("use self::time::Duration;\n");
    fun.push_str("\n");

    fun.push_str("extern crate uuid;\n");
    fun.push_str("use self::uuid::Uuid;\n");
    fun.push_str("\n");

    generate_struct(&mut fun, &variables);

    fun.push_str("impl DxfHeader {\n");
    generate_new(&mut fun, &variables);
    generate_set_defaults(&mut fun, &variables);
    generate_set_header_value(&mut fun, &variables);
    fun.push_str("}\n");

    let mut file = File::create("src/dxf_file/header_generated.rs").ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_struct(fun: &mut String, variables: &Vec<HeaderVariable>) {
    let mut seen_fields = HashSet::new();
    fun.push_str("pub struct DxfHeader {\n");
    for v in variables {
        if !seen_fields.contains(&v.field) {
            seen_fields.insert(&v.field);
            fun.push_str(format!("    pub {field}: {typ}, // ${name}\n", field=v.field, typ=v.typ, name=v.name).as_str());
        }
    }

    fun.push_str("}\n");
    fun.push_str("\n");
}

fn generate_new(fun: &mut String, variables: &Vec<HeaderVariable>) {
    let mut seen_fields = HashSet::new();
    fun.push_str("    pub fn new() -> DxfHeader {\n");
    fun.push_str("        DxfHeader {\n");
    for v in variables {
        if !seen_fields.contains(&v.field) {
            seen_fields.insert(&v.field);
            fun.push_str(format!("            {field}: {default_value}, // ${name}\n", field=v.field, default_value=v.default_value, name=v.name).as_str());
        }
    }

    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn generate_set_defaults(fun: &mut String, variables: &Vec<HeaderVariable>) {
    let mut seen_fields = HashSet::new();
    fun.push_str("    pub fn set_defaults(&mut self) {\n");
    for v in variables {
        if !seen_fields.contains(&v.field) {
            seen_fields.insert(&v.field);
            fun.push_str(format!("        self.{field} = {default_value}; // ${name}\n", field=v.field, default_value=v.default_value, name=v.name).as_str());
        }
    }

    fun.push_str("    }\n");
}

fn generate_set_header_value(fun: &mut String, variables: &Vec<HeaderVariable>) {
    let mut seen_fields = HashSet::new();
    fun.push_str("    pub fn set_header_value(&mut self, variable: &str, pair: DxfCodePair) {\n");
    fun.push_str("        match variable {\n");
    for v in variables {
        if !seen_fields.contains(&v.field) { // TODO: handle duplicates
            seen_fields.insert(&v.field);
            fun.push_str(format!("            \"${name}\" => {{ ", name=v.name).as_str());
            if v.code < 0 {
                fun.push_str(format!("set_point(&mut self.{field}, &pair);", field=v.field).as_str());
            }
            else {
                let expected_type = get_expected_type(v.code);
                let reader_fun = get_reader_function(&expected_type);
                let mut read_cmd = format!("{}(&pair.value)", reader_fun);
                if v.read_converter != "" {
                    if v.read_converter.starts_with("as ") {
                        read_cmd = format!("{} {}", read_cmd, v.read_converter);
                    }
                    else {
                        // function converter
                        read_cmd = format!("{}({})", v.read_converter, read_cmd);
                        if v.read_converter.contains("::from_i") || v.read_converter.contains("::from_f") { // enum
                            read_cmd = format!("{}.unwrap()", read_cmd);
                        }
                    }
                }
                fun.push_str(format!("verify_code({code}, &pair); self.{field} = {cmd};", code=v.code, field=v.field, cmd=read_cmd).as_str());
            }

            fun.push_str(" },\n");
        }
    }
    fun.push_str("            _ => (),\n");
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn gather_variables() -> Vec<HeaderVariable> {
    let file = File::open("spec/HeaderVariablesSpec.xml").unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let mut header_variables: Vec<HeaderVariable> = vec![];
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.local_name.as_str() {
                    "Variable" => {
                        let mut var = HeaderVariable::new();
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "Name" => var.name = attr.value,
                                "Code" => var.code = attr.value.parse::<i32>().unwrap(),
                                "Type" => var.typ = attr.value,
                                "Field" => var.field = attr.value,
                                "DefaultValue" => var.default_value = attr.value,
                                "ReadConverter" => var.read_converter = attr.value,
                                "WriteConverter" => var.write_converter = attr.value,
                                "Comment" => var.comment = attr.value,
                                "MinVersion" => var.min_version = attr.value,
                                "MaxVersion" => var.max_version = attr.value,
                                "SuppressWriting" => var.suppress_writing = attr.value == "true",
                                "DontWriteDefault" => var.dont_write_default = attr.value == "true",
                                _ => panic!("unexpected attribute: {}", attr.name),
                            }
                        }

                        header_variables.push(var);
                    },
                    "Flag" => {
                        // TODO: process flags
                    },
                    "Spec" => (),
                    _ => panic!("unexpected start element: {}", name)
                }
            },
            Ok(XmlEvent::EndElement { name: _ }) => {

            },
            Err(e) => {
                panic!("unable to read xml: {}", e);
            }
            _ => (),
        }
    }

    header_variables
}

struct HeaderVariable {
    name: String,
    code: i32,
    typ: String,
    field: String,
    default_value: String,
    read_converter: String,
    write_converter: String,
    comment: String,
    min_version: String,
    max_version: String,
    suppress_writing: bool,
    dont_write_default: bool,
}

impl HeaderVariable {
    pub fn new() -> HeaderVariable {
        HeaderVariable {
            name: String::new(),
            code: 0,
            typ: String::new(),
            field: String::new(),
            default_value: String::new(),
            read_converter: String::new(),
            write_converter: String::new(),
            comment: String::new(),
            min_version: String::new(),
            max_version: String::new(),
            suppress_writing: false,
            dont_write_default: false,
        }
    }
}
