use crate::other_helpers::*;
use crate::ExpectedType;
use xmltree::Element;

pub fn attr(element: &Element, name: &str) -> String {
    match element.attributes.get(name) {
        Some(v) => v.clone(),
        None => String::new(),
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
    let code_overrides = attr(element, "CodeOverrides");
    if code_overrides.is_empty() {
        vec![code(element)]
    } else {
        code_overrides
            .split(',')
            .map(|c| c.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    }
}

pub fn default_value(element: &Element) -> String {
    attr(element, "DefaultValue")
}

pub fn disable_writing_default(element: &Element) -> bool {
    attr(element, "DisableWritingDefault") == "true"
}

pub fn generate_reader(element: &Element) -> bool {
    attr(element, "GenerateReader") != "false"
}

pub fn generate_writer(element: &Element) -> bool {
    attr(element, "GenerateWriter") != "false"
}

pub fn field_reader(element: &Element) -> String {
    let reader_override = attr(element, "ReaderOverride");
    if !reader_override.is_empty() {
        reader_override
    } else {
        let expected_type = ExpectedType::new(code(element)).unwrap();
        let reader_fun = reader_function(&expected_type);
        let mut read_converter = attr(element, "ReadConverter");
        if read_converter.is_empty() {
            read_converter = String::from("{}");
        }
        let read_cmd = format!("pair.{}()?", reader_fun);
        let normalized_read_cmd = if element.name == "Pointer" {
            String::from("pair.as_handle()?")
        } else {
            read_cmd
        };
        read_converter.replace("{}", &normalized_read_cmd)
    }
}

pub fn methods_for_pointer_access(pointer: &Element) -> String {
    let mut fun = String::new();
    let typ = attr(pointer, "Type");
    let sub_type = attr(pointer, "SubType");
    let normalized_field_name = format!("__{}_handle", name(pointer));
    let return_type = match (allow_multiples(pointer), typ.is_empty()) {
        (true, true) => String::from("Vec<DrawingItem<'a>>"),
        (true, false) => format!("Vec<&'a {}>", typ),
        (false, true) => String::from("Option<DrawingItem<'a>>"),
        (false, false) => format!("Option<&'a {}>", typ),
    };

    // get method
    fun.push_str(&format!(
        "    pub fn {name}<'a>(&self, drawing: &'a Drawing) -> {return_type} {{\n",
        name = name(pointer),
        return_type = return_type
    ));
    if !typ.is_empty() {
        if allow_multiples(pointer) {
            if !sub_type.is_empty() {
                fun.push_str(&format!(
                    "        self.{field}.iter().filter_map(|&h| {{\n",
                    field = normalized_field_name
                ));
                fun.push_str("            match drawing.item_by_handle(h) {\n");
                fun.push_str(&format!(
                    "                Some(DrawingItem::{typ}(val)) => {{\n",
                    typ = typ
                ));
                fun.push_str("                    match val.specific {\n");
                fun.push_str(&format!(
                    "                        {typ}Type::{sub_type}(_) => Some(val),\n",
                    typ = typ,
                    sub_type = sub_type
                ));
                fun.push_str("                        _ => None,\n");
                fun.push_str("                    }\n");
                fun.push_str("                },\n");
                fun.push_str("                _ => None,\n");
                fun.push_str("            }\n");
                fun.push_str("        }).collect()\n");
            } else {
                fun.push_str(&format!(
                    "        self.{field}.iter().filter_map(|&h| {{\n",
                    field = normalized_field_name
                ));
                fun.push_str("            match drawing.item_by_handle(h) {\n");
                fun.push_str(&format!(
                    "                Some(DrawingItem::{typ}(val)) => Some(val),\n",
                    typ = typ
                ));
                fun.push_str("                _ => None,\n");
                fun.push_str("            }\n");
                fun.push_str("        }).collect()\n");
            }
        } else {
            fun.push_str(&format!(
                "        match drawing.item_by_handle(self.{field}) {{\n",
                field = normalized_field_name
            ));
            if !sub_type.is_empty() {
                fun.push_str(&format!(
                    "            Some(DrawingItem::{typ}(val)) => {{\n",
                    typ = typ
                ));
                fun.push_str("                match val.specific {\n");
                fun.push_str(&format!(
                    "                    {typ}Type::{sub_type}(_) => Some(val),\n",
                    typ = typ,
                    sub_type = sub_type
                ));
                fun.push_str("                    _ => None,\n");
                fun.push_str("                }\n");
                fun.push_str("            },\n");
            } else {
                fun.push_str(&format!(
                    "            Some(DrawingItem::{typ}(val)) => Some(val),\n",
                    typ = typ
                ));
            }
            fun.push_str("            _ => None,\n");
            fun.push_str("        }\n");
        }
    } else if allow_multiples(pointer) {
        fun.push_str(&format!(
            "        self.{field}.iter().filter_map(|&h| drawing.item_by_handle(h)).collect()\n",
            field = normalized_field_name
        ));
    } else {
        fun.push_str(&format!(
            "        drawing.item_by_handle(self.{field})\n",
            field = normalized_field_name
        ));
    }

    fun.push_str("    }\n");

    // add/set method
    if allow_multiples(pointer) {
        match (typ.is_empty(), sub_type.is_empty()) {
            (false, false) => {
                // we know the very specific type and should fail if it's not correct
                fun.push_str(&format!(
                    "    pub fn add_{name}(&mut self, item: &{typ}) -> DxfResult<()> {{\n",
                    name = name(pointer),
                    typ = typ
                ));
                fun.push_str("        match item.specific {\n");
                fun.push_str(&format!("            {typ}Type::{sub_type} {{ .. }} => self.{field}.push(item.common.handle),\n",
                    typ=typ, sub_type=sub_type, field=normalized_field_name));
                fun.push_str("            _ => return Err(DxfError::WrongItemType),\n");
                fun.push_str("        }\n");
                fun.push('\n');
                fun.push_str("        Ok(())\n");
            }
            (false, true) => {
                // we know the high level type
                fun.push_str(&format!(
                    "    pub fn add_{name}(&mut self, item: &{typ}) {{\n",
                    name = name(pointer),
                    typ = typ
                ));
                fun.push_str(&format!(
                    "        self.{field}.push(DrawingItem::{typ}(item).handle());\n",
                    field = normalized_field_name,
                    typ = typ
                ));
            }
            (true, true) => {
                // we don't know what type this should be
                fun.push_str(&format!(
                    "    pub fn add_{name}(&mut self, item: &DrawingItemMut) {{\n",
                    name = name(pointer)
                ));
                fun.push_str(&format!(
                    "        self.{field}.push(item.handle());\n",
                    field = normalized_field_name
                ));
            }
            (true, false) => panic!("a specific type was specified without a high level type"),
        }
    } else {
        match (typ.is_empty(), sub_type.is_empty()) {
            (false, false) => {
                // we know the very specific type and should fail if it's not correct
                fun.push_str(&format!(
                    "    pub fn set_{name}(&mut self, item: &{typ}) -> DxfResult<()> {{\n",
                    name = name(pointer),
                    typ = typ
                ));
                fun.push_str("        match item.specific {\n");
                fun.push_str(&format!("            {typ}Type::{sub_type} {{ .. }} => self.{field} = item.common.handle,\n",
                    typ=typ, sub_type=sub_type, field=normalized_field_name));
                fun.push_str("            _ => return Err(DxfError::WrongItemType),\n");
                fun.push_str("        }\n");
                fun.push('\n');
                fun.push_str("        Ok(())\n");
            }
            (false, true) => {
                // we know the high level type
                fun.push_str(&format!(
                    "    pub fn set_{name}(&mut self, item: &{typ}) {{\n",
                    name = name(pointer),
                    typ = typ
                ));
                fun.push_str(&format!(
                    "        self.{field} = DrawingItem::{typ}(item).handle();\n",
                    field = normalized_field_name,
                    typ = typ
                ));
            }
            (true, true) => {
                // we don't know what type this should be
                fun.push_str(&format!(
                    "    pub fn set_{name}(&mut self, item: &DrawingItemMut) {{\n",
                    name = name(pointer)
                ));
                fun.push_str(&format!(
                    "        self.{field} = item.handle();\n",
                    field = normalized_field_name
                ));
            }
            (true, false) => panic!("a specific type was specified without a high level type"),
        }
    }

    fun.push_str("    }\n");
    fun
}

pub fn min_version(element: &Element) -> String {
    attr(element, "MinVersion")
}

pub fn max_version(element: &Element) -> String {
    attr(element, "MaxVersion")
}

pub fn name(element: &Element) -> String {
    attr(element, "Name")
}

pub fn suppress_writing(element: &Element) -> bool {
    attr(element, "SuppressWriting") == "true"
}

pub fn typ(element: &Element) -> String {
    attr(element, "Type")
}

pub fn write_condition(element: &Element) -> String {
    attr(element, "WriteCondition")
}
