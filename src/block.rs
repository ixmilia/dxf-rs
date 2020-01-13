// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::{Read, Write};

use crate::{
    CodePair, CodePairValue, Drawing, DrawingItem, DrawingItemMut, DxfError, DxfResult,
    ExtensionGroup, Point, XData,
};

use crate::code_pair_put_back::CodePairPutBack;
use crate::code_pair_writer::CodePairWriter;
use crate::entities::Entity;
use crate::entity_iter::EntityIter;
use crate::enums::*;
use crate::extension_data;
use crate::handle_tracker::HandleTracker;
use crate::helper_functions::*;
use crate::x_data;

/// A block is a collection of entities.
#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Block {
    /// The block's handle.
    pub handle: u32,
    #[doc(hidden)]
    pub __owner_handle: u32,
    /// The name of the layer containing the block.
    pub layer: String,
    /// The name of the block.
    pub name: String,
    /// Block-type flags.
    pub flags: i32,
    /// The block's base insertion point.
    pub base_point: Point,
    /// The path name of the XREF.
    pub xref_path_name: String,
    /// The block's description.
    pub description: String,
    /// If the block is in PAPERSPACE or not.
    pub is_in_paperspace: bool,
    /// The entities contained by the block.
    pub entities: Vec<Entity>,
    /// Extension data groups.
    pub extension_data_groups: Vec<ExtensionGroup>,
    /// XData.
    pub x_data: Vec<XData>,
}

// public implementation
impl Block {
    pub fn get_owner<'a>(&self, drawing: &'a Drawing) -> Option<DrawingItem<'a>> {
        drawing.get_item_by_handle(self.__owner_handle)
    }
    pub fn set_owner<'a>(&mut self, item: &'a mut DrawingItemMut, drawing: &'a mut Drawing) {
        self.__owner_handle = drawing.assign_and_get_handle(item);
    }
    pub fn get_is_anonymous(&self) -> bool {
        self.get_flag(1)
    }
    pub fn set_is_anonymous(&mut self, val: bool) {
        self.set_flag(1, val)
    }
    pub fn has_non_consistent_attribute_definitions(&self) -> bool {
        self.get_flag(2)
    }
    pub fn set_has_non_consistent_attribute_definitions(&mut self, val: bool) {
        self.set_flag(2, val)
    }
    pub fn get_is_xref(&self) -> bool {
        self.get_flag(4)
    }
    pub fn set_is_xref(&mut self, val: bool) {
        self.set_flag(4, val)
    }
    pub fn get_is_xref_overlay(&self) -> bool {
        self.get_flag(8)
    }
    pub fn set_is_xref_overlay(&mut self, val: bool) {
        self.set_flag(8, val)
    }
    pub fn get_is_externally_dependent(&self) -> bool {
        self.get_flag(16)
    }
    pub fn set_is_externally_dependent(&mut self, val: bool) {
        self.set_flag(16, val)
    }
    pub fn get_is_referenced_external_reference(&self) -> bool {
        self.get_flag(32)
    }
    pub fn set_is_referenced_external_reference(&mut self, val: bool) {
        self.set_flag(32, val)
    }
    pub fn get_is_resolved_external_reference(&self) -> bool {
        self.get_flag(64)
    }
    pub fn set_is_resolved_external_reference(&mut self, val: bool) {
        self.set_flag(64, val)
    }
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        default_if_empty(&mut self.layer, "0");
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            handle: 0,
            __owner_handle: 0,
            layer: String::from("0"),
            name: String::new(),
            flags: 0,
            base_point: Point::origin(),
            xref_path_name: String::new(),
            description: String::new(),
            is_in_paperspace: false,
            entities: vec![],
            extension_data_groups: vec![],
            x_data: vec![],
        }
    }
}

// internal visibility only
impl Block {
    pub(crate) fn read_block<I>(
        drawing: &mut Drawing,
        iter: &mut CodePairPutBack<I>,
    ) -> DxfResult<()>
    where
        I: Read,
    {
        // match code pair:
        //   0/ENDBLK -> swallow code pairs and return
        //   0/* -> read entity and add to collection
        //   */* -> apply to block
        let mut current = Block::default();
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    match pair {
                        CodePair {
                            code: 0,
                            value: CodePairValue::Str(ref s),
                            ..
                        } if s == "ENDBLK" => {
                            // swallow all non-0 code pairs
                            loop {
                                match iter.next() {
                                    Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                        // done reading ENDBLK
                                        iter.put_back(Ok(pair));
                                        break;
                                    }
                                    Some(Ok(_)) => (), // swallow this
                                    Some(Err(e)) => return Err(e),
                                    None => return Err(DxfError::UnexpectedEndOfInput),
                                }
                            }

                            drawing.blocks.push(current);
                            break;
                        }
                        CodePair { code: 0, .. } => {
                            // should be an entity
                            iter.put_back(Ok(pair));
                            let mut iter = EntityIter { iter };
                            iter.read_entities_into_vec(&mut current.entities)?;
                        }
                        _ => {
                            // specific to the BLOCK
                            match pair.code {
                                1 => current.xref_path_name = pair.assert_string()?,
                                2 => current.name = pair.assert_string()?,
                                3 => (), // another instance of the name
                                4 => current.description = pair.assert_string()?,
                                5 => current.handle = pair.as_handle()?,
                                8 => current.layer = pair.assert_string()?,
                                10 => current.base_point.x = pair.assert_f64()?,
                                20 => current.base_point.y = pair.assert_f64()?,
                                30 => current.base_point.z = pair.assert_f64()?,
                                67 => current.is_in_paperspace = as_bool(pair.assert_i16()?),
                                70 => current.flags = i32::from(pair.assert_i16()?),
                                330 => current.__owner_handle = pair.as_handle()?,
                                extension_data::EXTENSION_DATA_GROUP => {
                                    let group = ExtensionGroup::read_group(
                                        pair.assert_string()?,
                                        iter,
                                        pair.offset,
                                    )?;
                                    current.extension_data_groups.push(group);
                                }
                                x_data::XDATA_APPLICATIONNAME => {
                                    let x = XData::read_item(pair.assert_string()?, iter)?;
                                    current.x_data.push(x);
                                }
                                _ => (), // unsupported code pair
                            }
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
        write_handles: bool,
        writer: &mut CodePairWriter<T>,
        handle_tracker: &mut HandleTracker,
    ) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "BLOCK"))?;
        if write_handles && self.handle != 0 {
            writer.write_code_pair(&CodePair::new_string(
                5,
                &as_handle(handle_tracker.get_block_handle(&self)),
            ))?;
        }

        if version >= AcadVersion::R14 {
            for group in &self.extension_data_groups {
                group.write(writer)?;
            }
        }

        if version >= AcadVersion::R13 {
            if self.__owner_handle != 0 {
                writer
                    .write_code_pair(&CodePair::new_string(330, &as_handle(self.__owner_handle)))?;
            }

            writer.write_code_pair(&CodePair::new_str(100, "AcDbEntity"))?;
        }

        if self.is_in_paperspace {
            writer.write_code_pair(&CodePair::new_i16(67, as_i16(self.is_in_paperspace)))?;
        }

        writer.write_code_pair(&CodePair::new_string(8, &self.layer))?;
        if version >= AcadVersion::R13 {
            writer.write_code_pair(&CodePair::new_str(100, "AcDbBlockBegin"))?;
        }

        writer.write_code_pair(&CodePair::new_string(2, &self.name))?;
        writer.write_code_pair(&CodePair::new_i16(70, self.flags as i16))?;
        writer.write_code_pair(&CodePair::new_f64(10, self.base_point.x))?;
        writer.write_code_pair(&CodePair::new_f64(20, self.base_point.y))?;
        writer.write_code_pair(&CodePair::new_f64(30, self.base_point.z))?;
        if version >= AcadVersion::R12 {
            writer.write_code_pair(&CodePair::new_string(3, &self.name))?;
        }

        writer.write_code_pair(&CodePair::new_string(1, &self.xref_path_name))?;
        if !self.description.is_empty() {
            writer.write_code_pair(&CodePair::new_string(4, &self.description))?;
        }

        for e in &self.entities {
            e.write(version, false, writer, &mut HandleTracker::new(0))?; // entities in blocks never have handles
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDBLK"))?;
        if write_handles && self.handle != 0 {
            writer.write_code_pair(&CodePair::new_string(5, &as_handle(self.handle)))?;
        }

        if version >= AcadVersion::R14 {
            for group in &self.extension_data_groups {
                group.write(writer)?;
            }
        }

        if version >= AcadVersion::R2000 && self.__owner_handle != 0 {
            writer.write_code_pair(&CodePair::new_string(330, &as_handle(self.__owner_handle)))?;
        }

        if version >= AcadVersion::R13 {
            writer.write_code_pair(&CodePair::new_str(100, "AcDbEntity"))?;
        }

        if self.is_in_paperspace {
            writer.write_code_pair(&CodePair::new_i16(67, as_i16(self.is_in_paperspace)))?;
        }

        writer.write_code_pair(&CodePair::new_string(8, &self.layer))?;
        if version >= AcadVersion::R13 {
            writer.write_code_pair(&CodePair::new_str(100, "AcDbBlockEnd"))?;
        }

        for x in &self.x_data {
            x.write(version, writer)?;
        }

        Ok(())
    }
}

// private implementation
impl Block {
    fn get_flag(&self, mask: i32) -> bool {
        self.flags & mask != 0
    }
    fn set_flag(&mut self, mask: i32, val: bool) {
        if val {
            self.flags |= mask;
        } else {
            self.flags &= !mask;
        }
    }
}
