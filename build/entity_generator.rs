use xmltree::Element;

use crate::ExpectedType;

use crate::other_helpers::*;
use crate::xml_helpers::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};
use std::iter::Iterator;
use std::path::Path;

pub fn generate_entities(generated_dir: &Path) {
    let element = load_xml();
    let mut fun = String::new();
    fun.push_str("
// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

use crate::{
    CodePair,
    Color,
    Drawing,
    DrawingItem,
    DrawingItemMut,
    DxfError,
    DxfResult,
    ExtensionGroup,
    Handle,
    LwPolylineVertex,
    Point,
    Vector,
    XData,
};
use crate::code_pair_put_back::CodePairPutBack;
use crate::extension_data;
use crate::helper_functions::*;
use crate::tables::*;
use crate::x_data;

use enum_primitive::FromPrimitive;

use crate::enums::*;
use crate::objects::*;
".trim_start());
    fun.push('\n');
    generate_base_entity(&mut fun, &element);
    generate_entity_types(&mut fun, &element);

    fun.push_str("impl EntityType {\n");
    generate_is_supported_on_version(&mut fun, &element);
    generate_type_string(&mut fun, &element);
    generate_try_apply_code_pair(&mut fun, &element);
    generate_get_code_pairs(&mut fun, &element);
    fun.push_str("}\n");

    let mut file = File::create(generated_dir.join("entities.rs"))
        .ok()
        .unwrap();
    file.write_all(fun.as_bytes()).ok().unwrap();
}

fn generate_base_entity(fun: &mut String, element: &Element) {
    let entity = &element.children[0];
    if name(entity) != "Entity" {
        panic!("Expected first entity to be 'Entity'.");
    }
    fun.push_str("#[derive(Debug, Clone)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub struct EntityCommon {\n");
    for c in &entity.children {
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
            _ => panic!("unexpected element under Entity: {}", c.name),
        }
    }

    fun.push_str("}\n");
    fun.push('\n');

    fun.push_str("#[derive(Debug, Clone)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub struct Entity {\n");
    fun.push_str("    pub common: EntityCommon,\n");
    fun.push_str("    pub specific: EntityType,\n");
    fun.push_str("}\n");
    fun.push('\n');

    fun.push_str("impl Default for EntityCommon {\n");
    fun.push_str("    fn default() -> EntityCommon {\n");
    fun.push_str("        EntityCommon {\n");
    for c in &entity.children {
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
            _ => panic!("unexpected element under Entity: {}", c.name),
        }
    }

    fun.push_str("        }\n");
    fun.push_str("    }\n");
    fun.push_str("}\n");
    fun.push('\n');

    fun.push_str("impl EntityCommon {\n");

    /////////////////////////////////////////////////////////////////// pointers
    for p in &entity.children {
        if p.name == "Pointer" {
            fun.push_str(&methods_for_pointer_access(p));
        }
    }

    ////////////////////////////////////////////////////// apply_individual_pair
    fun.push_str("    pub(crate) fn apply_individual_pair(&mut self, pair: &CodePair, iter: &mut CodePairPutBack) -> DxfResult<()> {\n");
    fun.push_str("        match pair.code {\n");
    for c in &entity.children {
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
    fun.push_str("            _ => (), // unknown code, just ignore\n");
    fun.push_str("        }\n");
    fun.push_str("        Ok(())\n");
    fun.push_str("    }\n");

    ///////////////////////////////////////////////////////////// add_code_pairs
    fun.push_str("    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion, write_handles: bool) {\n");
    fun.push_str("        let ent = self;\n");
    for line in generate_write_code_pairs(entity) {
        fun.push_str(&format!("        {}\n", line));
    }

    fun.push_str("    }\n");

    fun.push_str("}\n");
    fun.push('\n');
}

fn generate_entity_types(fun: &mut String, element: &Element) {
    fun.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    fun.push_str(
        "#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    fun.push_str("pub enum EntityType {\n");
    for c in &element.children {
        if c.name != "Entity" {
            panic!("expected top level entity");
        }
        if name(c) != "Entity" && name(c) != "DimensionBase" {
            fun.push_str(&format!("    {typ}({typ}),\n", typ = name(c)));
        }
    }

    fun.push_str("}\n");
    fun.push('\n');

    // individual structs
    for c in &element.children {
        if c.name != "Entity" {
            panic!("expected top level entity");
        }
        if name(c) != "Entity" {
            // definition
            fun.push_str("#[derive(Clone, Debug, PartialEq)]\n");
            fun.push_str("#[cfg_attr(feature = \"serialize\", derive(serde::Serialize, serde::Deserialize))]\n");
            fun.push_str(&format!("pub struct {typ} {{\n", typ = name(c)));
            if base_class(c) == "DimensionBase" {
                fun.push_str("    pub dimension_base: DimensionBase,\n");
            }
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
                    _ => panic!("unexpected element {} under Entity", f.name),
                }
            }

            fun.push_str("}\n");
            fun.push('\n');

            // implementation
            fun.push_str("#[allow(clippy::derivable_impls)]\n");
            fun.push_str(&format!("impl Default for {typ} {{\n", typ = name(c)));
            fun.push_str(&format!("    fn default() -> {typ} {{\n", typ = name(c)));
            fun.push_str(&format!("        {typ} {{\n", typ = name(c)));
            if base_class(c) == "DimensionBase" {
                fun.push_str("            dimension_base: Default::default(),\n");
            }
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
                    _ => panic!("unexpected element {} under Entity", f.name),
                }
            }

            fun.push_str("        }\n");
            fun.push_str("    }\n");
            fun.push_str("}\n");
            fun.push('\n');

            generate_implementation(fun, c);

            if name(c) == "DimensionBase" {
                fun.push_str("impl DimensionBase {\n");
                fun.push_str("    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) {\n");
                fun.push_str("        let ent = self;\n");
                for line in generate_write_code_pairs(c) {
                    fun.push_str(&format!("        {}\n", line));
                }
                fun.push_str("    }\n");
                fun.push_str("}\n");
                fun.push('\n');
            }
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
    fun.push_str("        match *self {\n");
    for entity in &element.children {
        if name(entity) != "Entity" && name(entity) != "DimensionBase" {
            let predicate = match (min_version(entity).as_str(), max_version(entity).as_str()) {
                ("", "") => "true".into(),
                ("", max) => format!("version <= AcadVersion::{max}"),
                (min, "") => format!("version >= AcadVersion::{min}"),
                (min, max) if min == max => format!("version == AcadVersion::{min}"),
                (min, max) => {
                    format!("version >= AcadVersion::{min} && version <= AcadVersion::{max}")
                }
            };
            fun.push_str(&format!(
                "            EntityType::{typ}(_) => {{ {predicate} }},\n",
                typ = name(entity),
                predicate = predicate
            ));
        }
    }
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn generate_type_string(fun: &mut String, element: &Element) {
    fun.push_str("    pub(crate) fn from_type_string(type_string: &str) -> Option<EntityType> {\n");
    fun.push_str("        match type_string {\n");
    for c in &element.children {
        if name(c) != "Entity"
            && name(c) != "DimensionBase"
            && base_class(c) != "DimensionBase"
            && !attr(c, "TypeString").is_empty()
        {
            let type_string = attr(c, "TypeString");
            let type_strings = type_string.split(',').collect::<Vec<_>>();
            for t in type_strings {
                fun.push_str(&format!("            \"{type_string}\" => Some(EntityType::{typ}(Default::default())),\n", type_string=t, typ=name(c)));
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
        if name(c) != "Entity" && name(c) != "DimensionBase" && !type_string.is_empty() {
            fun.push_str(&format!(
                "            EntityType::{typ}(_) => {{ \"{type_string}\" }},\n",
                typ = name(c),
                type_string = type_strings[0]
            ));
        }
    }
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn generate_try_apply_code_pair(fun: &mut String, element: &Element) {
    fun.push_str(
        "    pub(crate) fn try_apply_code_pair(&mut self, pair: &CodePair) -> DxfResult<bool> {\n",
    );
    fun.push_str("        match *self {\n");
    for c in &element.children {
        if c.name != "Entity" {
            panic!("expected top level entity");
        }
        if name(c) != "Entity" && name(c) != "DimensionBase" {
            if generate_reader_function(c) {
                let ent = if name(c) == "Seqend" { "_ent" } else { "ent" }; // SEQEND doesn't use this variable
                fun.push_str(&format!(
                    "            EntityType::{typ}(ref mut {ent}) => {{\n",
                    typ = name(c),
                    ent = ent
                ));
                if ent == "ent" {
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
                                                "ent.{field}{read_fun}",
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
                                                "ent.{field}.{suffix} = {reader}",
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
                                fun.push_str(&format!("                    {code} => {{ ent.__{field}_handle.push(pair.as_handle()?); }},\n", code=code(f), field=name(f)));
                            } else {
                                fun.push_str(&format!("                    {code} => {{ ent.__{field}_handle = pair.as_handle()?; }},\n", code=code(f), field=name(f)));
                            }
                        }
                    }
                    fun.push_str("                    _ => return Ok(false),\n");
                    fun.push_str("                }\n");
                } else {
                    fun.push_str("                return Ok(false);\n")
                }
                fun.push_str("            },\n");
            } else {
                fun.push_str(&format!("            EntityType::{typ}(_) => {{ panic!(\"this case should have been covered in a custom reader\"); }},\n", typ=name(c)));
            }
        }
    }

    fun.push_str("        }\n");
    fun.push_str("        Ok(true)\n");
    fun.push_str("    }\n");
}

fn generate_get_code_pairs(fun: &mut String, element: &Element) {
    fun.push_str("    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, common: &EntityCommon, version: AcadVersion) {\n");
    fun.push_str("        match *self {\n");
    for entity in &element.children {
        if name(entity) != "Entity" && name(entity) != "DimensionBase" {
            if generate_writer_function(entity) {
                let ent = if name(entity) == "Seqend" {
                    "_ent"
                } else {
                    "ent"
                }; // SEQEND doesn't use this variable
                fun.push_str(&format!(
                    "            EntityType::{typ}(ref {ent}) => {{\n",
                    typ = name(entity),
                    ent = ent
                ));
                for line in generate_write_code_pairs(entity) {
                    fun.push_str(&format!("                {}\n", line));
                }

                fun.push_str("            },\n");
            } else {
                fun.push_str(&format!("            EntityType::{typ}(_) => {{ panic!(\"this case should have been covered in a custom writer\"); }},\n", typ=name(entity)));
            }
        }
    }
    fun.push_str("        }\n");
    fun.push_str("    }\n");
}

fn field_with_name<'a>(entity: &'a Element, field_name: &String) -> &'a Element {
    for field in &entity.children {
        if name(field) == *field_name {
            return field;
        }
    }

    panic!("unable to find field {}", field_name);
}

fn generate_write_code_pairs(entity: &Element) -> Vec<String> {
    let mut commands = vec![];
    for f in &entity.children {
        if f.name == "WriteOrder" {
            // order was specifically given to us
            for write_command in &f.children {
                for line in generate_write_code_pairs_for_write_order(entity, write_command) {
                    commands.push(line);
                }
            }
            return commands;
        }
    }

    // no order given, use declaration order
    let subclass = attr(entity, "SubclassMarker");
    if !subclass.is_empty() {
        commands.push("if version >= AcadVersion::R13 {".to_string());
        commands.push(format!(
            "    pairs.push(CodePair::new_str(100, \"{subclass}\"));",
            subclass = subclass
        ));
        commands.push("}".to_string());
    }
    for field in &entity.children {
        if generate_writer(field) {
            match &*field.name {
                "Field" => {
                    for line in write_lines_for_field(field, vec![]) {
                        commands.push(line);
                    }
                }
                "Pointer" => {
                    panic!("not used");
                }
                _ => panic!("unexpected item {} in entity", field.name),
            }
        }
    }
    commands
}

fn generate_write_code_pairs_for_write_order(
    entity: &Element,
    write_command: &Element,
) -> Vec<String> {
    let mut commands = vec![];
    match &*write_command.name {
        "WriteField" => {
            let field_name = write_command.attributes.get("Field").unwrap();
            let field = field_with_name(entity, field_name);
            let normalized_field_name = if field.name == "Pointer" {
                format!("__{}_handle", field_name)
            } else {
                field_name.clone()
            };
            let mut write_conditions = vec![attr(write_command, "WriteCondition")];
            if !attr(write_command, "DontWriteIfValueIs").is_empty() {
                write_conditions.push(format!(
                    "ent.{} != {}",
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
            if !attr(write_command, "WriteCondition").is_empty() {
                predicates.push(attr(write_command, "WriteCondition"));
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
                for line in generate_write_code_pairs_for_write_order(entity, write_command) {
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
        predicates.push(match default_value(field).as_str() {
            "true" => format!("!ent.{field}", field = name(field)),
            "false" => format!("ent.{field}", field = name(field)),
            default => format!(
                "ent.{} != {}",
                name(field),
                match typ(field).as_str() {
                    "String" => default.replace("String::from(", "").replace(')', ""),
                    _ => default.into(),
                }
            ),
        });
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
        let val = if field.name == "Pointer" {
            "&v.as_string()"
        } else {
            match expected_type {
                ExpectedType::Str => "v",
                ExpectedType::Binary => "v.clone()",
                _ => "*v",
            }
        };
        let normalized_field_name = if field.name == "Pointer" {
            format!("__{}_handle", name(field))
        } else {
            name(field)
        };
        let typ = code_pair_type(&expected_type);
        commands.push(format!(
            "{indent}for v in &ent.{field} {{",
            indent = indent,
            field = normalized_field_name
        ));
        commands.push(format!(
            "{indent}    pairs.push(CodePair::new_{typ}({code}, {val}));",
            indent = indent,
            typ = typ,
            code = codes(field)[0],
            val = val
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

fn code_pair_for_field_and_code(code: i32, field: &Element, suffix: Option<&str>) -> String {
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
    let normalized_field_name = if field.name == "Pointer" {
        format!("__{}_handle", name(field))
    } else {
        name(field)
    };
    let mut field_access = format!("ent.{field}", field = normalized_field_name);
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
    let file = File::open("spec/EntitiesSpec.xml").unwrap();
    let file = BufReader::new(file);
    Element::parse(file).unwrap()
}

fn generate_reader_function(element: &Element) -> bool {
    attr(element, "GenerateReaderFunction") != "false"
}

fn generate_writer_function(element: &Element) -> bool {
    attr(element, "GenerateWriterFunction") != "false"
}

fn base_class(element: &Element) -> String {
    attr(element, "BaseClass")
}
