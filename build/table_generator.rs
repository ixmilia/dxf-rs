// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xmltree;
use self::xmltree::Element;

use ::{get_code_pair_type, get_expected_type};

use xml_helpers::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
//use std::iter::Iterator;

pub fn generate_tables() {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

extern crate itertools;

use ::{CodePair, CodePairAsciiWriter, Color, Drawing, LineWeight, Point, Vector};
use ::helper_functions::*;

use enums::*;
use enum_primitive::FromPrimitive;

use std::io;
use std::io::Write;

use itertools::PutBack;

// Used to turn Option<T> into io::Result.
macro_rules! try_result {
    ($expr : expr) => (
        match $expr {
            Some(v) => v,
            None => return Err(io::Error::new(io::ErrorKind::InvalidData, \"unexpected enum value\"))
        }
    )
}
".trim_left());
    fun.push_str("\n");
    generate_table_items(&mut fun, &element);
    generate_table_reader(&mut fun, &element);
    generate_table_writer(&mut fun, &element);

    let mut file = File::create("src/tables.rs").ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_table_items(fun: &mut String, element: &Element) {
    for table in &element.children {
        let mut seen_fields = HashSet::new();
        let table_item = &table.children[0];
        fun.push_str(format!("pub struct {name} {{\n", name=name(&table_item)).as_str());
        fun.push_str("    pub name: String,\n");
        fun.push_str("    pub handle: u32,\n");
        fun.push_str("    pub owner_handle: u32,\n");
        for field in &table_item.children {
            let name = name(&field);
            if !seen_fields.contains(&name) {
                seen_fields.insert(name.clone());
                let mut typ = attr(&field, "Type");
                if allow_multiples(&field) {
                    typ = format!("Vec<{}>", typ);
                }
                fun.push_str(format!("    pub {name}: {typ},\n", name=name, typ=typ).as_str());
            }
        }
        fun.push_str("}\n");
        fun.push_str("\n");

        fun.push_str(format!("impl {name} {{\n", name=name(&table_item)).as_str());
        fun.push_str("    pub fn new() -> Self {\n");
        fun.push_str("        Default::default()\n");
        fun.push_str("    }\n");
        fun.push_str("}\n");
        fun.push_str("\n");

        seen_fields.clear();
        fun.push_str(format!("impl Default for {name} {{\n", name=name(&table_item)).as_str());
        fun.push_str("    fn default() -> Self {\n");
        fun.push_str(format!("        {name} {{\n", name=name(&table_item)).as_str());
        fun.push_str("            name: String::new(),\n");
        fun.push_str("            handle: 0,\n");
        fun.push_str("            owner_handle: 0,\n");
        for field in &table_item.children {
            let name = name(&field);
            if !seen_fields.contains(&name) {
                seen_fields.insert(name.clone());
                fun.push_str(format!("            {field}: {default_value},\n", field=name, default_value=attr(&field, "DefaultValue")).as_str());
            }
        }

        fun.push_str("        }\n");
        fun.push_str("    }\n");
        fun.push_str("}\n");
        fun.push_str("\n");
    }
}

fn generate_table_reader(fun: &mut String, element: &Element) {
    fun.push_str("pub fn read_specific_table<I>(drawing: &mut Drawing, iter: &mut PutBack<I>) -> io::Result<()>\n");
    fun.push_str("    where I: Iterator<Item = io::Result<CodePair>> {\n");
    fun.push_str("    match iter.next() {\n");
    fun.push_str("        Some(Ok(pair)) => {\n");
    fun.push_str("            if pair.code != 2 {\n");
    fun.push_str("                return Err(io::Error::new(io::ErrorKind::InvalidData, \"expected table type\"));\n");
    fun.push_str("            }\n");
    fun.push_str("\n");
    fun.push_str("            match string_value(&pair.value).as_str() {\n");

    for table in &element.children {
        fun.push_str(format!("                \"{table_name}\" => try!(read_{collection}(drawing, iter)),\n", table_name=attr(&table, "TypeString"), collection=attr(&table, "Collection")).as_str());
    }

    fun.push_str("                _ => try!(Drawing::swallow_table(iter)),\n");
    fun.push_str("            }\n");
    fun.push_str("        },\n");
    fun.push_str("        Some(Err(e)) => return Err(e),\n");
    fun.push_str("        None => return Err(io::Error::new(io::ErrorKind::InvalidData, \"unexpected end of input\")),\n");
    fun.push_str("    }\n");
    fun.push_str("\n");
    fun.push_str("    Ok(())\n");
    fun.push_str("}\n");
    fun.push_str("\n");

    for table in &element.children {
        let table_item = &table.children[0];

        fun.push_str(format!("fn read_{collection}<I>(drawing: &mut Drawing, iter: &mut PutBack<I>) -> io::Result<()>\n", collection=attr(&table, "Collection")).as_str());
        fun.push_str("    where I: Iterator<Item = io::Result<CodePair>> {\n");
        fun.push_str("    loop {\n");
        fun.push_str("        match iter.next() {\n");
        fun.push_str("            Some(Ok(pair)) => {\n");
        fun.push_str("                if pair.code != 0 {\n");
        fun.push_str("                    return Err(io::Error::new(io::ErrorKind::InvalidData, \"expected table item, new table, or end of section\"));\n");
        fun.push_str("                }\n");
        fun.push_str("\n");
        fun.push_str(format!("                if string_value(&pair.value) != \"{table_type}\" {{\n", table_type=attr(&table, "TypeString")).as_str());
        fun.push_str("                    iter.put_back(Ok(pair));\n");
        fun.push_str("                    break;\n");
        fun.push_str("                }\n");
        fun.push_str("\n");
        fun.push_str(format!("                let mut item = {typ}::new();\n", typ=attr(&table_item, "Name")).as_str());
        fun.push_str("                loop {\n");
        fun.push_str("                    match iter.next() {\n");
        fun.push_str("                        Some(Ok(pair @ CodePair { code: 0, .. })) => {\n");
        fun.push_str("                            iter.put_back(Ok(pair));\n");
        fun.push_str("                            break;\n");
        fun.push_str("                        },\n");
        fun.push_str("                        Some(Ok(pair)) => {\n");
        fun.push_str("                            match pair.code {\n");
        fun.push_str("                                2 => item.name = string_value(&pair.value),\n");
        fun.push_str("                                5 => item.handle = try!(as_u32(string_value(&pair.value))),\n");
        fun.push_str("                                330 => item.owner_handle = try!(as_u32(string_value(&pair.value))),\n");
        for field in &table_item.children {
            if generate_reader(&field) {
                for (i, &cd) in codes(&field).iter().enumerate() {
                    let reader = get_field_reader(&field);
                    let codes = codes(&field);
                    let write_cmd = match codes.len() {
                        1 => {
                            let read_fun = if allow_multiples(&field) {
                                format!(".push({})", reader)
                            }
                            else {
                                format!(" = {}", reader)
                            };
                            format!("item.{field}{read_fun}", field=name(&field), read_fun=read_fun)
                        },
                        _ => {
                            let suffix = match i {
                                0 => "x",
                                1 => "y",
                                2 => "z",
                                _ => panic!("impossible"),
                            };
                            format!("item.{field}.{suffix} = {reader}", field=name(&field), suffix=suffix, reader=reader)
                        }
                    };
                    fun.push_str(format!("                                {code} => {{ {cmd}; }},\n", code=cd, cmd=write_cmd).as_str());
                }
            }
        }

        fun.push_str("                                _ => (), // unsupported code\n");
        fun.push_str("                            }\n");
        fun.push_str("                        },\n");
        fun.push_str("                        Some(Err(e)) => return Err(e),\n");
        fun.push_str("                        None => return Err(io::Error::new(io::ErrorKind::InvalidData, \"unexpected end of input\")),\n");
        fun.push_str("                    }\n");
        fun.push_str("                }\n");
        fun.push_str("\n");
        fun.push_str(format!("                drawing.{collection}.push(item);\n", collection=attr(&table, "Collection")).as_str());
        fun.push_str("            },\n");
        fun.push_str("            Some(Err(e)) => return Err(e),\n");
        fun.push_str("            None => return Err(io::Error::new(io::ErrorKind::InvalidData, \"unexpected end of input\")),\n");
        fun.push_str("        }\n");
        fun.push_str("    }\n");
        fun.push_str("\n");
        fun.push_str("    Ok(())\n");
        fun.push_str("}\n");
        fun.push_str("\n");
    }
}

fn generate_table_writer(fun: &mut String, element: &Element) {
    fun.push_str("pub fn write_tables<T>(drawing: &Drawing, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>\n");
    fun.push_str("    where T: Write {\n");
    for table in &element.children {
        fun.push_str(format!("    try!(write_{collection}(drawing, write_handles, writer));\n", collection=attr(&table, "Collection")).as_str());
    }

    fun.push_str("    Ok(())\n");
    fun.push_str("}\n");
    fun.push_str("\n");

    for table in &element.children {
        let table_item = &table.children[0];
        fun.push_str(format!("fn write_{collection}<T>(drawing: &Drawing, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>\n", collection=attr(&table, "Collection")).as_str());
        fun.push_str("    where T: Write {\n");
        fun.push_str(format!("    if drawing.{collection}.len() == 0 {{\n", collection=attr(&table, "Collection")).as_str());
        fun.push_str("        return Ok(()) // nothing to write\n");
        fun.push_str("    }\n");
        fun.push_str("\n");
        fun.push_str("    try!(writer.write_code_pair(&CodePair::new_str(0, \"TABLE\")));\n");
        fun.push_str(format!("    try!(writer.write_code_pair(&CodePair::new_str(2, \"{type_string}\")));\n", type_string=attr(&table, "TypeString")).as_str());

        // TODO: handles
        // fun.push_str("    if write_handles {\n");
        // fun.push_str("        try!(writer.write_code_pair(&CodePair::new_str(5, \"0\")));\n");
        // fun.push_str("    }\n");
        // fun.push_str("\n");

        fun.push_str("    try!(writer.write_code_pair(&CodePair::new_str(100, \"AcDbSymbolTable\")));\n");
        fun.push_str("    try!(writer.write_code_pair(&CodePair::new_short(70, 0)));\n");
        fun.push_str(format!("    for item in &drawing.{collection} {{\n", collection=attr(&table, "Collection")).as_str());
        fun.push_str(format!("        try!(writer.write_code_pair(&CodePair::new_str(0, \"{type_string}\")));\n", type_string=attr(&table, "TypeString")).as_str());
        fun.push_str("        if write_handles {\n");
        fun.push_str("            try!(writer.write_code_pair(&CodePair::new_string(5, &as_handle(item.handle))));\n");
        fun.push_str("        }\n");
        fun.push_str("\n");
        fun.push_str("        try!(writer.write_code_pair(&CodePair::new_str(100, \"AcDbSymbolTableRecord\")));\n");
        fun.push_str(format!("        try!(writer.write_code_pair(&CodePair::new_str(100, \"{class_name}\")));\n", class_name=attr(&table_item, "ClassName")).as_str());
        fun.push_str("        try!(writer.write_code_pair(&CodePair::new_string(2, &item.name)));\n");
        fun.push_str("        try!(writer.write_code_pair(&CodePair::new_short(70, 0)));\n"); // TODO: flags
        for field in &table_item.children {
            if generate_writer(&field) {
                let mut predicates = vec![];
                if !min_version(&field).is_empty() {
                    predicates.push(format!("drawing.header.version >= AcadVersion::{}", min_version(&field)));
                }
                if !max_version(&field).is_empty() {
                    predicates.push(format!("drawing.header.version <= AcadVersion::{}", max_version(&field)));
                }
                if !write_condition(&field).is_empty() {
                    predicates.push(write_condition(&field));
                }
                if disable_writing_default(&field) {
                    predicates.push(format!("item.{field} != {default_value}", field=name(&field), default_value=default_value(&field)));
                }
                let indent = if predicates.len() == 0 { "" } else { "    " };
                if predicates.len() != 0 {
                    fun.push_str(format!("        if {predicate} {{\n", predicate=predicates.join(" && ")).as_str());
                }

                if allow_multiples(&field) {
                    let code = code(&field);
                    let typ = get_expected_type(code).unwrap();
                    let typ = get_code_pair_type(typ);
                    let deref = if typ == "string" { "" } else { "*" };
                    fun.push_str(format!("{indent}        for x in &item.{field} {{\n", indent=indent, field=name(&field)).as_str());
                    fun.push_str(format!("{indent}            try!(writer.write_code_pair(&CodePair::new_{typ}({code}, {deref}x)))\n",
                        indent=indent, typ=typ, code=code, deref=deref).as_str());
                    fun.push_str(format!("{indent}        }}\n", indent=indent).as_str());
                }
                else {
                    let codes = codes(&field);
                    if codes.len() == 1 {
                        let code = codes[0];
                        let typ = get_expected_type(code).unwrap();
                        let typ = get_code_pair_type(typ);
                        let value = format!("item.{}", name(&field));
                        let write_converter = if attr(&field, "WriteConverter").is_empty() { String::from("{}") } else { attr(&field, "WriteConverter") };
                        let value = write_converter.replace("{}", value.as_str());
                        fun.push_str(format!("{indent}        try!(writer.write_code_pair(&CodePair::new_{typ}({code}, {value})));\n",
                            indent=indent, typ=typ, code=code, value=value).as_str());
                    }
                    else {
                        for (i, code) in codes.iter().enumerate() {
                            let suffix = match i {
                                0 => "x",
                                1 => "y",
                                2 => "z",
                                _ => panic!("impossible"),
                            };
                            fun.push_str(format!("{indent}        try!(writer.write_code_pair(&CodePair::new_double({code}, item.{field}.{suffix})));\n",
                                indent=indent, code=code, field=name(&field), suffix=suffix).as_str());
                        }
                    }
                }

                if predicates.len() != 0 {
                    fun.push_str("        }\n");
                }
            }
        }

        fun.push_str("    }\n");
        fun.push_str("\n");
        fun.push_str("    try!(writer.write_code_pair(&CodePair::new_str(0, \"ENDTAB\")));\n");
        fun.push_str("    Ok(())\n");
        fun.push_str("}\n");
        fun.push_str("\n");
    }
}

fn load_xml() -> Element {
    let file = File::open("spec/TableSpec.xml").unwrap();
    let file = BufReader::new(file);
    Element::parse(file).unwrap()
}
