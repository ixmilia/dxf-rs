use xmltree::Element;

use crate::ExpectedType;

use crate::other_helpers::*;
use crate::xml_helpers::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::Iterator;
use std::path::Path;

pub fn generate_objects(generated_dir: &Path) {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

use crate::{
    CodePair,
    Color,
    DataTableValue,
    Drawing,
    DrawingItem,
    DrawingItemMut,
    DxfError,
    DxfResult,
    ExtensionGroup,
    GeoMeshPoint,
    Handle,
    MLineStyleElement,
    Point,
    SectionTypeSettings,
    TableCellStyle,
    TransformationMatrix,
    Vector,
    XData,
};
use crate::code_pair_put_back::CodePairPutBack;
use crate::extension_data;
use crate::helper_functions::*;
use crate::tables::*;
use crate::x_data;

use crate::entities::*;
use crate::enums::*;
use enum_primitive::FromPrimitive;
use std::collections::HashMap;

use chrono::{DateTime, Local};
".trim_start());
    fun.push('\n');
    generate_base_object(&mut fun, &element);
    generate_object_types(&mut fun, &element);

    fun.push_str("impl ObjectType {\n");
    generate_is_supported_on_version(&mut fun, &element);
    generate_type_string(&mut fun, &element);
    generate_try_apply_code_pair(&mut fun, &element);
    generate_write(&mut fun, &element);
    fun.push_str("}\n");

    let mut file = File::create(generated_dir.join("objects.rs")).ok().unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_base_object(fun: &mut String, element: &Element) {
    let object = &element.children[0];
    if name(object) != "Object" {
        panic!("Expected first object to be 'Object'.");
    }
    fun.push_str("#[derive(Clone, Debug)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub struct ObjectCommon {\n");
    for c in &object.children {
        let t = if allow_multiples(c) {
            format!("Vec<{}>", typ(c))
        } else {
            typ(c)
        };
        if !comment(c).is_empty() {
            fun.push_str(&format!("    /// {}\n", comment(c)));
        }
        match &*c.name {
            "Field" => {
                fun.push_str(&format!(
                    "    pub {name}: {typ},\n",
                    name = name(c),
                    typ = t
                ));
            }
            "Pointer" => {
                let typ = if allow_multiples(c) {
                    "Vec<Handle>"
                } else {
                    "Handle"
                };
                fun.push_str("    #[doc(hidden)]\n");
                fun.push_str(&format!(
                    "    pub __{name}_handle: {typ},\n",
                    name = name(c),
                    typ = typ
                ));
            }
            "WriteOrder" => (),
            _ => panic!("unexpected element under Object: {}", c.name),
        }
    }

    fun.push_str("}\n");
    fun.push('\n');

    fun.push_str("#[derive(Clone, Debug)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub struct Object {\n");
    fun.push_str("    pub common: ObjectCommon,\n");
    fun.push_str("    pub specific: ObjectType,\n");
    fun.push_str("}\n");
    fun.push('\n');

    fun.push_str("impl Default for ObjectCommon {\n");
    fun.push_str("    fn default() -> ObjectCommon {\n");
    fun.push_str("        ObjectCommon {\n");
    for c in &object.children {
        match &*c.name {
            "Field" => {
                fun.push_str(&format!(
                    "            {name}: {val},\n",
                    name = name(c),
                    val = default_value(c)
                ));
            }
            "Pointer" => {
                fun.push_str(&format!(
                    "            __{name}_handle: Handle::empty(),\n",
                    name = name(c)
                ));
            }
            "WriteOrder" => (),
            _ => panic!("unexpected element under Object: {}", c.name),
        }
    }

    fun.push_str("        }\n");
    fun.push_str("    }\n");
    fun.push_str("}\n");
    fun.push('\n');

    fun.push_str("impl ObjectCommon {\n");

    /////////////////////////////////////////////////////////////////// pointers
    for p in &object.children {
        if p.name == "Pointer" {
            fun.push_str(&methods_for_pointer_access(p));
        }
    }

    ////////////////////////////////////////////////////// apply_individual_pair
    fun.push_str("    pub(crate) fn apply_individual_pair(&mut self, pair: &CodePair, iter: &mut CodePairPutBack) -> DxfResult<bool> {\n");
    fun.push_str("        match pair.code {\n");
    for c in &object.children {
        if c.name == "Field" {
            if name(c) == "extension_data_groups" && code(c) == 102 {
                fun.push_str("            extension_data::EXTENSION_DATA_GROUP => {\n");
                fun.push_str("                let group = ExtensionGroup::read_group(pair.assert_string()?, iter, pair.offset)?;\n");
                fun.push_str("                self.extension_data_groups.push(group);\n");
                fun.push_str("            },\n");
            } else if name(c) == "x_data" && code(c) == 1001 {
                // handled below: x_data::XDATA_APPLICATIONNAME
            } else {
                let read_fun = if allow_multiples(c) {
                    format!(".push({})", field_reader(c))
                } else {
                    format!(" = {}", field_reader(c))
                };
                fun.push_str(&format!(
                    "            {code} => {{ self.{field}{read_fun} }},\n",
                    code = code(c),
                    field = name(c),
                    read_fun = read_fun
                ));
            }
        } else if c.name == "Pointer" {
            fun.push_str(&format!(
                "            {code} => {{ self.__{field}_handle = pair.as_handle()? }},\n",
                code = code(c),
                field = name(c)
            ));
        }
    }

    fun.push_str("            x_data::XDATA_APPLICATIONNAME => {\n");
    fun.push_str("                let x = XData::read_item(pair.assert_string()?, iter)?;\n");
    fun.push_str("                self.x_data.push(x);\n");
    fun.push_str("            },\n");
    fun.push_str("            _ => return Ok(false), // unknown code\n");
    fun.push_str("        }\n");
    fun.push_str("        Ok(true)\n");
    fun.push_str("    }\n");

    ////////////////////////////////////////////////////////////////////// write
    fun.push_str(
        "    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) {\n",
    );
    fun.push_str("        let obj = self;\n");
    for line in generate_write_code_pairs(object) {
        fun.push_str(&format!("        {}\n", line));
    }

    fun.push_str("    }\n");

    fun.push_str("}\n");
    fun.push('\n');
}

fn generate_object_types(fun: &mut String, element: &Element) {
    fun.push_str("#[allow(clippy::large_enum_variant)]\n");
    fun.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub enum ObjectType {\n");
    for c in &element.children {
        if c.name != "Object" {
            panic!("expected top level object");
        }
        if name(c) != "Object" {
            fun.push_str(&format!("    {typ}({typ}),\n", typ = name(c)));
        }
    }

    fun.push_str("}\n");
    fun.push('\n');

    // individual structs
    for c in &element.children {
        if c.name != "Object" {
            panic!("expected top level object");
        }
        if name(c) != "Object" {
            // definition
            fun.push_str("#[derive(Clone, Debug, PartialEq)]\n");
            fun.push_str("#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n");
            fun.push_str(&format!("pub struct {typ} {{\n", typ = name(c)));
            for f in &c.children {
                let t = if allow_multiples(f) {
                    format!("Vec<{}>", typ(f))
                } else {
                    typ(f)
                };
                let is_private = name(f).starts_with('_');
                if !comment(f).is_empty() {
                    fun.push_str(&format!("    /// {}\n", comment(f)));
                }
                if is_private {
                    fun.push_str("    #[doc(hidden)]\n");
                }
                match &*f.name {
                    "Field" => {
                        fun.push_str(&format!(
                            "    pub {name}: {typ},\n",
                            name = name(f),
                            typ = t
                        ));
                    }
                    "Pointer" => {
                        let typ = if allow_multiples(f) {
                            "Vec<Handle>"
                        } else {
                            "Handle"
                        };
                        fun.push_str("    #[doc(hidden)]\n");
                        fun.push_str(&format!(
                            "    pub __{name}_handle: {typ},\n",
                            name = name(f),
                            typ = typ
                        ));
                    }
                    "WriteOrder" => (),
                    _ => panic!("unexpected element {} under Object", f.name),
                }
            }

            fun.push_str("}\n");
            fun.push('\n');

            // implementation
            fun.push_str("#[allow(clippy::derivable_impls)]\n");
            fun.push_str(&format!("impl Default for {typ} {{\n", typ = name(c)));
            fun.push_str(&format!("    fn default() -> {typ} {{\n", typ = name(c)));
            fun.push_str(&format!("        {typ} {{\n", typ = name(c)));
            for f in &c.children {
                match &*f.name {
                    "Field" => {
                        fun.push_str(&format!(
                            "            {name}: {val},\n",
                            name = name(f),
                            val = default_value(f)
                        ));
                    }
                    "Pointer" => {
                        let val = if allow_multiples(f) {
                            "vec![]"
                        } else {
                            "Handle::empty()"
                        };
                        fun.push_str(&format!(
                            "            __{name}_handle: {val},\n",
                            name = name(f),
                            val = val
                        ));
                    }
                    "WriteOrder" => (),
                    _ => panic!("unexpected element {} under Object", f.name),
                }
            }

            fun.push_str("        }\n");
            fun.push_str("    }\n");
            fun.push_str("}\n");
            fun.push('\n');

            generate_implementation(fun, c);
        }
    }
}

fn generate_implementation(fun: &mut String, element: &Element) {
    let mut implementation = String::new();

    // generate flags methods
    for field in &element.children {
        if field.name == "Field" {
            for flag in &field.children {
                if flag.name == "Flag" {
                    let flag_name = name(flag);
                    let mask = attr(flag, "Mask");
                    implementation.push_str(&format!(
                        "    pub fn {name}(&self) -> bool {{\n",
                        name = flag_name
                    ));
                    implementation.push_str(&format!(
                        "        self.{name} & {mask} != 0\n",
                        name = name(field),
                        mask = mask
                    ));
                    implementation.push_str("    }\n");
                    implementation.push_str(&format!(
                        "    pub fn set_{name}(&mut self, val: bool) {{\n",
                        name = flag_name
                    ));
                    implementation.push_str("        if val {\n");
                    implementation.push_str(&format!(
                        "            self.{name} |= {mask};\n",
                        name = name(field),
                        mask = mask
                    ));
                    implementation.push_str("        }\n");
                    implementation.push_str("        else {\n");
                    implementation.push_str(&format!(
                        "            self.{name} &= !{mask};\n",
                        name = name(field),
                        mask = mask
                    ));
                    implementation.push_str("        }\n");
                    implementation.push_str("    }\n");
                }
            }
        }
    }

    // generate pointer methods
    for field in &element.children {
        if field.name == "Pointer" {
            implementation.push_str(&methods_for_pointer_access(field));
        }
    }

    if !implementation.is_empty() {
        fun.push_str(&format!("impl {typ} {{\n", typ = name(element)));
        fun.push_str(&implementation);
        fun.push_str("}\n");
        fun.push('\n');
    }
}

fn generate_is_supported_on_version(fun: &mut String, element: &Element) {
    fun.push_str(
        "    pub(crate) fn is_supported_on_version(&self, version: AcadVersion) -> bool {\n",
    );
    fun.push_str("        match self {\n");
    for object in &element.children {
        if name(object) != "Object" {
            let mut predicates = vec![];
            if !min_version(object).is_empty() {
                predicates.push(format!("version >= AcadVersion::{}", min_version(object)));
            }
            if !max_version(object).is_empty() {
                predicates.push(format!("version <= AcadVersion::{}", max_version(object)));
            }
            let predicate = if predicates.is_empty() {
                String::from("true")
            } else {
                predicates.join(" && ")
            };
            fun.push_str(&format!(
                "            ObjectType::{typ}(_) => {{ {predicate} }},\n",
                typ = name(object),
                predicate = predicate
            ));
        }
    }
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn generate_type_string(fun: &mut String, element: &Element) {
    fun.push_str("    pub(crate) fn from_type_string(type_string: &str) -> Option<ObjectType> {\n");
    fun.push_str("        match type_string {\n");
    for c in &element.children {
        if name(c) != "Object" && !attr(c, "TypeString").is_empty() {
            let type_string = attr(c, "TypeString");
            let type_strings = type_string.split(',').collect::<Vec<_>>();
            for t in type_strings {
                fun.push_str(&format!("            \"{type_string}\" => Some(ObjectType::{typ}(Default::default())),\n", type_string=t, typ=name(c)));
            }
        }
    }

    fun.push_str("            _ => None,\n");
    fun.push_str("        }\n");
    fun.push_str("    }\n");

    fun.push_str("    pub(crate) fn to_type_string(&self) -> &str {\n");
    fun.push_str("        match *self {\n");
    for c in &element.children {
        // only write the first type string given
        let type_string = attr(c, "TypeString");
        let type_strings = type_string.split(',').collect::<Vec<_>>();
        if name(c) != "Object" && !type_string.is_empty() {
            fun.push_str(&format!(
                "            ObjectType::{typ}(_) => {{ \"{type_string}\" }},\n",
                typ = name(c),
                type_string = type_strings[0]
            ));
        }
    }
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn generate_try_apply_code_pair(fun: &mut String, element: &Element) {
    let mut unused_readers = vec![];
    fun.push_str("    #[allow(clippy::cognitive_complexity)] // long function, no good way to simplify this\n");
    fun.push_str(
        "    pub(crate) fn try_apply_code_pair(&mut self, pair: &CodePair) -> DxfResult<bool> {\n",
    );
    fun.push_str("        match *self {\n");
    for c in &element.children {
        if c.name != "Object" {
            panic!("expected top level object");
        }
        if name(c) != "Object" {
            if generate_reader_function(c) {
                let obj = if name(c) == "PlaceHolder" || name(c) == "ObjectPointer" {
                    "_obj"
                } else {
                    "obj"
                }; // ACDBPLACEHOLDER and OBJECT_PTR don't use this variable
                fun.push_str(&format!(
                    "            ObjectType::{typ}(ref mut {obj}) => {{\n",
                    typ = name(c),
                    obj = obj
                ));
                if obj == "obj" {
                    fun.push_str("                match pair.code {\n");
                    let mut seen_codes = HashSet::new();
                    for f in &c.children {
                        if f.name == "Field" && generate_reader(f) {
                            for (i, &cd) in codes(f).iter().enumerate() {
                                if !seen_codes.contains(&cd) {
                                    seen_codes.insert(cd); // TODO: allow for duplicates
                                    let reader = field_reader(f);
                                    let codes = codes(f);
                                    let write_cmd = match codes.len() {
                                        1 => {
                                            let read_fun = if allow_multiples(f) {
                                                format!(".push({})", reader)
                                            } else {
                                                format!(" = {}", reader)
                                            };
                                            format!(
                                                "obj.{field}{read_fun}",
                                                field = name(f),
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
                                                "obj.{field}.{suffix} = {reader}",
                                                field = name(f),
                                                suffix = suffix,
                                                reader = reader
                                            )
                                        }
                                    };
                                    fun.push_str(&format!(
                                        "                    {code} => {{ {cmd}; }},\n",
                                        code = cd,
                                        cmd = write_cmd
                                    ));
                                }
                            }
                        } else if f.name == "Pointer" {
                            if allow_multiples(f) {
                                fun.push_str(&format!("                    {code} => {{ obj.__{field}_handle.push(pair.as_handle()?); }},\n", code=code(f), field=name(f)));
                            } else {
                                fun.push_str(&format!("                    {code} => {{ obj.__{field}_handle = pair.as_handle()?; }},\n", code=code(f), field=name(f)));
                            }
                        }
                    }

                    fun.push_str("                    _ => return Ok(false),\n");
                    fun.push_str("                }\n");
                } else {
                    fun.push_str("                return Ok(false);\n");
                }
                fun.push_str("            },\n");
            } else {
                // ensure no read converters were specified (because they won't be used)
                for f in &c.children {
                    if f.name == "Field" && !attr(f, "ReadConverter").is_empty() {
                        unused_readers.push(format!("{}.{}", name(c), name(f)));
                    }
                }
                fun.push_str(&format!("            ObjectType::{typ}(_) => {{ panic!(\"this case should have been covered in a custom reader\"); }},\n", typ=name(c)));
            }
        }
    }

    fun.push_str("        }\n");
    fun.push_str("        Ok(true)\n");
    fun.push_str("    }\n");

    if !unused_readers.is_empty() {
        panic!("There were unused reader functions: {:?}", unused_readers);
    }
}

fn generate_write(fun: &mut String, element: &Element) {
    let mut unused_writers = vec![];
    fun.push_str("\n    #[allow(clippy::cognitive_complexity)] // long function, no good way to simplify this\n");
    fun.push_str(
        "    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) {\n",
    );
    fun.push_str("        match *self {\n");
    for object in &element.children {
        if name(object) != "Object" {
            if generate_writer_function(object) {
                let obj = if name(object) == "PlaceHolder" || name(object) == "ObjectPointer" {
                    "_obj"
                } else {
                    "obj"
                }; // ACDBPLACEHOLDER and OBJECT_PTR don't use this variable
                fun.push_str(&format!(
                    "            ObjectType::{typ}(ref {obj}) => {{\n",
                    typ = name(object),
                    obj = obj
                ));
                for line in generate_write_code_pairs(object) {
                    fun.push_str(&format!("                {}\n", line));
                }

                fun.push_str("            },\n");
            } else {
                // ensure no write converters were specified (because they won't be used)
                for f in &element.children {
                    if f.name == "Field" && !attr(f, "WriteConverter").is_empty() {
                        unused_writers.push(format!("{}.{}", name(element), name(f)));
                    }
                }
                fun.push_str(&format!("            ObjectType::{typ}(_) => {{ panic!(\"this case should have been covered in a custom writer\"); }},\n", typ=name(object)));
            }
        }
    }
    fun.push_str("        }\n");
    fun.push_str("    }\n");

    if !unused_writers.is_empty() {
        panic!("There were unused writer functions: {:?}", unused_writers);
    }
}

fn field_with_name<'a>(object: &'a Element, field_name: &String) -> &'a Element {
    for field in &object.children {
        if name(field) == *field_name {
            return field;
        }
    }

    panic!("unable to find field {}", field_name);
}

fn generate_write_code_pairs(object: &Element) -> Vec<String> {
    let mut commands = vec![];
    for f in &object.children {
        if f.name == "WriteOrder" {
            // order was specifically given to us
            for write_command in &f.children {
                for line in generate_write_code_pairs_for_write_order(object, write_command) {
                    commands.push(line);
                }
            }
            return commands;
        }
    }

    // no order given, use declaration order
    let subclass = attr(object, "SubclassMarker");
    if !subclass.is_empty() {
        commands.push(format!(
            "pairs.push(CodePair::new_str(100, \"{subclass}\"));",
            subclass = subclass
        ));
    }
    for field in &object.children {
        if generate_writer(field) {
            match &*field.name {
                "Field" | "Pointer" => {
                    for line in write_lines_for_field(field, vec![]) {
                        commands.push(line);
                    }
                }
                _ => panic!("unexpected item {} in object", field.name),
            }
        }
    }
    commands
}

fn generate_write_code_pairs_for_write_order(
    object: &Element,
    write_command: &Element,
) -> Vec<String> {
    let mut commands = vec![];
    match &*write_command.name {
        "WriteField" => {
            let field_name = write_command.attributes.get("Field").unwrap();
            let field = field_with_name(object, field_name);
            let normalized_field_name = if field.name == "Pointer" {
                format!("__{}_handle", field_name)
            } else {
                field_name.clone()
            };
            let mut write_conditions = vec![attr(write_command, "WriteCondition")];
            if !attr(write_command, "DontWriteIfValueIs").is_empty() {
                write_conditions.push(format!(
                    "obj.{} != {}",
                    normalized_field_name,
                    attr(write_command, "DontWriteIfValueIs")
                ));
            }
            for line in write_lines_for_field(field, write_conditions) {
                commands.push(line);
            }
        }
        "WriteSpecificValue" => {
            let mut predicates = vec![];
            if !min_version(write_command).is_empty() {
                predicates.push(format!(
                    "version >= AcadVersion::{}",
                    min_version(write_command)
                ));
            }
            if !max_version(write_command).is_empty() {
                predicates.push(format!(
                    "version <= AcadVersion::{}",
                    max_version(write_command)
                ));
            }
            if !attr(write_command, "DontWriteIfValueIs").is_empty() {
                predicates.push(format!(
                    "{} != {}",
                    attr(write_command, "Value"),
                    attr(write_command, "DontWriteIfValueIs")
                ));
            }
            let code = code(write_command);
            let expected_type = ExpectedType::new(code).unwrap();
            let typ = code_pair_type(&expected_type);
            if !predicates.is_empty() {
                commands.push(format!("if {} {{", predicates.join(" && ")));
            }
            let indent = if !predicates.is_empty() { "    " } else { "" };
            commands.push(format!(
                "{indent}pairs.push(CodePair::new_{typ}({code}, {val}));",
                indent = indent,
                typ = typ,
                code = code,
                val = attr(write_command, "Value")
            ));
            if !predicates.is_empty() {
                commands.push(String::from("}"));
            }
        }
        "Foreach" => {
            commands.push(format!("for item in &{} {{", attr(write_command, "Field")));
            for write_command in &write_command.children {
                for line in generate_write_code_pairs_for_write_order(object, write_command) {
                    commands.push(format!("    {}", line));
                }
            }
            commands.push(String::from("}"));
        }
        "WriteExtensionData" => {
            commands.push(String::from("if version >= AcadVersion::R14 {"));
            commands.push(String::from(
                "    for group in &self.extension_data_groups {",
            ));
            commands.push(String::from("        group.add_code_pairs(pairs);"));
            commands.push(String::from("    }"));
            commands.push(String::from("}"));
        }
        _ => panic!("unexpected write command {}", write_command.name),
    }

    commands
}

fn write_lines_for_field(field: &Element, write_conditions: Vec<String>) -> Vec<String> {
    let mut commands = vec![];
    let mut predicates = vec![];
    if !min_version(field).is_empty() {
        predicates.push(format!("version >= AcadVersion::{}", min_version(field)));
    }
    if !max_version(field).is_empty() {
        predicates.push(format!("version <= AcadVersion::{}", max_version(field)));
    }
    if disable_writing_default(field) {
        predicates.push(format!(
            "obj.{field} != {default}",
            field = name(field),
            default = default_value(field)
        ));
    }
    for wc in write_conditions {
        if !wc.is_empty() {
            predicates.push(wc);
        }
    }
    let indent = if predicates.is_empty() { "" } else { "    " };
    if !predicates.is_empty() {
        commands.push(format!("if {} {{", predicates.join(" && ")));
    }

    if allow_multiples(field) {
        let expected_type = ExpectedType::new(codes(field)[0]).unwrap();
        let val = match (&*field.name, &expected_type) {
            ("Pointer", _) => "v",
            (_, &ExpectedType::Str) => "&v",
            (_, &ExpectedType::Binary) => "v.clone()",
            _ => "*v",
        };
        let write_converter = write_converter(field);
        let to_write = write_converter.replace("{}", val).replace("&&", "");
        let typ = code_pair_type(&expected_type);
        let normalized_field_name = if field.name == "Pointer" {
            format!("__{}_handle", name(field))
        } else {
            name(field)
        };
        commands.push(format!(
            "{indent}for v in &obj.{field} {{",
            indent = indent,
            field = normalized_field_name
        ));
        commands.push(format!(
            "{indent}    pairs.push(CodePair::new_{typ}({code}, {to_write}));",
            indent = indent,
            typ = typ,
            code = codes(field)[0],
            to_write = to_write
        ));
        commands.push(format!("{indent}}}", indent = indent));
    } else {
        for command in code_pairs_for_field(field) {
            commands.push(format!(
                "{indent}pairs.push({command});",
                indent = indent,
                command = command
            ));
        }
    }

    if !predicates.is_empty() {
        commands.push(String::from("}"));
    }

    commands
}

fn code_pairs_for_field(field: &Element) -> Vec<String> {
    let codes = codes(field);
    match codes.len() {
        1 => vec![code_pair_for_field_and_code(codes[0], field, None)],
        _ => {
            let mut pairs = vec![];
            for (i, &cd) in codes.iter().enumerate() {
                let suffix = match i {
                    0 => "x",
                    1 => "y",
                    2 => "z",
                    _ => panic!("unexpected multiple codes"),
                };
                pairs.push(code_pair_for_field_and_code(cd, field, Some(suffix)));
            }
            pairs
        }
    }
}

fn write_converter(field: &Element) -> String {
    let code = codes(field)[0];
    write_converter_with_code(code, field)
}

fn write_converter_with_code(code: i32, field: &Element) -> String {
    let expected_type = ExpectedType::new(code).unwrap();
    let typ = code_pair_type(&expected_type);
    let mut write_converter = attr(field, "WriteConverter");
    if field.name == "Pointer" {
        write_converter = String::from("&{}.as_string()");
    }
    if write_converter.is_empty() {
        if typ == "string" {
            write_converter = String::from("&{}");
        } else {
            write_converter = String::from("{}");
        }
    }

    write_converter
}

fn code_pair_for_field_and_code(code: i32, field: &Element, suffix: Option<&str>) -> String {
    let expected_type = ExpectedType::new(code).unwrap();
    let typ = code_pair_type(&expected_type);
    let write_converter = write_converter_with_code(code, field);
    let normalized_field_name = if field.name == "Pointer" {
        format!("__{}_handle", name(field))
    } else {
        name(field)
    };
    let mut field_access = format!("obj.{field}", field = normalized_field_name);
    if let Some(suffix) = suffix {
        field_access = format!("{}.{}", field_access, suffix);
    }
    let writer = write_converter.replace("{}", &field_access);
    if name(field) == "handle" && code == 5 {
        String::from("CodePair::new_string(5, &self.handle.as_string())")
    } else {
        format!(
            "CodePair::new_{typ}({code}, {writer})",
            typ = typ,
            code = code,
            writer = writer
        )
    }
}

fn load_xml() -> Element {
    let file = File::open("spec/ObjectsSpec.xml").unwrap();
    let file = BufReader::new(file);
    Element::parse(file).unwrap()
}

fn generate_reader_function(element: &Element) -> bool {
    attr(element, "GenerateReaderFunction") != "false"
}

fn generate_writer_function(element: &Element) -> bool {
    attr(element, "GenerateWriterFunction") != "false"
}
