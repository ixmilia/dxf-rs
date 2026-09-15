use crate::{
    CodePair, CodePairValue, Drawing, DrawingItem, DrawingItemMut, DxfError, DxfResult,
    ExtensionGroup, Handle, Point, XData,
};

use crate::code_pair_put_back::CodePairPutBack;
use crate::entities::Entity;
use crate::entity_iter::EntityIter;
use crate::enums::*;
use crate::extension_data;
use crate::helper_functions::*;
use crate::x_data;

/// A block is a collection of entities.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// The block's handle.
    pub handle: Handle,
    /// The handle of the block's `ENDBLK`.
    ///
    /// `ENDBLK` is a separate object and needs a handle of its own; reusing the
    /// block's puts the same handle on two objects. Assigned when the block is
    /// added to a `Drawing`, or taken from the file when one is read.
    #[doc(hidden)]
    pub __endblk_handle: Handle,
    #[doc(hidden)]
    pub __owner_handle: Handle,
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
    /// `XData`.
    pub x_data: Vec<XData>,
}

// public implementation
impl Block {
    pub fn owner<'a>(&self, drawing: &'a Drawing) -> Option<DrawingItem<'a>> {
        drawing.item_by_handle(self.__owner_handle)
    }
    pub fn set_owner<'a>(&mut self, item: &'a mut DrawingItemMut, drawing: &'a mut Drawing) {
        self.__owner_handle = drawing.assign_and_get_handle(item);
    }
    pub fn is_anonymous(&self) -> bool {
        self.flag(1)
    }
    pub fn set_is_anonymous(&mut self, val: bool) {
        self.set_flag(1, val)
    }
    pub fn has_non_consistent_attribute_definitions(&self) -> bool {
        self.flag(2)
    }
    pub fn set_has_non_consistent_attribute_definitions(&mut self, val: bool) {
        self.set_flag(2, val)
    }
    pub fn is_xref(&self) -> bool {
        self.flag(4)
    }
    pub fn set_is_xref(&mut self, val: bool) {
        self.set_flag(4, val)
    }
    pub fn is_xref_overlay(&self) -> bool {
        self.flag(8)
    }
    pub fn set_is_xref_overlay(&mut self, val: bool) {
        self.set_flag(8, val)
    }
    pub fn is_externally_dependent(&self) -> bool {
        self.flag(16)
    }
    pub fn set_is_externally_dependent(&mut self, val: bool) {
        self.set_flag(16, val)
    }
    pub fn is_referenced_external_reference(&self) -> bool {
        self.flag(32)
    }
    pub fn set_is_referenced_external_reference(&mut self, val: bool) {
        self.set_flag(32, val)
    }
    pub fn is_resolved_external_reference(&self) -> bool {
        self.flag(64)
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
            handle: Handle::empty(),
            __endblk_handle: Handle::empty(),
            __owner_handle: Handle::empty(),
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
    pub(crate) fn read_block(drawing: &mut Drawing, iter: &mut CodePairPutBack) -> DxfResult<()> {
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
                            // swallow all non-0 code pairs, but keep the
                            // `ENDBLK` handle: it belongs to that object, and
                            // writing the file again has to give it back.
                            loop {
                                match iter.next() {
                                    Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                        // done reading ENDBLK
                                        iter.put_back(Ok(pair));
                                        break;
                                    }
                                    Some(Ok(pair @ CodePair { code: 5, .. })) => {
                                        current.__endblk_handle = pair.as_handle()?;
                                    }
                                    Some(Ok(_)) => (), // swallow this
                                    Some(Err(e)) => return Err(e),
                                    None => return Err(DxfError::UnexpectedEndOfInput),
                                }
                            }

                            if current.handle.is_empty() {
                                drawing.add_block(current);
                            } else {
                                drawing.add_block_no_handle_set(current);
                            }
                            break;
                        }
                        CodePair {
                            code: 0,
                            value: CodePairValue::Str(ref s),
                            ..
                        } if s == "ENDSEC" || s == "EOF" => {
                            // The block never reached its `ENDBLK`.  Put the pair
                            // back so the enclosing section reader sees it, and
                            // finalize the block with what was read so far; saving
                            // the drawing again writes the missing `ENDBLK`.
                            //
                            // Without this the read does not terminate:
                            // `Entity::read()` returns `Ok(None)` for `0/ENDSEC`
                            // *without consuming the pair*, so the loop below took
                            // the same pair again on every trip, allocating each
                            // time.
                            iter.put_back(Ok(pair));
                            if current.handle.is_empty() {
                                drawing.add_block(current);
                            } else {
                                drawing.add_block_no_handle_set(current);
                            }
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
    pub(crate) fn add_code_pairs(
        &self,
        pairs: &mut Vec<CodePair>,
        version: AcadVersion,
        write_handles: bool,
    ) {
        pairs.push(CodePair::new_str(0, "BLOCK"));
        if write_handles && version >= AcadVersion::R13 {
            pairs.push(CodePair::new_string(5, &self.handle.as_string()));
        }

        if version >= AcadVersion::R14 {
            for group in &self.extension_data_groups {
                group.add_code_pairs(pairs);
            }
        }

        if version >= AcadVersion::R13 {
            if !self.__owner_handle.is_empty() {
                pairs.push(CodePair::new_string(330, &self.__owner_handle.as_string()));
            }

            pairs.push(CodePair::new_str(100, "AcDbEntity"));
        }

        if self.is_in_paperspace {
            pairs.push(CodePair::new_i16(67, as_i16(self.is_in_paperspace)));
        }

        pairs.push(CodePair::new_string(8, &self.layer));
        if version >= AcadVersion::R13 {
            pairs.push(CodePair::new_str(100, "AcDbBlockBegin"));
        }

        pairs.push(CodePair::new_string(2, &self.name));
        pairs.push(CodePair::new_i16(70, self.flags as i16));
        pairs.push(CodePair::new_f64(10, self.base_point.x));
        pairs.push(CodePair::new_f64(20, self.base_point.y));
        pairs.push(CodePair::new_f64(30, self.base_point.z));
        if version >= AcadVersion::R12 {
            pairs.push(CodePair::new_string(3, &self.name));
        }

        pairs.push(CodePair::new_string(1, &self.xref_path_name));
        if !self.description.is_empty() {
            pairs.push(CodePair::new_string(4, &self.description));
        }

        for e in &self.entities {
            e.add_code_pairs(pairs, version, write_handles);
        }

        pairs.push(CodePair::new_str(0, "ENDBLK"));
        if write_handles {
            // `ENDBLK` is its own object: it must not repeat the block's handle.
            let endblk = if self.__endblk_handle.is_empty() {
                self.handle
            } else {
                self.__endblk_handle
            };
            if !endblk.is_empty() {
                pairs.push(CodePair::new_string(5, &endblk.as_string()));
            }
        }

        if version >= AcadVersion::R14 {
            for group in &self.extension_data_groups {
                group.add_code_pairs(pairs);
            }
        }

        if version >= AcadVersion::R2000 && !self.__owner_handle.is_empty() {
            pairs.push(CodePair::new_string(330, &self.__owner_handle.as_string()));
        }

        if version >= AcadVersion::R13 {
            pairs.push(CodePair::new_str(100, "AcDbEntity"));
        }

        if self.is_in_paperspace {
            pairs.push(CodePair::new_i16(67, as_i16(self.is_in_paperspace)));
        }

        pairs.push(CodePair::new_string(8, &self.layer));
        if version >= AcadVersion::R13 {
            pairs.push(CodePair::new_str(100, "AcDbBlockEnd"));
        }

        for x in &self.x_data {
            x.add_code_pairs(pairs, version);
        }
    }
}

// private implementation
impl Block {
    fn flag(&self, mask: i32) -> bool {
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
    use crate::code_pair_iter::DirectCodePairIter;
    use crate::entities::*;
    use crate::enums::*;
    use crate::helper_functions::tests::*;
    use crate::*;
    use float_cmp::approx_eq;

    fn read_blocks_section(content: Vec<CodePair>) -> Drawing {
        let mut pairs = vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "BLOCKS"),
        ];
        for pair in content {
            pairs.push(pair);
        }
        pairs.push(CodePair::new_str(0, "ENDSEC"));
        pairs.push(CodePair::new_str(0, "EOF"));
        drawing_from_pairs(pairs)
    }

    fn read_single_block(content: Vec<CodePair>) -> Block {
        let mut pairs = vec![CodePair::new_str(0, "BLOCK")];
        for pair in content {
            pairs.push(pair);
        }
        pairs.push(CodePair::new_str(0, "ENDBLK"));
        let drawing = read_blocks_section(pairs);
        let blocks = drawing.blocks().collect::<Vec<_>>();
        assert_eq!(1, blocks.len());
        blocks[0].clone()
    }

    fn assert_block_contains(block: Block, version: AcadVersion, expected: Vec<CodePair>) {
        let mut drawing = Drawing::new();
        let block = drawing.add_block(block);
        let mut pairs = Vec::new();
        block.add_code_pairs(&mut pairs, version, true);
        assert_vec_contains(&pairs, &expected);
    }

    /// Every handle written in the file, in order.
    fn written_handles(drawing: &Drawing) -> Vec<String> {
        let written = to_test_string(drawing);
        let lines: Vec<&str> = written.lines().collect();
        lines
            .windows(2)
            .filter(|w| w[0].trim() == "5")
            .map(|w| w[1].trim().to_string())
            .collect()
    }

    fn assert_handles_are_unique(drawing: &Drawing) {
        let handles = written_handles(drawing);
        let mut unique = handles.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            handles.len(),
            unique.len(),
            "every handle in the file is unique: {handles:?}"
        );
    }

    /// `ENDBLK` carries its own handle, not the block's.
    ///
    /// A handle identifies exactly one object in a DXF file. Writing the
    /// block's handle on its `ENDBLK` puts the same value on two objects, and
    /// a reader that indexes by handle has to repair the file before using it:
    /// ezdxf reports a non-unique entity handle on load.
    #[test]
    fn endblk_handle_differs_from_block_handle() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2000;
        let block = Block {
            name: String::from("some-block"),
            ..Default::default()
        };
        drawing.add_block(block);

        assert_handles_are_unique(&drawing);
    }

    /// A block recovered without its `ENDBLK` gets a unique handle for it too.
    ///
    /// The recovery added in #105 finalizes a block that never reached its
    /// `ENDBLK`, and saving writes the missing one. That `ENDBLK` is a new
    /// object and cannot borrow the block's handle either.
    #[test]
    fn recovered_block_endblk_handle_is_unique() {
        let mut drawing = read_blocks_section(vec![
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(5, "A1"),
            CodePair::new_str(2, "block-without-endblk"),
        ]);
        // R13+ writes the block's handle, which is where a duplicate shows.
        drawing.header.version = AcadVersion::R2000;
        let block = drawing.blocks().next().expect("the block is recovered");
        assert_eq!(Handle(0xA1), block.handle);
        assert_ne!(
            block.handle, block.__endblk_handle,
            "the recovered ENDBLK has a handle of its own"
        );
        assert_handles_are_unique(&drawing);
    }

    /// An `ENDBLK` handle that came from a file is given back when written.
    ///
    /// The handle belongs to that object, and a reader that kept a reference to
    /// it - an owner pointer, say - still finds it after a round trip.
    #[test]
    fn endblk_handle_round_trips() {
        let drawing = read_blocks_section(vec![
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(5, "A1"),
            CodePair::new_str(2, "some-block"),
            CodePair::new_str(0, "ENDBLK"),
            CodePair::new_str(5, "A2"),
        ]);
        let blocks = drawing.blocks().collect::<Vec<_>>();
        assert_eq!(1, blocks.len());
        assert_eq!(Handle(0xA1), blocks[0].handle);
        assert_eq!(Handle(0xA2), blocks[0].__endblk_handle);

        // Writing it again gives the same handle back.
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "ENDBLK"),
                CodePair::new_str(5, "A2"),
            ],
        );
    }

    /// A `BLOCK` that never reaches its `ENDBLK` is recovered, not rejected.
    ///
    /// Three things are asserted, and the first is the one that matters most:
    /// that the call **returns**. The rest come after. If the defect came back
    /// this test would not fail - it would hang, and CI would time out rather
    /// than go red. That is a property of the defect, not of the test.
    #[test]
    fn read_block_without_endblk_recovers() {
        let drawing = read_blocks_section(vec![
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(2, "block-without-endblk"),
            CodePair::new_str(5, "ABC"),
            CodePair::new_str(8, "some-layer"),
        ]);

        // The block is kept, and what it declared survives.
        let blocks = drawing.blocks().collect::<Vec<_>>();
        assert_eq!(1, blocks.len());
        assert_eq!("block-without-endblk", blocks[0].name);
        assert_eq!("some-layer", blocks[0].layer);
    }

    /// A file that ends without even an `ENDSEC` terminates too - with an
    /// error, which is the right answer.
    ///
    /// Recovering the block does not make the *section* well formed: the pair
    /// is put back and the section reader rejects it, one level up, which is
    /// where that judgement belongs. What matters here is the same thing as
    /// above: the call returns instead of spinning.
    #[test]
    fn read_block_without_endblk_or_endsec_terminates() {
        let pairs = vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "BLOCKS"),
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(2, "block-without-endblk"),
            CodePair::new_str(0, "EOF"),
        ];
        let iter = Box::new(DirectCodePairIter::new(pairs));
        assert!(Drawing::load_from_iter(iter).is_err());
    }

    /// Saving a recovered drawing writes the `ENDBLK` that was missing, and
    /// reading it back gives the same block.
    ///
    /// This is what makes the recovery worth more than an error: the file is
    /// repaired by a round trip rather than merely tolerated.
    #[test]
    fn recovered_block_round_trips_with_its_endblk() {
        let drawing = read_blocks_section(vec![
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(2, "block-without-endblk"),
            CodePair::new_str(8, "some-layer"),
        ]);

        let written = to_test_string(&drawing);
        assert!(
            written.contains("ENDBLK"),
            "saving the drawing writes the missing ENDBLK"
        );

        let again = parse_drawing(&written);
        let blocks = again.blocks().collect::<Vec<_>>();
        assert_eq!(1, blocks.len());
        assert_eq!("block-without-endblk", blocks[0].name);
        assert_eq!("some-layer", blocks[0].layer);
    }

    #[test]
    fn read_empty_blocks_section_2() {
        let drawing = read_blocks_section(vec![]);
        assert_eq!(0, drawing.blocks().count());
    }

    #[test]
    fn read_empty_block() {
        let _block = read_single_block(vec![]);
    }

    #[test]
    fn read_block_specific_values() {
        let block = read_single_block(vec![
            CodePair::new_string(2, "block-name"),
            CodePair::new_f64(10, 1.1),
            CodePair::new_f64(20, 2.2),
            CodePair::new_f64(30, 3.3),
        ]);
        assert_eq!("block-name", block.name);
        assert_eq!(0, block.entities.len());
        assert_eq!(Point::new(1.1, 2.2, 3.3), block.base_point);
    }

    #[test]
    fn read_with_end_block_values() {
        // these values should be ignored
        let drawing = read_blocks_section(vec![
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(0, "ENDBLK"),
            CodePair::new_str(5, "1"),   // handle
            CodePair::new_str(330, "2"), // owner handle
            CodePair::new_str(100, "AcDbEntity"),
            CodePair::new_str(8, "layer-name"),
            CodePair::new_str(100, "AcDbBlockEnd"),
        ]);
        assert_eq!(1, drawing.blocks().count());
    }

    #[test]
    fn read_multiple_blocks() {
        let drawing = read_blocks_section(vec![
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(0, "ENDBLK"),
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(0, "ENDBLK"),
        ]);
        assert_eq!(2, drawing.blocks().count())
    }

    #[test]
    fn read_block_with_single_entity() {
        let block = read_single_block(vec![
            CodePair::new_str(0, "LINE"),
            CodePair::new_f64(10, 1.1),
            CodePair::new_f64(20, 2.2),
            CodePair::new_f64(30, 3.3),
            CodePair::new_f64(11, 4.4),
            CodePair::new_f64(21, 5.5),
            CodePair::new_f64(31, 6.6),
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
        let block = read_single_block(vec![
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(0, "CIRCLE"),
        ]);
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
        let block = read_single_block(vec![
            CodePair::new_str(0, "UNSUPPORTED_ENTITY"),
            CodePair::new_str(0, "LINE"),
        ]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_unsupported_entity_last() {
        let block = read_single_block(vec![
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(0, "UNSUPPORTED_ENTITY"),
        ]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn read_block_with_unsupported_entity_in_the_middle() {
        let block = read_single_block(vec![
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(0, "UNSUPPORTED_ENTITY"),
            CodePair::new_str(0, "CIRCLE"),
        ]);
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
            CodePair::new_str(0, "POLYLINE"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "SEQEND"),
        ]);
        assert_eq!(1, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(3, p.vertices().count());
            }
            _ => panic!("expected a polyline"),
        }
    }

    #[test]
    fn read_block_with_polyline_and_another_entity() {
        let block = read_single_block(vec![
            CodePair::new_str(0, "POLYLINE"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "SEQEND"),
            CodePair::new_str(0, "LINE"),
        ]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(3, p.vertices().count());
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
            CodePair::new_str(0, "POLYLINE"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "VERTEX"),
            CodePair::new_str(0, "LINE"),
        ]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(3, p.vertices().count());
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
        let block = read_single_block(vec![
            CodePair::new_str(0, "POLYLINE"),
            CodePair::new_str(0, "LINE"),
        ]);
        assert_eq!(2, block.entities.len());
        match block.entities[0].specific {
            EntityType::Polyline(ref p) => {
                assert_eq!(0, p.vertices().count());
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
        let drawing = Drawing::new();
        assert_not_contains_pairs(&drawing, vec![CodePair::new_str(0, "BLOCKS")]);
    }

    #[test]
    fn read_extension_group_data() {
        let block = read_single_block(vec![
            CodePair::new_str(102, "{IXMILIA"),
            CodePair::new_str(1, "some string"),
            CodePair::new_str(102, "{NESTED"),
            CodePair::new_f64(10, 1.1),
            CodePair::new_str(102, "}"),
            CodePair::new_str(102, "}"),
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
        assert_block_contains(
            block,
            AcadVersion::R14, // block data is only supported in R14
            vec![
                CodePair::new_str(102, "{IXMILIA"),
                CodePair::new_str(1, "some string"),
                CodePair::new_str(102, "{NESTED"),
                CodePair::new_f64(10, 1.1),
                CodePair::new_str(102, "}"),
                CodePair::new_str(102, "}"),
            ],
        );
    }

    #[test]
    fn read_x_data() {
        let block = read_single_block(vec![
            CodePair::new_str(1001, "IXMILIA"),
            CodePair::new_str(1000, "some string"),
            CodePair::new_str(1002, "{"),
            CodePair::new_f64(1040, 1.1),
            CodePair::new_str(1002, "}"),
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
        assert_block_contains(
            block,
            AcadVersion::R2000, // xdata only written on >= R2000
            vec![
                CodePair::new_str(1001, "IXMILIA"),
                CodePair::new_str(1000, "some string"),
                CodePair::new_str(1002, "{"),
                CodePair::new_f64(1040, 1.1),
                CodePair::new_str(1002, "}"),
            ],
        );
    }

    #[test]
    fn round_trip_blocks() {
        let mut drawing = Drawing::new();
        let mut b1 = Block::default();
        b1.entities.push(Entity {
            common: Default::default(),
            specific: EntityType::Line(Default::default()),
        });
        drawing.add_block(b1);
        let mut b2 = Block::default();
        b2.entities.push(Entity {
            common: Default::default(),
            specific: EntityType::Circle(Default::default()),
        });
        drawing.add_block(b2);

        let drawing_pairs = drawing.code_pairs().unwrap();
        let reparsed = drawing_from_pairs(drawing_pairs);

        let blocks = reparsed.blocks().collect::<Vec<_>>();
        assert_eq!(2, blocks.len());
        assert_eq!(1, blocks[0].entities.len());
        match blocks[0].entities[0].specific {
            EntityType::Line(_) => (),
            _ => panic!("expected a line"),
        }
        assert_eq!(1, blocks[1].entities.len());
        match blocks[1].entities[0].specific {
            EntityType::Circle(_) => (),
            _ => panic!("expected a circle"),
        }
    }

    /// Test case derived from <https://ezdxf.readthedocs.io/en/stable/dxfinternals/block_management.html>
    #[test]
    fn write_block_r12_compat() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R12;
        let mut block = Block {
            name: "block-name".to_string(),
            ..Default::default()
        };
        block.entities.push(Entity {
            common: Default::default(),
            specific: EntityType::Line(Line::new(
                Point::new(0.0, 0.0, 0.0),
                Point::new(1.0, 1.0, 0.0),
            )),
        });
        drawing.add_block(block);
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "BLOCKS"),
                CodePair::new_str(0, "BLOCK"),
                // no handle
                CodePair::new_str(8, "0"),          // layer
                CodePair::new_str(2, "block-name"), // name
                CodePair::new_i16(70, 0),           // flags
                CodePair::new_f64(10, 0.0),         // insertion point
                CodePair::new_f64(20, 0.0),
                CodePair::new_f64(30, 0.0),
                CodePair::new_str(3, "block-name"), // name again
                CodePair::new_str(1, ""),           // x-ref name; empty = external
                CodePair::new_str(0, "LINE"),       // first entity
                CodePair::new_str(5, "13"),         // entity handle
            ],
        );
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "ENDBLK"),
                // `ENDBLK` has a handle of its own; the block's is not written
                // in R12.
                CodePair::new_str(5, "11"),
                CodePair::new_str(8, "0"),  // layer
                CodePair::new_str(0, "ENDSEC"), // end of block
            ],
        );
    }
}
