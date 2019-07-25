// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Write;

use {CodePair, Drawing, DxfError, DxfResult};

use code_pair_writer::CodePairWriter;
use enums::*;
use helper_functions::*;

use itertools::PutBack;

/// Represents an application-defined class whose instances are `Block`s, `Entity`s, and `Object`s.
#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
    pub fn get_is_erase_allowed(&self) -> bool {
        self.get_flag(1)
    }
    pub fn set_is_erase_allowed(&mut self, val: bool) {
        self.set_flag(1, val)
    }
    pub fn get_is_transform_allowed(&self) -> bool {
        self.get_flag(2)
    }
    pub fn set_is_transform_allowed(&mut self, val: bool) {
        self.set_flag(2, val)
    }
    pub fn get_is_color_change_allowed(&self) -> bool {
        self.get_flag(4)
    }
    pub fn set_is_color_change_allowed(&mut self, val: bool) {
        self.set_flag(4, val)
    }
    pub fn get_is_layer_change_allowed(&self) -> bool {
        self.get_flag(8)
    }
    pub fn set_is_layer_change_allowed(&mut self, val: bool) {
        self.set_flag(8, val)
    }
    pub fn get_is_line_type_change_allowed(&self) -> bool {
        self.get_flag(16)
    }
    pub fn set_is_line_type_change_allowed(&mut self, val: bool) {
        self.set_flag(16, val)
    }
    pub fn get_is_line_type_scale_change_allowed(&self) -> bool {
        self.get_flag(32)
    }
    pub fn set_is_line_type_scale_change_allowed(&mut self, val: bool) {
        self.set_flag(32, val)
    }
    pub fn get_is_visibility_change_allowed(&self) -> bool {
        self.get_flag(64)
    }
    pub fn set_is_visibility_change_allowed(&mut self, val: bool) {
        self.set_flag(64, val)
    }
    pub fn get_is_clone_allowed(&self) -> bool {
        self.get_flag(128)
    }
    pub fn set_is_clone_allowed(&mut self, val: bool) {
        self.set_flag(128, val)
    }
    pub fn get_is_lineweight_change_allowed(&self) -> bool {
        self.get_flag(256)
    }
    pub fn set_is_lineweight_change_allowed(&mut self, val: bool) {
        self.set_flag(256, val)
    }
    pub fn get_is_plot_style_name_change_allowed(&self) -> bool {
        self.get_flag(512)
    }
    pub fn set_is_plot_style_name_change_allowed(&mut self, val: bool) {
        self.set_flag(512, val)
    }
    #[allow(non_snake_case)]
    pub fn get_is_R13_format_proxy(&self) -> bool {
        self.get_flag(32768)
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
    pub(crate) fn read_classes<I>(drawing: &mut Drawing, iter: &mut PutBack<I>) -> DxfResult<()>
    where
        I: Iterator<Item = DxfResult<CodePair>>,
    {
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
    pub(crate) fn write<T>(
        &self,
        version: AcadVersion,
        writer: &mut CodePairWriter<T>,
    ) -> DxfResult<()>
    where
        T: Write,
    {
        if version >= AcadVersion::R14 {
            writer.write_code_pair(&CodePair::new_str(0, "CLASS"))?;
            writer.write_code_pair(&CodePair::new_string(1, &self.record_name))?;
            writer.write_code_pair(&CodePair::new_string(2, &self.class_name))?;
            writer.write_code_pair(&CodePair::new_string(3, &self.application_name))?;
            writer.write_code_pair(&CodePair::new_i32(90, self.proxy_capability_flags))?;
            if version >= AcadVersion::R2004 {
                writer.write_code_pair(&CodePair::new_i32(91, self.instance_count as i32))?;
            }
        } else {
            writer.write_code_pair(&CodePair::new_string(0, &self.record_name))?;
            writer.write_code_pair(&CodePair::new_string(1, &self.class_name))?;
            writer.write_code_pair(&CodePair::new_string(2, &self.application_name))?;
            writer.write_code_pair(&CodePair::new_i32(90, self.version_number))?;
        }

        writer.write_code_pair(&CodePair::new_i16(
            280,
            as_i16(!self.was_class_loaded_with_file),
        ))?;
        writer.write_code_pair(&CodePair::new_i16(281, as_i16(self.is_entity)))?;

        Ok(())
    }
}

// private implementation
impl Class {
    fn read_class<I>(typ: &str, drawing: &mut Drawing, iter: &mut PutBack<I>) -> DxfResult<()>
    where
        I: Iterator<Item = DxfResult<CodePair>>,
    {
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
    fn get_flag(&self, mask: i32) -> bool {
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
