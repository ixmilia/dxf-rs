// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate xmltree;
use self::xmltree::Element;

use crate::ExpectedType;

use crate::other_helpers::*;
use crate::xml_helpers::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub fn generate_tables(generated_dir: &Path) {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

extern crate itertools;

use crate::{
    CodePair,
    CodePairValue,
    Color,
    Drawing,
    DrawingItem,
    DrawingItemMut,
    DxfError,
    DxfResult,
    ExtensionGroup,
    LineWeight,
    Point,
    Vector,
    XData,
};
use crate::code_pair_put_back::CodePairPutBack;
use crate::code_pair_writer::CodePairWriter;
use crate::handle_tracker::HandleTracker;
use crate::helper_functions::*;
use crate::extension_data;
use crate::x_data;

use crate::enums::*;
use crate::enum_primitive::FromPrimitive;
use std::io::{Read, Write};
".trim_start());
    fun.push_str("\n");
    generate_table_items(&mut fun, &element);
    generate_table_reader(&mut fun, &element);
    generate_table_writer(&mut fun, &element);

    let mut file = File::create(generated_dir.join("tables.rs")).ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_table_items(fun: &mut String, element: &Element) {
    for table in &element.children {
        let mut seen_fields = HashSet::new();
        let table_item = &table.children[0];
        fun.push_str("#[cfg_attr(feature = \"serialize\", derive(Serialize, Deserialize))]\n");
        fun.push_str(&format!("pub struct {name} {{\n", name = name(&table_item)));
        fun.push_str("    pub name: String,\n");
        fun.push_str("    pub handle: u32,\n");
        fun.push_str("    #[doc(hidden)]\n");
        fun.push_str("    pub __owner_handle: u32,\n");
        fun.push_str("    pub extension_data_groups: Vec<ExtensionGroup>,\n");
        fun.push_str("    pub x_data: Vec<XData>,\n");
        for field in &table_item.children {
            let name = if field.name == "Pointer" {
                format!("__{}_handle", name(&field))
            } else {
                name(&field)
            };
            if !seen_fields.contains(&name) {
                seen_fields.insert(name.clone());
                let mut typ = if field.name == "Pointer" {
                    String::from("u32")
                } else {
                    attr(&field, "Type")
                };
                if allow_multiples(&field) {
                    typ = format!("Vec<{}>", typ);
                }
                let is_private = name.starts_with("_");
                if is_private {
                    fun.push_str("    #[doc(hidden)]\n");
                }
                fun.push_str(&format!("    pub {name}: {typ},\n", name = name, typ = typ));
            }
        }
        fun.push_str("}\n");
        fun.push_str("\n");

        seen_fields.clear();
        fun.push_str(&format!(
            "impl Default for {name} {{\n",
            name = name(&table_item)
        ));
        fun.push_str("    fn default() -> Self {\n");
        fun.push_str(&format!("        {name} {{\n", name = name(&table_item)));
        fun.push_str("            name: String::new(),\n");
        fun.push_str("            handle: 0,\n");
        fun.push_str("            __owner_handle: 0,\n");
        fun.push_str("            extension_data_groups: vec![],\n");
        fun.push_str("            x_data: vec![],\n");
        for field in &table_item.children {
            let name = if field.name == "Pointer" {
                format!("__{}_handle", name(&field))
            } else {
                name(&field)
            };
            if !seen_fields.contains(&name) {
                seen_fields.insert(name.clone());
                let default_value = match (&*field.name, allow_multiples(&field)) {
                    ("Pointer", true) => String::from("vec![]"),
                    ("Pointer", false) => String::from("0"),
                    (_, _) => attr(&field, "DefaultValue"),
                };
                fun.push_str(&format!(
                    "            {field}: {default_value},\n",
                    field = name,
                    default_value = default_value
                ));
            }
        }

        fun.push_str("        }\n");
        fun.push_str("    }\n");
        fun.push_str("}\n");
        fun.push_str("\n");

        fun.push_str(&format!("impl {name} {{\n", name = name(&table_item)));
        fun.push_str(
            "    pub fn get_owner<'a>(&self, drawing: &'a Drawing) -> Option<DrawingItem<'a>> {\n",
        );
        fun.push_str("        drawing.get_item_by_handle(self.__owner_handle)\n");
        fun.push_str("    }\n");
        fun.push_str("    pub fn set_owner<'a>(&mut self, item: &'a mut DrawingItemMut, drawing: &'a mut Drawing) {\n");
        fun.push_str("        self.__owner_handle = drawing.assign_and_get_handle(item);\n");
        fun.push_str("    }\n");
        fun.push_str("}\n");
        fun.push_str("\n");
    }
}

fn generate_table_reader(fun: &mut String, element: &Element) {
    fun.push_str("pub(crate) fn read_specific_table<I>(drawing: &mut Drawing, iter: &mut CodePairPutBack<I>) -> DxfResult<()>\n");
    fun.push_str("    where I: Read {\n");
    fun.push_str("\n");
    fun.push_str("    match iter.next() {\n");
    fun.push_str("        Some(Ok(pair)) => {\n");
    fun.push_str("            if pair.code != 2 {\n");
    fun.push_str("                return Err(DxfError::ExpectedTableType(pair.offset));\n");
    fun.push_str("            }\n");
    fun.push_str("\n");
    fun.push_str("            match &*pair.assert_string()? {\n");

    for table in &element.children {
        fun.push_str(&format!(
            "                \"{table_name}\" => read_{collection}(drawing, iter)?,\n",
            table_name = attr(&table, "TypeString"),
            collection = attr(&table, "Collection")
        ));
    }

    fun.push_str("                _ => Drawing::swallow_table(iter)?,\n");
    fun.push_str("            }\n");
    fun.push_str("\n");
    fun.push_str("            match iter.next() {\n");
    fun.push_str("                Some(Ok(CodePair { code: 0, value: CodePairValue::Str(ref s), .. })) if s == \"ENDTAB\" => (),\n");
    fun.push_str("                Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from(\"expected 0/ENDTAB\"))),\n");
    fun.push_str("                Some(Err(e)) => return Err(e),\n");
    fun.push_str("                None => return Err(DxfError::UnexpectedEndOfInput),\n");
    fun.push_str("            }\n");
    fun.push_str("        },\n");
    fun.push_str("        Some(Err(e)) => return Err(e),\n");
    fun.push_str("        None => return Err(DxfError::UnexpectedEndOfInput),\n");
    fun.push_str("    }\n");
    fun.push_str("\n");
    fun.push_str("    Ok(())\n");
    fun.push_str("}\n");
    fun.push_str("\n");

    for table in &element.children {
        let table_item = &table.children[0];

        fun.push_str(&format!("fn read_{collection}<I>(drawing: &mut Drawing, iter: &mut CodePairPutBack<I>) -> DxfResult<()>\n", collection=attr(&table, "Collection")));
        fun.push_str("    where I: Read {\n");
        fun.push_str("\n");
        fun.push_str("    loop {\n");
        fun.push_str("        match iter.next() {\n");
        fun.push_str("            Some(Ok(pair)) => {\n");
        fun.push_str("                if pair.code == 0 {\n");
        fun.push_str(&format!(
            "                    if pair.assert_string()? != \"{table_type}\" {{\n",
            table_type = attr(&table, "TypeString")
        ));
        fun.push_str("                        iter.put_back(Ok(pair));\n");
        fun.push_str("                        break;\n");
        fun.push_str("                    }\n");
        fun.push_str("\n");
        fun.push_str(&format!(
            "                    let mut item = {typ}::default();\n",
            typ = attr(&table_item, "Name")
        ));
        fun.push_str("                    loop {\n");
        fun.push_str("                        match iter.next() {\n");
        fun.push_str(
            "                            Some(Ok(pair @ CodePair { code: 0, .. })) => {\n",
        );
        fun.push_str("                                iter.put_back(Ok(pair));\n");
        fun.push_str("                                break;\n");
        fun.push_str("                            },\n");
        fun.push_str("                            Some(Ok(pair)) => {\n");
        fun.push_str("                                match pair.code {\n");
        fun.push_str(
            "                                    2 => item.name = pair.assert_string()?,\n",
        );
        fun.push_str("                                    5 => item.handle = pair.as_handle()?,\n");
        fun.push_str(
            "                                    extension_data::EXTENSION_DATA_GROUP => {\n",
        );
        fun.push_str("                                        let group = ExtensionGroup::read_group(pair.assert_string()?, iter, pair.offset)?;\n");
        fun.push_str(
            "                                        item.extension_data_groups.push(group);\n",
        );
        fun.push_str("                                    },\n");
        fun.push_str("                                    x_data::XDATA_APPLICATIONNAME => {\n");
        fun.push_str("                                        let x = XData::read_item(pair.assert_string()?, iter)?;\n");
        fun.push_str("                                        item.x_data.push(x);\n");
        fun.push_str("                                    },\n");
        fun.push_str(
            "                                    330 => item.__owner_handle = pair.as_handle()?,\n",
        );
        for field in &table_item.children {
            if generate_reader(&field) {
                for (i, &cd) in codes(&field).iter().enumerate() {
                    let reader = get_field_reader(&field);
                    let codes = codes(&field);
                    let write_cmd = match codes.len() {
                        1 => {
                            let read_fun = if allow_multiples(&field) {
                                format!(".push({})", reader)
                            } else {
                                format!(" = {}", reader)
                            };
                            let normalized_field_name = if field.name == "Pointer" {
                                format!("__{}_handle", name(&field))
                            } else {
                                name(&field)
                            };
                            format!(
                                "item.{field}{read_fun}",
                                field = normalized_field_name,
                                read_fun = read_fun
                            )
                        }
                        _ => {
                            let suffix = match i {
                                0 => "x",
                                1 => "y",
                                2 => "z",
                                _ => panic!("impossible"),
                            };
                            format!(
                                "item.{field}.{suffix} = {reader}",
                                field = name(&field),
                                suffix = suffix,
                                reader = reader
                            )
                        }
                    };
                    fun.push_str(&format!(
                        "                                    {code} => {{ {cmd}; }},\n",
                        code = cd,
                        cmd = write_cmd
                    ));
                }
            }
        }

        fun.push_str("                                    _ => (), // unsupported code\n");
        fun.push_str("                                }\n");
        fun.push_str("                            },\n");
        fun.push_str("                            Some(Err(e)) => return Err(e),\n");
        fun.push_str(
            "                            None => return Err(DxfError::UnexpectedEndOfInput),\n",
        );
        fun.push_str("                        }\n");
        fun.push_str("                    }\n");
        fun.push_str("\n");
        fun.push_str(&format!(
            "                    drawing.{collection}.push(item);\n",
            collection = attr(&table, "Collection")
        ));
        fun.push_str("                }\n");
        fun.push_str("                else {\n");
        fun.push_str("                    // do nothing, probably the table's handle or flags\n");
        fun.push_str("                }\n");
        fun.push_str("            },\n");
        fun.push_str("            Some(Err(e)) => return Err(e),\n");
        fun.push_str("            None => return Err(DxfError::UnexpectedEndOfInput),\n");
        fun.push_str("        }\n");
        fun.push_str("    }\n");
        fun.push_str("\n");
        fun.push_str("    Ok(())\n");
        fun.push_str("}\n");
        fun.push_str("\n");
    }
}

fn generate_table_writer(fun: &mut String, element: &Element) {
    fun.push_str("pub(crate) fn write_tables<T>(drawing: &Drawing, write_handles: bool, writer: &mut CodePairWriter<T>, handle_tracker: &mut HandleTracker) -> DxfResult<()>\n");
    fun.push_str("    where T: Write {\n");
    fun.push_str("\n");
    for table in &element.children {
        fun.push_str(&format!(
            "    write_{collection}(drawing, write_handles, writer, handle_tracker)?;\n",
            collection = attr(&table, "Collection")
        ));
    }

    fun.push_str("    Ok(())\n");
    fun.push_str("}\n");
    fun.push_str("\n");

    for table in &element.children {
        let table_item = &table.children[0];
        fun.push_str("#[allow(clippy::cognitive_complexity)] // long function, no good way to simplify this\n");
        fun.push_str(&format!("fn write_{collection}<T>(drawing: &Drawing, write_handles: bool, writer: &mut CodePairWriter<T>, handle_tracker: &mut HandleTracker) -> DxfResult<()>\n", collection=attr(&table, "Collection")));
        fun.push_str("    where T: Write {\n");
        fun.push_str("\n");
        fun.push_str(&format!(
            "    if drawing.{collection}.is_empty() {{\n",
            collection = attr(&table, "Collection")
        ));
        fun.push_str("        return Ok(()) // nothing to write\n");
        fun.push_str("    }\n");
        fun.push_str("\n");
        fun.push_str("    writer.write_code_pair(&CodePair::new_str(0, \"TABLE\"))?;\n");
        fun.push_str(&format!(
            "    writer.write_code_pair(&CodePair::new_str(2, \"{type_string}\"))?;\n",
            type_string = attr(&table, "TypeString")
        ));

        // TODO: assign and write table handles
        // fun.push_str("    if write_handles {\n");
        // fun.push_str("        writer.write_code_pair(&CodePair::new_str(5, \"0\"))?;\n");
        // fun.push_str("    }\n");
        // fun.push_str("\n");

        let collection = attr(&table, "Collection");
        let (item_type, _) = collection.split_at(collection.len() - 1); // remove the 's' suffix

        fun.push_str(
            "    writer.write_code_pair(&CodePair::new_str(100, \"AcDbSymbolTable\"))?;\n",
        );
        fun.push_str("    writer.write_code_pair(&CodePair::new_i16(70, 0))?;\n");
        fun.push_str(&format!(
            "    for item in &drawing.{collection} {{\n",
            collection = attr(&table, "Collection")
        ));
        fun.push_str(&format!(
            "        writer.write_code_pair(&CodePair::new_str(0, \"{type_string}\"))?;\n",
            type_string = attr(&table, "TypeString")
        ));
        fun.push_str("        if write_handles {\n");
        fun.push_str(&format!("            writer.write_code_pair(&CodePair::new_string(5, &as_handle(handle_tracker.get_{item_type}_handle(&item))))?;\n",
            item_type=item_type));
        fun.push_str("        }\n");
        fun.push_str("\n");
        fun.push_str("        if drawing.header.version >= AcadVersion::R14 {\n");
        fun.push_str("            for group in &item.extension_data_groups {\n");
        fun.push_str("                group.write(writer)?;\n");
        fun.push_str("            }\n");
        fun.push_str("        }\n");
        fun.push_str("\n");
        fun.push_str("        writer.write_code_pair(&CodePair::new_str(100, \"AcDbSymbolTableRecord\"))?;\n");
        fun.push_str(&format!(
            "        writer.write_code_pair(&CodePair::new_str(100, \"{class_name}\"))?;\n",
            class_name = attr(&table_item, "ClassName")
        ));
        fun.push_str("        writer.write_code_pair(&CodePair::new_string(2, &item.name))?;\n");
        fun.push_str("        writer.write_code_pair(&CodePair::new_i16(70, 0))?;\n"); // TODO: flags
        for field in &table_item.children {
            if generate_writer(&field) {
                let mut predicates = vec![];
                if !min_version(&field).is_empty() {
                    predicates.push(format!(
                        "drawing.header.version >= AcadVersion::{}",
                        min_version(&field)
                    ));
                }
                if !max_version(&field).is_empty() {
                    predicates.push(format!(
                        "drawing.header.version <= AcadVersion::{}",
                        max_version(&field)
                    ));
                }
                if !write_condition(&field).is_empty() {
                    predicates.push(write_condition(&field));
                }
                if disable_writing_default(&field) {
                    predicates.push(format!(
                        "item.{field} != {default_value}",
                        field = name(&field),
                        default_value = default_value(&field)
                    ));
                }
                let indent = if predicates.len() == 0 { "" } else { "    " };
                if predicates.len() != 0 {
                    fun.push_str(&format!(
                        "        if {predicate} {{\n",
                        predicate = predicates.join(" && ")
                    ));
                }

                if allow_multiples(&field) {
                    let code = code(&field);
                    if field.name == "Pointer" {
                        fun.push_str(&format!(
                            "{indent}        for x in &item.__{field}_handle {{\n",
                            indent = indent,
                            field = name(&field)
                        ));
                        fun.push_str(&format!("{indent}            writer.write_code_pair(&CodePair::new_string({code}, &as_handle(*x)))?;\n",
                            indent=indent, code=code));
                    } else {
                        let typ = ExpectedType::get_expected_type(code).unwrap();
                        let typ = get_code_pair_type(typ);
                        let deref = if typ == "string" { "" } else { "*" };
                        fun.push_str(&format!(
                            "{indent}        for x in &item.{field} {{\n",
                            indent = indent,
                            field = name(&field)
                        ));
                        fun.push_str(&format!("{indent}            writer.write_code_pair(&CodePair::new_{typ}({code}, {deref}x))?;\n",
                            indent=indent, typ=typ, code=code, deref=deref));
                    }
                    fun.push_str(&format!("{indent}        }}\n", indent = indent));
                } else {
                    let codes = codes(&field);
                    if codes.len() == 1 {
                        let code = codes[0];
                        if field.name == "Pointer" {
                            fun.push_str(&format!("{indent}        writer.write_code_pair(&CodePair::new_string({code}, &as_handle(item.__{field}_handle)))?;\n",
                                indent=indent, code=code, field=name(&field)));
                        } else {
                            let typ = ExpectedType::get_expected_type(code).unwrap();
                            let typ = get_code_pair_type(typ);
                            let value = format!("item.{}", name(&field));
                            let write_converter = if attr(&field, "WriteConverter").is_empty() {
                                String::from("{}")
                            } else {
                                attr(&field, "WriteConverter")
                            };
                            let value = write_converter.replace("{}", &value);
                            fun.push_str(&format!("{indent}        writer.write_code_pair(&CodePair::new_{typ}({code}, {value}))?;\n",
                                indent=indent, typ=typ, code=code, value=value));
                        }
                    } else {
                        for (i, code) in codes.iter().enumerate() {
                            let suffix = match i {
                                0 => "x",
                                1 => "y",
                                2 => "z",
                                _ => panic!("impossible"),
                            };
                            fun.push_str(&format!("{indent}        writer.write_code_pair(&CodePair::new_f64({code}, item.{field}.{suffix}))?;\n",
                                indent=indent, code=code, field=name(&field), suffix=suffix));
                        }
                    }
                }

                if predicates.len() != 0 {
                    fun.push_str("        }\n");
                }
            }
        }

        fun.push_str("        for x in &item.x_data {\n");
        fun.push_str("            x.write(drawing.header.version, writer)?;\n");
        fun.push_str("        }\n");

        fun.push_str("    }\n");
        fun.push_str("\n");
        fun.push_str("    writer.write_code_pair(&CodePair::new_str(0, \"ENDTAB\"))?;\n");
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
