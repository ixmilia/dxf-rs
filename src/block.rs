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

#[cfg(test)]
mod tests {
    use crate::entities::*;
    use crate::enums::*;
    use crate::helper_functions::tests::*;
    use crate::*;

    fn read_blocks_section(content: Vec<&str>) -> Drawing {
        let mut file = String::new();
        file.push_str(vec!["0", "SECTION", "2", "BLOCKS"].join("\n").as_str());
        file.push('\n');
        for line in content {
            file.push_str(line);
            file.push('\n');
        }
        file.push_str(vec!["0", "ENDSEC", "0", "EOF"].join("\n").as_str());
        parse_drawing(file.as_str())
    }

    fn read_single_block(content: Vec<&str>) -> Block {
        let mut full_block = vec![];
        full_block.push("0");
        full_block.push("BLOCK");
        for line in content {
            full_block.push(line);
        }
        full_block.push("0");
        full_block.push("ENDBLK");
        let drawing = read_blocks_section(full_block);
        assert_eq!(1, drawing.blocks.len());
        drawing.blocks[0].to_owned()
    }

    #[test]
    fn read_empty_blocks_section_2() {
        let drawing = read_blocks_section(vec![]);
        assert_eq!(0, drawing.blocks.len());
    }

    #[test]
    fn read_empty_block() {
        let _block = read_single_block(vec![]);
    }

    #[test]
    fn read_block_specific_values() {
        let block = read_single_block(vec![
            "2",
            "block-name",
            "10",
            "1.1",
            "20",
            "2.2",
            "30",
            "3.3",
        ]);
        assert_eq!("block-name", block.name);
        assert_eq!(0, block.entities.len());
        assert_eq!(Point::new(1.1, 2.2, 3.3), block.base_point);
    }

    #[test]
    fn read_with_end_block_values() {
        // these values should be ignored
        let drawing = read_blocks_section(vec![
            "0",
            "BLOCK",
            "0",
            "ENDBLK",
            "5",
            "1", // handle
            "330",
            "2", // owner handle
            "100",
            "AcDbEntity",
            "8",
            "layer-name",
            "100",
            "AcDbBlockEnd",
        ]);
        assert_eq!(1, drawing.blocks.len());
    }

    #[test]
    fn read_multiple_blocks() {
        let drawing = read_blocks_section(vec![
            "0", "BLOCK", "0", "ENDBLK", "0", "BLOCK", "0", "ENDBLK",
        ]);
        assert_eq!(2, drawing.blocks.len())
    }

    #[test]
    fn read_block_with_single_entity() {
        let block = read_single_block(vec![
            "0", "LINE", "10", "1.1", "20", "2.2", "30", "3.3", "11", "4.4", "21", "5.5", "31",
            "6.6",
        ]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(ref line) => {
                assert_eq!(Point::new(1.1, 2.2, 3.3), line.p1);
                assert_eq!(Point::new(4.4, 5.5, 6.6), line.p2);
            }
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_multiple_entities() {
        let block = read_single_block(vec!["0", "LINE", "0", "CIRCLE"]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
        match block.entities[1].specific {
            EntityType::Circle(_) => (),
            _ => panic!("expected a circle"),
        }
    }

    #[test]
    fn read_block_with_unsupported_entity_first() {
        let block = read_single_block(vec!["0", "UNSUPPORTED_ENTITY", "0", "LINE"]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_unsupported_entity_last() {
        let block = read_single_block(vec!["0", "LINE", "0", "UNSUPPORTED_ENTITY"]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_unsupported_entity_in_the_middle() {
        let block = read_single_block(vec!["0", "LINE", "0", "UNSUPPORTED_ENTITY", "0", "CIRCLE"]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
        match block.entities[1].specific {
            EntityType::Circle(_) => (),
            _ => panic!("expected a circle"),
        }
    }

    #[test]
    fn read_block_with_polyline() {
        let block = read_single_block(vec![
            "0", "POLYLINE", "0", "VERTEX", "0", "VERTEX", "0", "VERTEX", "0", "SEQEND",
        ]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(3, p.vertices.len());
            }
            _ => panic!("expected a polyline"),
        }
    }

    #[test]
    fn read_block_with_polyline_and_another_entity() {
        let block = read_single_block(vec![
            "0", "POLYLINE", "0", "VERTEX", "0", "VERTEX", "0", "VERTEX", "0", "SEQEND", "0",
            "LINE",
        ]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(3, p.vertices.len());
            }
            _ => panic!("expected a polyline"),
        }
        match block.entities[1].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_polyline_without_seqend_and_another_entity() {
        let block = read_single_block(vec![
            "0", "POLYLINE", "0", "VERTEX", "0", "VERTEX", "0", "VERTEX", "0", "LINE",
        ]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(3, p.vertices.len());
            }
            _ => panic!("expected a polyline"),
        }
        match block.entities[1].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_empty_polyline_without_seqend_and_another_entity() {
        let block = read_single_block(vec!["0", "POLYLINE", "0", "LINE"]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(0, p.vertices.len());
            }
            _ => panic!("expected a polyline"),
        }
        match block.entities[1].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn dont_write_blocks_section_if_no_blocks() {
        let drawing = Drawing::default();
        let contents = to_test_string(&drawing);
        assert!(!contents.contains("BLOCKS"));
    }

    #[test]
    fn read_extension_group_data() {
        let block = read_single_block(vec![
            "102",
            "{IXMILIA",
            "  1",
            "some string",
            "102",
            "{NESTED",
            " 10",
            "1.1",
            "102",
            "}",
            "102",
            "}",
        ]);
        assert_eq!(1, block.extension_data_groups.len());
        let x = &block.extension_data_groups[0];
        assert_eq!("IXMILIA", x.application_name);
        assert_eq!(2, x.items.len());
        match x.items[0] {
            ExtensionGroupItem::CodePair(ref p) => {
                assert_eq!(&CodePair::new_str(1, "some string"), p)
            }
            _ => panic!("expected a code pair"),
        }
        match x.items[1] {
            ExtensionGroupItem::Group(ref g) => {
                assert_eq!("NESTED", g.application_name);
                assert_eq!(1, g.items.len());
                match g.items[0] {
                    ExtensionGroupItem::CodePair(ref p) => {
                        assert_eq!(&CodePair::new_f64(10, 1.1), p)
                    }
                    _ => panic!("expected a code pair"),
                }
            }
            _ => panic!("expected a nested group"),
        }
    }

    #[test]
    fn write_extension_group_data() {
        let mut block = Block::default();
        block.extension_data_groups.push(ExtensionGroup {
            application_name: String::from("IXMILIA"),
            items: vec![
                ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string")),
                ExtensionGroupItem::Group(ExtensionGroup {
                    application_name: String::from("NESTED"),
                    items: vec![ExtensionGroupItem::CodePair(CodePair::new_f64(10, 1.1))],
                }),
            ],
        });
        let mut drawing = Drawing::default();
        drawing.header.version = AcadVersion::R14; // extension group data only written on >= R14
        drawing.blocks.push(block);
        assert_contains(
            &drawing,
            vec![
                "102",
                "{IXMILIA",
                "  1",
                "some string",
                "102",
                "{NESTED",
                " 10",
                "1.1",
                "102",
                "}",
                "102",
                "}",
            ]
            .join("\r\n"),
        );
    }

    #[test]
    fn read_x_data() {
        let block = read_single_block(vec![
            "1001",
            "IXMILIA",
            "1000",
            "some string",
            "1002",
            "{",
            "1040",
            "1.1",
            "1002",
            "}",
        ]);
        assert_eq!(1, block.x_data.len());
        let x = &block.x_data[0];
        assert_eq!("IXMILIA", x.application_name);
        assert_eq!(2, x.items.len());
        match x.items[0] {
            XDataItem::Str(ref s) => assert_eq!("some string", s),
            _ => panic!("expected a string"),
        }
        match x.items[1] {
            XDataItem::ControlGroup(ref items) => {
                assert_eq!(1, items.len());
                match items[0] {
                    XDataItem::Real(r) => assert!(approx_eq!(f64, 1.1, r)),
                    _ => panic!("expected a real"),
                }
            }
            _ => panic!("expected a control group"),
        }
    }

    #[test]
    fn write_x_data() {
        let mut block = Block::default();
        block.x_data.push(XData {
            application_name: String::from("IXMILIA"),
            items: vec![
                XDataItem::Str(String::from("some string")),
                XDataItem::ControlGroup(vec![XDataItem::Real(1.1)]),
            ],
        });
        let mut drawing = Drawing::default();
        drawing.header.version = AcadVersion::R2000; // xdata only written on >= R2000
        drawing.blocks.push(block);
        assert_contains(
            &drawing,
            vec![
                "1001",
                "IXMILIA",
                "1000",
                "some string",
                "1002",
                "{",
                "1040",
                "1.1",
                "1002",
                "}",
            ]
            .join("\r\n"),
        );
    }

    #[test]
    fn round_trip_blocks() {
        let mut drawing = Drawing::default();
        let mut b1 = Block::default();
        b1.entities.push(Entity {
            common: Default::default(),
            specific: EntityType::Line(Default::default()),
        });
        drawing.blocks.push(b1);
        let mut b2 = Block::default();
        b2.entities.push(Entity {
            common: Default::default(),
            specific: EntityType::Circle(Default::default()),
        });
        drawing.blocks.push(b2);
        let written = to_test_string(&drawing);
        let reparsed = unwrap_drawing(Drawing::load(&mut written.as_bytes()));
        assert_eq!(2, reparsed.blocks.len());
        assert_eq!(1, reparsed.blocks[0].entities.len());
        match reparsed.blocks[0].entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
        assert_eq!(1, reparsed.blocks[1].entities.len());
        match reparsed.blocks[1].entities[0].specific {
            EntityType::Circle(_) => (),
            _ => panic!("expected a circle"),
        }
    }
}
