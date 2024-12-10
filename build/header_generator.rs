use xmltree::Element;

use crate::ExpectedType;

use crate::other_helpers::*;
use crate::xml_helpers::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::Iterator;
use std::path::Path;

pub fn generate_header(generated_dir: &Path) {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

// types from `lib.rs`.
use crate::{
    CodePair,
    Color,
    DxfError,
    DxfResult,
    Handle,
    LineWeight,
    Point,
    Vector,
};
use crate::helper_functions::*;

use crate::enums::*;
use enum_primitive::FromPrimitive;

use std::time::Duration;

use chrono::{DateTime, Local, Utc};

use uuid::Uuid;
".trim_start());
    generate_struct(&mut fun, &element);

    generate_default(&mut fun, &element);

    fun.push_str("impl Header {\n");
    generate_flags(&mut fun, &element);
    generate_set_defaults(&mut fun, &element);
    generate_set_header_value(&mut fun, &element);
    generate_get_code_pairs_internal(&mut fun, &element);
    fun.push_str("}\n");

    let mut file = File::create(generated_dir.join("header.rs")).ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_struct(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("/// Contains common properties for the DXF file.\n");
    fun.push_str("#[derive(Debug, Clone)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub struct Header {\n");
    for v in &element.children {
        let field_name = field(v);
        if !seen_fields.contains(&field_name) {
            seen_fields.insert(field_name.clone());
            let mut comment = format!("The ${} header variable.  {}", name(v), comment(v));
            if !min_version(v).is_empty() {
                comment.push_str(&format!("  Minimum AutoCAD version: {}.", min_version(v)));
            }
            if !max_version(v).is_empty() {
                comment.push_str(&format!("  Maximum AutoCAD version: {}.", max_version(v)));
            }
            fun.push_str(&format!("    /// {}\n", comment));
            fun.push_str(&format!(
                "    pub {field}: {typ},\n",
                field = field(v),
                typ = typ(v)
            ));
        }
    }

    fun.push_str("}\n");
    fun.push('\n');
}

fn generate_default(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("impl Default for Header {\n");
    fun.push_str("    fn default() -> Self {\n");
    fun.push_str("        Header {\n");
    for v in &element.children {
        if !seen_fields.contains(&field(v)) {
            seen_fields.insert(field(v));
            fun.push_str(&format!(
                "            {field}: {default_value}, // ${name}\n",
                field = field(v),
                default_value = default_value(v),
                name = name(v)
            ));
        }
    }

    fun.push_str("        }\n");
    fun.push_str("    }\n");
    fun.push_str("}\n");
    fun.push('\n');
}

fn generate_flags(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    for v in &element.children {
        if !seen_fields.contains(&field(v)) {
            seen_fields.insert(field(v));
            if !v.children.is_empty() {
                fun.push_str(&format!("    // {} flags\n", field(v)));
            }
            for f in &v.children {
                let mut comment = comment(f).to_string();
                if !min_version(v).is_empty() {
                    comment.push_str(&format!("  Minimum AutoCAD version: {}.", min_version(v)));
                }
                if !max_version(v).is_empty() {
                    comment.push_str(&format!("  Maximum AutoCAD version: {}.", max_version(v)));
                }
                fun.push_str(&format!("    /// {}\n", comment));
                fun.push_str(&format!(
                    "    pub fn {flag}(&self) -> bool {{\n",
                    flag = name(f)
                ));
                fun.push_str(&format!(
                    "        self.{field} & {mask} != 0\n",
                    field = field(v),
                    mask = mask(f)
                ));
                fun.push_str("    }\n");
                fun.push_str(&format!("    /// {}\n", comment));
                fun.push_str(&format!(
                    "    pub fn set_{flag}(&mut self, val: bool) {{\n",
                    flag = name(f)
                ));
                fun.push_str("        if val {\n");
                fun.push_str(&format!(
                    "            self.{field} |= {mask};\n",
                    field = field(v),
                    mask = mask(f)
                ));
                fun.push_str("        }\n");
                fun.push_str("        else {\n");
                fun.push_str(&format!(
                    "            self.{field} &= !{mask};\n",
                    field = field(v),
                    mask = mask(f)
                ));
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
        if !seen_fields.contains(&field(v)) {
            seen_fields.insert(field(v));
            fun.push_str(&format!(
                "        self.{field} = {default_value}; // ${name}\n",
                field = field(v),
                default_value = default_value(v),
                name = name(v)
            ));
        }
    }

    fun.push_str("    }\n");
}

fn generate_set_header_value(fun: &mut String, element: &Element) {
    let mut seen_fields = HashSet::new();
    fun.push_str("    #[allow(clippy::cognitive_complexity)] // generated method\n");
    fun.push_str("    pub(crate) fn set_header_value(&mut self, variable: &str, pair: &CodePair) -> DxfResult<()> {\n");
    fun.push_str("        match variable {\n");
    for v in &element.children {
        if !seen_fields.contains(&field(v)) {
            seen_fields.insert(field(v));
            fun.push_str(&format!("            \"${name}\" => {{", name = name(v)));
            let variables_with_name: Vec<&Element> = element
                .children
                .iter()
                .filter(|&vv| name(vv) == name(v))
                .collect();
            if variables_with_name.len() == 1 {
                // only one variable with that name
                fun.push(' ');
                if code(v) < 0 {
                    fun.push_str(&format!("self.{field}.set(pair)?;", field = field(v)));
                } else {
                    let read_cmd = read_command(v);
                    fun.push_str(&format!(
                        "verify_code(pair, {code})?; self.{field} = {cmd};",
                        code = code(v),
                        field = field(v),
                        cmd = read_cmd
                    ));
                }

                fun.push(' ');
            } else {
                // multiple variables with that name
                fun.push('\n');
                fun.push_str("                match pair.code {\n");
                let expected_codes: Vec<i32> =
                    variables_with_name.iter().map(|&vv| code(vv)).collect();
                for v in &variables_with_name {
                    let read_cmd = read_command(v);
                    fun.push_str(&format!(
                        "                    {code} => self.{field} = {cmd},\n",
                        code = code(v),
                        field = field(v),
                        cmd = read_cmd
                    ));
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
    fun.push('\n');
    fun.push_str("        Ok(())\n");
    fun.push_str("    }\n");
}

fn read_command(element: &Element) -> String {
    let reader_override = reader_override(element);
    if !reader_override.is_empty() {
        reader_override
    } else {
        let expected_type = ExpectedType::new(code(element)).unwrap();
        let reader_fun = reader_function(&expected_type);
        let converter = if read_converter(element).is_empty() {
            String::from("{}")
        } else {
            read_converter(element)
        };
        converter.replace("{}", &format!("pair.{}()?", reader_fun))
    }
}

fn generate_get_code_pairs_internal(fun: &mut String, element: &Element) {
    fun.push_str("    #[allow(clippy::cognitive_complexity)] // long function, no good way to simplify this\n");
    fun.push_str("    pub(crate) fn add_code_pairs_internal(&self, pairs: &mut Vec<CodePair>) {\n");
    for v in &element.children {
        if suppress_writing(v) {
            continue;
        }

        // prepare writing predicate
        let mut parts = vec![];
        match (min_version(v).as_str(), max_version(v).as_str()) {
            ("", "") => (),
            (min, "") => parts.push(format!("self.version >= AcadVersion::{min}")),
            ("", max) => parts.push(format!("self.version <= AcadVersion::{max}")),
            (min, max) if min == max => parts.push(format!("self.version == AcadVersion::{min}")),
            (min, max) => {
                parts.push(format!("self.version >= AcadVersion::{min}"));
                parts.push(format!("self.version <= AcadVersion::{max}"));
            }
        }
        if do_not_write_default(v) {
            parts.push(format!("self.{} != {}", field(v), default_value(v)));
        }
        let indent = match parts.len() {
            0 => "",
            _ => "    ",
        };

        // write the value
        fun.push_str(&format!("        // ${}\n", name(v)));
        if !parts.is_empty() {
            fun.push_str(&format!("        if {} {{\n", parts.join(" && ")));
        }
        fun.push_str(&format!(
            "        {indent}pairs.push(CodePair::new_str(9, \"${name}\"));\n",
            name = name(v),
            indent = indent
        ));
        let write_converter = if write_converter(v).is_empty() {
            String::from("{}")
        } else {
            write_converter(v).clone()
        };
        if code(v) > 0 {
            let expected_type = code_pair_type(&ExpectedType::new(code(v)).unwrap());
            let field_name = field(v);
            let value = format!("self.{}", field_name);
            let value = write_converter.replace("{}", &value);
            fun.push_str(&format!(
                "        {indent}pairs.push(CodePair::new_{typ}({code}, {value}));\n",
                code = code(v),
                value = value,
                typ = expected_type,
                indent = indent
            ));
        } else {
            // write a point or vector as it's components
            for i in 0..code(v).abs() {
                let (code, fld) = match i {
                    0 => (10, "x"),
                    1 => (20, "y"),
                    2 => (30, "z"),
                    _ => panic!("unexpected number of values"),
                };
                let value = write_converter.replace("{}", &format!("self.{}.{}", field(v), fld));
                fun.push_str(&format!(
                    "        {indent}pairs.push(CodePair::new_f64({code}, {value}));\n",
                    code = code,
                    value = value,
                    indent = indent
                ));
            }
        }
        if !parts.is_empty() {
            fun.push_str("        }\n");
        }

        // newline between values
        fun.push('\n');
    }

    fun.push_str("    }\n");
}

fn load_xml() -> Element {
    let file = File::open("spec/HeaderVariablesSpec.xml").unwrap();
    let file = BufReader::new(file);
    Element::parse(file).unwrap()
}

fn do_not_write_default(element: &Element) -> bool {
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
