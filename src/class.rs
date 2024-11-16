use crate::{CodePair, Drawing, DxfError, DxfResult};

use crate::code_pair_put_back::CodePairPutBack;
use crate::enums::*;
use crate::helper_functions::*;

/// Represents an application-defined class whose instances are `Block`s, `Entity`s, and `Object`s.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Class {
    /// Class DXF record name.
    pub record_name: String,
    /// C++ class name.  Used to bind with software that defines object class behavior.
    pub class_name: String,
    /// Application name.
    pub application_name: String,
    /// The class's version number.
    pub version_number: i32,
    /// Indicates the capabilities of this object as a proxy.
    pub proxy_capability_flags: i32,
    /// Number of instances of this class.
    pub instance_count: usize,
    /// Was the class loaded with the file.
    pub was_class_loaded_with_file: bool,
    /// Is class derived from the `AcDbEntity` class.
    pub is_entity: bool,
}

// public implementation
impl Class {
    pub fn is_erase_allowed(&self) -> bool {
        self.flag(1)
    }
    pub fn set_is_erase_allowed(&mut self, val: bool) {
        self.set_flag(1, val)
    }
    pub fn is_transform_allowed(&self) -> bool {
        self.flag(2)
    }
    pub fn set_is_transform_allowed(&mut self, val: bool) {
        self.set_flag(2, val)
    }
    pub fn is_color_change_allowed(&self) -> bool {
        self.flag(4)
    }
    pub fn set_is_color_change_allowed(&mut self, val: bool) {
        self.set_flag(4, val)
    }
    pub fn is_layer_change_allowed(&self) -> bool {
        self.flag(8)
    }
    pub fn set_is_layer_change_allowed(&mut self, val: bool) {
        self.set_flag(8, val)
    }
    pub fn is_line_type_change_allowed(&self) -> bool {
        self.flag(16)
    }
    pub fn set_is_line_type_change_allowed(&mut self, val: bool) {
        self.set_flag(16, val)
    }
    pub fn is_line_type_scale_change_allowed(&self) -> bool {
        self.flag(32)
    }
    pub fn set_is_line_type_scale_change_allowed(&mut self, val: bool) {
        self.set_flag(32, val)
    }
    pub fn is_visibility_change_allowed(&self) -> bool {
        self.flag(64)
    }
    pub fn set_is_visibility_change_allowed(&mut self, val: bool) {
        self.set_flag(64, val)
    }
    pub fn is_clone_allowed(&self) -> bool {
        self.flag(128)
    }
    pub fn set_is_clone_allowed(&mut self, val: bool) {
        self.set_flag(128, val)
    }
    pub fn is_lineweight_change_allowed(&self) -> bool {
        self.flag(256)
    }
    pub fn set_is_lineweight_change_allowed(&mut self, val: bool) {
        self.set_flag(256, val)
    }
    pub fn is_plot_style_name_change_allowed(&self) -> bool {
        self.flag(512)
    }
    pub fn set_is_plot_style_name_change_allowed(&mut self, val: bool) {
        self.set_flag(512, val)
    }
    #[allow(non_snake_case)]
    pub fn is_R13_format_proxy(&self) -> bool {
        self.flag(32768)
    }
    #[allow(non_snake_case)]
    pub fn set_is_R13_format_proxy(&mut self, val: bool) {
        self.set_flag(32768, val)
    }
}

impl Default for Class {
    fn default() -> Self {
        Class {
            record_name: String::new(),
            class_name: String::new(),
            application_name: String::new(),
            version_number: 0,
            proxy_capability_flags: 0,
            instance_count: 0,
            was_class_loaded_with_file: true,
            is_entity: false,
        }
    }
}

// internal visibility only
impl Class {
    pub(crate) fn read_classes(drawing: &mut Drawing, iter: &mut CodePairPutBack) -> DxfResult<()> {
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 {
                        match &*pair.assert_string()? {
                            "ENDSEC" => {
                                iter.put_back(Ok(pair));
                                break;
                            }
                            typ => Class::read_class(typ, drawing, iter)?,
                        }
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }

        Ok(())
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) {
        if version >= AcadVersion::R14 {
            pairs.push(CodePair::new_str(0, "CLASS"));
            pairs.push(CodePair::new_string(1, &self.record_name));
            pairs.push(CodePair::new_string(2, &self.class_name));
            pairs.push(CodePair::new_string(3, &self.application_name));
            pairs.push(CodePair::new_i32(90, self.proxy_capability_flags));
            if version >= AcadVersion::R2004 {
                pairs.push(CodePair::new_i32(91, self.instance_count as i32));
            }
        } else {
            pairs.push(CodePair::new_string(0, &self.record_name));
            pairs.push(CodePair::new_string(1, &self.class_name));
            pairs.push(CodePair::new_string(2, &self.application_name));
            pairs.push(CodePair::new_i32(90, self.version_number));
        }

        pairs.push(CodePair::new_i16(
            280,
            as_i16(!self.was_class_loaded_with_file),
        ));
        pairs.push(CodePair::new_i16(281, as_i16(self.is_entity)));
    }
}

// private implementation
impl Class {
    fn read_class(typ: &str, drawing: &mut Drawing, iter: &mut CodePairPutBack) -> DxfResult<()> {
        let mut class = Class::default();

        // R13 has alternate values for the code pairs
        if drawing.header.version <= AcadVersion::R13 {
            class.record_name = typ.to_string();
        }

        loop {
            match iter.next() {
                Some(Ok(pair)) => match pair.code {
                    0 => {
                        iter.put_back(Ok(pair));
                        break;
                    }
                    1 => {
                        if drawing.header.version <= AcadVersion::R13 {
                            class.class_name = pair.assert_string()?;
                        } else {
                            class.record_name = pair.assert_string()?;
                        }
                    }
                    2 => {
                        if drawing.header.version <= AcadVersion::R13 {
                            class.application_name = pair.assert_string()?;
                        } else {
                            class.class_name = pair.assert_string()?;
                        }
                    }
                    3 => {
                        if drawing.header.version >= AcadVersion::R14 {
                            class.application_name = pair.assert_string()?;
                        }
                    }
                    90 => {
                        if drawing.header.version <= AcadVersion::R13 {
                            class.version_number = pair.assert_i32()?;
                        } else {
                            class.proxy_capability_flags = pair.assert_i32()?;
                        }
                    }
                    91 => class.instance_count = pair.assert_i32()? as usize,
                    280 => class.was_class_loaded_with_file = !as_bool(pair.assert_i16()?),
                    281 => class.is_entity = as_bool(pair.assert_i16()?),
                    _ => (),
                },
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }

        drawing.classes.push(class);
        Ok(())
    }
    fn flag(&self, mask: i32) -> bool {
        self.proxy_capability_flags & mask != 0
    }
    fn set_flag(&mut self, mask: i32, val: bool) {
        if val {
            self.proxy_capability_flags |= mask;
        } else {
            self.proxy_capability_flags &= !mask;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::*;
    use crate::helper_functions::tests::*;
    use crate::*;

    fn read_single_class(version_str: &str, body: Vec<CodePair>) -> Class {
        let mut pairs = vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "HEADER"),
            CodePair::new_str(9, "$ACADVER"),
            CodePair::new_str(1, version_str),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "CLASSES"),
        ];
        for pair in body {
            pairs.push(pair);
        }
        pairs.push(CodePair::new_str(0, "ENDSEC"));
        pairs.push(CodePair::new_str(0, "EOF"));
        let drawing = drawing_from_pairs(pairs);
        assert_eq!(1, drawing.classes.len());
        drawing.classes[0].to_owned()
    }

    #[test]
    fn read_empty_classes_section() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "CLASSES"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(0, drawing.classes.len());
    }

    #[test]
    fn read_single_class_r13() {
        let class = read_single_class(
            "AC1012",
            vec![
                CodePair::new_str(0, "record-name"),
                CodePair::new_str(1, "class-name"),
                CodePair::new_str(2, "application-name"),
                CodePair::new_i32(90, 42),
                CodePair::new_i32(91, 43),
                CodePair::new_i16(280, 1),
                CodePair::new_i16(281, 1),
            ],
        );
        assert_eq!("record-name", class.record_name);
        assert_eq!("class-name", class.class_name);
        assert_eq!("application-name", class.application_name);
        assert_eq!(42, class.version_number);
        assert_eq!(43, class.instance_count);
        assert_eq!(0, class.proxy_capability_flags);
        assert!(!class.was_class_loaded_with_file);
        assert!(class.is_entity);
    }

    #[test]
    fn read_single_class_r14() {
        let class = read_single_class(
            "AC1015",
            vec![
                CodePair::new_str(0, "CLASS"),
                CodePair::new_str(1, "record-name"),
                CodePair::new_str(2, "class-name"),
                CodePair::new_str(3, "application-name"),
                CodePair::new_i32(90, 42),
                CodePair::new_i32(91, 43),
                CodePair::new_i16(280, 1),
                CodePair::new_i16(281, 1),
            ],
        );
        assert_eq!("record-name", class.record_name);
        assert_eq!("class-name", class.class_name);
        assert_eq!("application-name", class.application_name);
        assert_eq!(42, class.proxy_capability_flags);
        assert_eq!(43, class.instance_count);
        assert_eq!(0, class.version_number);
        assert!(!class.was_class_loaded_with_file);
        assert!(class.is_entity);
    }

    #[test]
    fn read_multiple_classes_r13() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "HEADER"),
            CodePair::new_str(9, "$ACADVER"),
            CodePair::new_str(1, "AC1012"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "CLASSES"),
            CodePair::new_str(0, "some class 1"),
            CodePair::new_str(0, "some class 2"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(2, drawing.classes.len());
    }

    #[test]
    fn read_multiple_classes_r14() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "HEADER"),
            CodePair::new_str(9, "$ACADVER"),
            CodePair::new_str(1, "AC1014"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "CLASSES"),
            CodePair::new_str(0, "CLASS"),
            CodePair::new_str(0, "CLASS"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(2, drawing.classes.len());
    }

    #[test]
    fn dont_write_classes_section_if_no_classes() {
        let drawing = Drawing::new();
        assert_not_contains_pairs(&drawing, vec![CodePair::new_str(0, "CLASSES")]);
    }

    #[test]
    fn write_class_r13() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R13;
        let class = Class {
            record_name: "record-name".to_string(),
            class_name: "class-name".to_string(),
            application_name: "application-name".to_string(),
            version_number: 42,
            proxy_capability_flags: 43,
            instance_count: 44,
            was_class_loaded_with_file: false,
            is_entity: true,
        };
        drawing.classes.push(class);
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "CLASSES"),
                CodePair::new_str(0, "record-name"),
                CodePair::new_str(1, "class-name"),
                CodePair::new_str(2, "application-name"),
                CodePair::new_i32(90, 42),
                CodePair::new_i16(280, 1),
                CodePair::new_i16(281, 1),
                CodePair::new_str(0, "ENDSEC"),
            ],
        );
    }

    #[test]
    fn write_class_r14() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14;
        let class = Class {
            record_name: "record-name".to_string(),
            class_name: "class-name".to_string(),
            application_name: "application-name".to_string(),
            version_number: 42,
            proxy_capability_flags: 43,
            instance_count: 44,
            was_class_loaded_with_file: false,
            is_entity: true,
        };
        drawing.classes.push(class);
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "CLASSES"),
                CodePair::new_str(0, "CLASS"),
                CodePair::new_str(1, "record-name"),
                CodePair::new_str(2, "class-name"),
                CodePair::new_str(3, "application-name"),
                CodePair::new_i32(90, 43),
                CodePair::new_i16(280, 1),
                CodePair::new_i16(281, 1),
                CodePair::new_str(0, "ENDSEC"),
            ],
        );
    }
}
