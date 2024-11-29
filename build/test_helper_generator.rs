use xmltree::Element;

use crate::xml_helpers::*;

use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::Iterator;
use std::path::Path;

pub fn generate_test_helpers(generated_dir: &Path) {
    let _ = ::std::fs::create_dir(generated_dir.join("tests")); // might fail if it's already there

    let mut file = File::create(generated_dir.join("tests/mod.rs"))
        .ok()
        .unwrap();
    file.write_all("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

pub mod all_types;
".trim_start().as_bytes()).ok().unwrap();

    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

use crate::enums::*;
use crate::entities::*;
use crate::objects::*;
".trim_start());
    fun.push('\n');
    let mut file = File::create(generated_dir.join("tests/all_types.rs"))
        .ok()
        .unwrap();
    generate_entity_helpers(&mut fun);
    generate_object_helpers(&mut fun);
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_entity_helpers(fun: &mut String) {
    let file = File::open("spec/EntitiesSpec.xml").unwrap();
    let file = BufReader::new(file);
    let element = Element::parse(file).unwrap();

    fun.push_str("#[cfg(test)]\n");
    fun.push_str("#[allow(dead_code)]\n");
    fun.push_str("pub fn all_entity_types() -> Vec<(&'static str, &'static str, EntityType, AcadVersion)> {\n");
    fun.push_str("    vec![\n");
    for c in &element.children {
        if name(c) != "Entity" && name(c) != "DimensionBase" {
            let type_string = attr(c, "TypeString");
            let type_strings = type_string.split(',').collect::<Vec<_>>();
            let subclass = attr(c, "SubclassMarker");
            let maxver = max_version(c);
            let maxver = if maxver.is_empty() {
                String::from("R2018")
            } else {
                maxver
            };
            for type_string in &type_strings {
                fun.push_str(&format!("        (\"{type_string}\", \"{subclass}\", EntityType::{typ}({typ}::default()), AcadVersion::{ver}),\n",
                    type_string=type_string,
                    subclass=subclass,
                    typ=name(c),
                    ver=maxver));
            }
        }
    }
    fun.push_str("    ]\n");
    fun.push_str("}\n");
    fun.push('\n');
}

fn generate_object_helpers(fun: &mut String) {
    let file = File::open("spec/ObjectsSpec.xml").unwrap();
    let file = BufReader::new(file);
    let element = Element::parse(file).unwrap();

    fun.push_str("#[cfg(test)]\n");
    fun.push_str("#[allow(dead_code)]\n");
    fun.push_str("pub fn all_object_types() -> Vec<(&'static str, ObjectType, AcadVersion)> {\n");
    fun.push_str("    vec![\n");
    for c in &element.children {
        if name(c) != "Object" {
            let type_string = attr(c, "TypeString");
            let type_strings = type_string.split(',').collect::<Vec<_>>();
            let maxver = max_version(c);
            let maxver = if maxver.is_empty() {
                String::from("R2018")
            } else {
                maxver
            };
            for type_string in &type_strings {
                fun.push_str(&format!("        (\"{type_string}\", ObjectType::{typ}({typ}::default()), AcadVersion::{ver}),\n",
                    type_string=type_string,
                    typ=name(c),
                    ver=maxver));
            }
        }
    }
    fun.push_str("    ]\n");
    fun.push_str("}\n");
}
