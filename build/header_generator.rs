// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xmltree;
use self::xmltree::Element;

use ::ExpectedType;

use xml_helpers::*;
use other_helpers::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::Iterator;

pub fn generate_header() {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

// types from `lib.rs`.
use ::{
    CodePair,
    Color,
    DxfError,
    DxfResult,
    LineWeight,
    Point,
    Vector,
};
use ::code_pair_writer::CodePairWriter;
use ::helper_functions::*;

use enums::*;
use enum_primitive::FromPrimitive;

use std::io::Write;

extern crate chrono;
use self::chrono::{DateTime, Local, UTC};

extern crate time;
use self::time::Duration;

extern crate uuid;
use self::uuid::Uuid;
".trim_left());
    generate_struct(&mut fun, &element);

    generate_default(&mut fun, &element);

    fun.push_str("impl Header {\n");
    generate_flags(&mut fun, &element);
    generate_set_defaults(&mut fun, &element);
    generate_set_header_value(&mut fun, &element);
    generate_add_code_pairs(&mut fun, &element);
    fun.push_str("}\n");

    let mut file = File::create("src/generated/header.rs").ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_struct(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("/// Contains common properties for the DXF file.\n");
    fun.push_str("pub struct Header {\n");
    for v in &element.children {
        let field_name = field(v);
        if !seen_fields.contains(&field_name) {
            seen_fields.insert(field_name.clone());
            let mut comment = format!("The ${} header variable.  {}", name(&v), comment(&v));
            if !min_version(&v).is_empty() {
                comment.push_str(&format!("  Minimum AutoCAD version: {}.", min_version(&v)));
            }
            if !max_version(&v).is_empty() {
                comment.push_str(&format!("  Maximum AutoCAD version: {}.", max_version(&v)));
            }
            fun.push_str(&format!("    /// {}\n", comment));
            fun.push_str(&format!("    pub {field}: {typ},\n", field=field(&v), typ=typ(&v)));
        }
    }

    fun.push_str("}\n");
    fun.push_str("\n");
}

fn generate_default(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("impl Default for Header {\n");
    fun.push_str("    fn default() -> Self {\n");
    fun.push_str("        Header {\n");
    for v in &element.children {
        if !seen_fields.contains(&field(&v)) {
            seen_fields.insert(field(&v));
            fun.push_str(&format!("            {field}: {default_value}, // ${name}\n", field=field(&v), default_value=default_value(&v), name=name(&v)));
        }
    }

    fun.push_str("        }\n");
    fun.push_str("    }\n");
    fun.push_str("}\n");
    fun.push_str("\n");
}

fn generate_flags(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    for v in &element.children {
        if !seen_fields.contains(&field(&v)) {
            seen_fields.insert(field(&v));
            if v.children.len() > 0 {
                fun.push_str(&format!("    // {} flags\n", field(&v)));
            }
            for f in &v.children {
                let mut comment = format!("{}", comment(&f));
                if !min_version(&v).is_empty() {
                    comment.push_str(&format!("  Minimum AutoCAD version: {}.", min_version(&v)));
                }
                if !max_version(&v).is_empty() {
                    comment.push_str(&format!("  Maximum AutoCAD version: {}.", max_version(&v)));
                }
                fun.push_str(&format!("    /// {}\n", comment));
                fun.push_str(&format!("    pub fn get_{flag}(&self) -> bool {{\n", flag=name(&f)));
                fun.push_str(&format!("        self.{field} & {mask} != 0\n", field=field(&v), mask=mask(&f)));
                fun.push_str("    }\n");
                fun.push_str(&format!("    /// {}\n", comment));
                fun.push_str(&format!("    pub fn set_{flag}(&mut self, val: bool) {{\n", flag=name(&f)));
                fun.push_str(&format!("        if val {{\n"));
                fun.push_str(&format!("            self.{field} |= {mask};\n", field=field(&v), mask=mask(&f)));
                fun.push_str("        }\n");
                fun.push_str("        else {\n");
                fun.push_str(&format!("            self.{field} &= !{mask};\n", field=field(&v), mask=mask(&f)));
                fun.push_str("        }\n");
                fun.push_str("    }\n");
            }
        }
    }
}

fn generate_set_defaults(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("    /// Sets the default values on the header.\n");
    fun.push_str("    pub fn set_defaults(&mut self) {\n");
    for v in &element.children {
        if !seen_fields.contains(&field(&v)) {
            seen_fields.insert(field(&v));
            fun.push_str(&format!("        self.{field} = {default_value}; // ${name}\n", field=field(&v), default_value=default_value(&v), name=name(&v)));
        }
    }

    fun.push_str("    }\n");
}

fn generate_set_header_value(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("    pub(crate) fn set_header_value(&mut self, variable: &str, pair: &CodePair) -> DxfResult<()> {\n");
    fun.push_str("        match variable {\n");
    for v in &element.children {
        if !seen_fields.contains(&field(&v)) {
            seen_fields.insert(field(&v));
            fun.push_str(&format!("            \"${name}\" => {{", name=name(&v)));
            let variables_with_name: Vec<&Element> = element.children.iter().filter(|&vv| name(&vv) == name(&v)).collect();
            if variables_with_name.len() == 1 {
                // only one variable with that name
                fun.push_str(" ");
                if code(&v) < 0 {
                    fun.push_str(&format!("self.{field}.set(&pair)?;", field=field(&v)));
                }
                else {
                    let read_cmd = get_read_command(&v);
                    fun.push_str(&format!("verify_code(&pair, {code})?; self.{field} = {cmd};", code=code(&v), field=field(&v), cmd=read_cmd));
                }

                fun.push_str(" ");
            }
            else {
                // multiple variables with that name
                fun.push_str("\n");
                fun.push_str("                match pair.code {\n");
                let expected_codes: Vec<i32> = variables_with_name.iter().map(|&vv| code(&vv)).collect();
                for v in &variables_with_name {
                    let read_cmd = get_read_command(&v);
                    fun.push_str(&format!("                    {code} => self.{field} = {cmd},\n", code=code(&v), field=field(&v), cmd=read_cmd));
                }
                fun.push_str(&format!("                    _ => return Err(DxfError::UnexpectedCodePair(pair.clone(), String::from(\"expected code {:?}\"))),\n", expected_codes));
                fun.push_str("                }\n");
                fun.push_str("            ");
            }

            fun.push_str("},\n");
        }
    }
    fun.push_str("            _ => (),\n");
    fun.push_str("        }\n");
    fun.push_str("\n");
    fun.push_str("        Ok(())\n");
    fun.push_str("    }\n");
}

fn get_read_command(element: &Element) -> String {
    let reader_override = reader_override(&element);
    if !reader_override.is_empty() {
        reader_override
    }
    else {
        let expected_type = ExpectedType::get_expected_type(code(element)).unwrap();
        let reader_fun = get_reader_function(&expected_type);
        let converter = if read_converter(&element).is_empty() { String::from("{}") } else { read_converter(&element).clone() };
        converter.replace("{}", &format!("pair.{}()?", reader_fun))
    }
}

fn generate_add_code_pairs(fun: &mut String, element: &Element) {
    fun.push_str("    pub(crate) fn write_code_pairs<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>\n");
    fun.push_str("        where T: Write {\n");
    fun.push_str("\n");
    for v in &element.children {
        // prepare writing predicate
        let mut parts = vec![];
        if !min_version(&v).is_empty() {
            parts.push(format!("self.version >= AcadVersion::{}", min_version(&v)));
        }
        if !max_version(&v).is_empty() {
            parts.push(format!("self.version <= AcadVersion::{}", max_version(&v)));
        }
        if dont_write_default(&v) {
            parts.push(format!("self.{} != {}", field(&v), default_value(&v)));
        }
        let indent = match parts.len() {
            0 => "",
            _ => "    ",
        };

        // write the value
        fun.push_str(&format!("        // ${}\n", name(&v)));
        if parts.len() > 0 {
            fun.push_str(&format!("        if {} {{\n", parts.join(" && ")));
        }
        fun.push_str(&format!("        {indent}writer.write_code_pair(&CodePair::new_str(9, \"${name}\"))?;\n", name=name(&v), indent=indent));
        let write_converter = if write_converter(&v).is_empty() { String::from("{}") } else { write_converter(&v).clone() };
        if code(&v) > 0 {
            let expected_type = get_code_pair_type(ExpectedType::get_expected_type(code(&v)).unwrap());
            let value = write_converter.replace("{}", &format!("self.{}", field(&v)));
            fun.push_str(&format!("        {indent}writer.write_code_pair(&CodePair::new_{typ}({code}, {value}))?;\n",
                code=code(&v),
                value=value,
                typ=expected_type,
                indent=indent));
        }
        else {
            // write a point or vector as it's components
            for i in 0..code(&v).abs() {
                let (code, fld) = match i {
                    0 => (10, "x"),
                    1 => (20, "y"),
                    2 => (30, "z"),
                    _ => panic!("unexpected number of values"),
                };
                let value = write_converter.replace("{}", &format!("self.{}.{}", field(&v), fld));
                fun.push_str(&format!("        {indent}writer.write_code_pair(&CodePair::new_f64({code}, {value}))?;\n",
                    code=code,
                    value=value,
                    indent=indent));
            }
        }
        if parts.len() > 0 {
            fun.push_str("        }\n");
        }

        // newline between values
        fun.push_str("\n");
    }

    fun.push_str("        Ok(())\n");
    fun.push_str("    }\n");
}

fn load_xml() -> Element {
    let file = File::open("spec/HeaderVariablesSpec.xml").unwrap();
    let file = BufReader::new(file);
    Element::parse(file).unwrap()
}

fn dont_write_default(element: &Element) -> bool {
    attr(element, "DontWriteDefault") == "true"
}

fn field(element: &Element) -> String {
    attr(element, "Field")
}

fn mask(element: &Element) -> String {
    attr(element, "Mask")
}

fn read_converter(element: &Element) -> String {
    attr(element, "ReadConverter")
}

fn reader_override(element: &Element) -> String {
    attr(element, "ReaderOverride")
}

fn write_converter(element: &Element) -> String {
    attr(element, "WriteConverter")
}
