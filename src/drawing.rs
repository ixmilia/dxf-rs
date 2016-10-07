// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use entities::*;
use enums::*;
use header::*;
use objects::*;
use tables::*;

use ::{
    CodePair,
    CodePairValue,
    DxfError,
    DxfResult,
    EntityIter,
    ObjectIter,
};

use block::Block;
use class::Class;

use code_pair_iter::CodePairIter;
use code_pair_writer::CodePairWriter;

use std::fs::File;
use std::io::{
    BufReader,
    BufWriter,
    Read,
    Write,
};

use std::path::Path;
use itertools::PutBack;

/// Represents a DXF drawing.
pub struct Drawing {
    /// The drawing's header.  Contains various drawing-specific values and settings.
    pub header: Header,
    /// The classes contained by the drawing.
    pub classes: Vec<Class>,
    /// The AppIds contained by the drawing.
    pub app_ids: Vec<AppId>,
    /// The block records contained by the drawing.
    pub block_records: Vec<BlockRecord>,
    /// The dimension styles contained by the drawing.
    pub dim_styles: Vec<DimStyle>,
    /// The layers contained by the drawing.
    pub layers: Vec<Layer>,
    /// The line types contained by the drawing.
    pub line_types: Vec<LineType>,
    /// The visual styles contained by the drawing.
    pub styles: Vec<Style>,
    /// The user coordinate systems (UCS) contained by the drawing.
    pub ucs: Vec<Ucs>,
    /// The views contained by the drawing.
    pub views: Vec<View>,
    /// The view ports contained by the drawing.
    pub view_ports: Vec<ViewPort>,
    /// The blocks contained by the drawing.
    pub blocks: Vec<Block>,
    /// The entities contained by the drawing.
    pub entities: Vec<Entity>,
    /// The objects contained by the drawing.
    pub objects: Vec<Object>,
}

// public implementation
impl Drawing {
    /// Creates a new empty `Drawing`.
    pub fn new() -> Self {
        Drawing {
            header: Header::new(),
            classes: vec![],
            app_ids: vec![],
            block_records: vec![],
            dim_styles: vec![],
            layers: vec![],
            line_types: vec![],
            styles: vec![],
            ucs: vec![],
            views: vec![],
            view_ports: vec![],
            blocks: vec![],
            entities: vec![],
            objects: vec![],
        }
    }
    /// Loads a `Drawing` from anything that implements the `Read` trait.
    pub fn load<T>(reader: T) -> DxfResult<Drawing>
        where T: Read {

        let reader = CodePairIter::new(reader);
        let mut drawing = Drawing::new();
        let mut iter = PutBack::new(reader);
        try!(Drawing::read_sections(&mut drawing, &mut iter));
        match iter.next() {
            Some(Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "EOF" => Ok(drawing),
            Some(Ok(pair)) => Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/EOF"))),
            Some(Err(e)) => Err(e),
            None => Ok(drawing),
        }
    }
    /// Loads a `Drawing` from disk, using a `BufReader`.
    pub fn load_file(file_name: &str) -> DxfResult<Drawing> {
        let path = Path::new(file_name);
        let file = try!(File::open(&path));
        let buf_reader = BufReader::new(file);
        Drawing::load(buf_reader)
    }
    /// Writes a `Drawing` to anything that implements the `Write` trait.
    pub fn save<T>(&self, writer: &mut T) -> DxfResult<()>
        where T: Write {

        let mut writer = CodePairWriter::new_ascii_writer(writer);
        self.save_internal(&mut writer)
    }
    /// Writes a `Drawing` as binary to anything that implements the `Write` trait.
    pub fn save_binary<T>(&self, writer: &mut T) -> DxfResult<()>
        where T: Write {

        let mut writer = CodePairWriter::new_binary_writer(writer);
        self.save_internal(&mut writer)
    }
    fn save_internal<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        try!(writer.write_prelude());
        try!(self.header.write(writer));
        let write_handles = self.header.version >= AcadVersion::R13 || self.header.handles_enabled;
        try!(self.write_classes(writer));
        try!(self.write_tables(write_handles, writer));
        try!(self.write_blocks(write_handles, writer));
        try!(self.write_entities(write_handles, writer));
        try!(self.write_objects(writer));
        // TODO: write THUMBNAILIMAGE section
        try!(writer.write_code_pair(&CodePair::new_str(0, "EOF")));
        Ok(())
    }
    /// Writes a `Drawing` to disk, using a `BufWriter`.
    pub fn save_file(&self, file_name: &str) -> DxfResult<()> {
        self.save_file_internal(file_name, true)
    }
    /// Writes a `Drawing` as binary to disk, using a `BufWriter`.
    pub fn save_file_binary(&self, file_name: &str) -> DxfResult<()> {
        self.save_file_internal(file_name, false)
    }
    fn save_file_internal(&self, file_name: &str, as_ascii: bool) -> DxfResult<()> {
        let path = Path::new(file_name);
        let file = try!(File::create(&path));
        let buf_writer = BufWriter::new(file);
        let mut writer = match as_ascii {
            true => CodePairWriter::new_ascii_writer(buf_writer),
            false => CodePairWriter::new_binary_writer(buf_writer),
        };
        self.save_internal(&mut writer)
    }
}

// private implementation
impl Drawing {
    fn write_classes<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        if self.classes.len() == 0 {
            return Ok(());
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "CLASSES")));
        for c in &self.classes {
            try!(c.write(&self.header.version, writer));
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn write_tables<T>(&self, write_handles: bool, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "TABLES")));
        try!(write_tables(&self, write_handles, writer));
        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn write_blocks<T>(&self, write_handles: bool, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        if self.blocks.len() == 0 {
            return Ok(());
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "BLOCKS")));
        for b in &self.blocks {
            try!(b.write(&self.header.version, write_handles, writer));
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn write_entities<T>(&self, write_handles: bool, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "ENTITIES")));
        for e in &self.entities {
            try!(e.write(&self.header.version, write_handles, writer));
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn write_objects<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "OBJECTS")));
        for o in &self.objects {
            try!(o.write(&self.header.version, writer));
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn read_sections<I>(drawing: &mut Drawing, iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>> {

        loop {
            match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    match &*try!(pair.value.assert_string()) {
                        "EOF" => {
                            iter.put_back(Ok(pair));
                            break;
                        },
                        "SECTION" => {
                            match iter.next() {
                               Some(Ok(CodePair { code: 2, value: CodePairValue::Str(s) })) => {
                                    match &*s {
                                        "HEADER" => drawing.header = try!(Header::read(iter)),
                                        "CLASSES" => try!(Class::read_classes(drawing, iter)),
                                        "TABLES" => try!(drawing.read_section_item(iter, "TABLE", read_specific_table)),
                                        "BLOCKS" => try!(drawing.read_section_item(iter, "BLOCK", Block::read_block)),
                                        "ENTITIES" => try!(drawing.read_entities(iter)),
                                        "OBJECTS" => try!(drawing.read_objects(iter)),
                                        "THUMBNAILIMAGE" => (), // TODO
                                        _ => try!(Drawing::swallow_section(iter)),
                                    }

                                    match iter.next() {
                                        Some(Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "ENDSEC" => (),
                                        Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/ENDSEC"))),
                                        Some(Err(e)) => return Err(e),
                                        None => return Err(DxfError::UnexpectedEndOfInput),
                                    }
                                },
                                Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 2/<section-name>"))),
                                Some(Err(e)) => return Err(e),
                                None => return Err(DxfError::UnexpectedEndOfInput),
                            }
                        },
                        _ => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/SECTION"))),
                    }
                },
                Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/SECTION or 0/EOF"))),
                Some(Err(e)) => return Err(e),
                None => break, // ideally should have been 0/EOF
            }
        }

        Ok(())
    }
    fn swallow_section<I>(iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>> {

        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 && try!(pair.value.assert_string()) == "ENDSEC" {
                        iter.put_back(Ok(pair));
                        break;
                    }
                },
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(())
    }
    fn read_entities<I>(&mut self, iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>> {

        let mut iter = PutBack::new(EntityIter { iter: iter });
        loop {
            match iter.next() {
                Some(Entity { ref common, specific: EntityType::Insert(ref ins) }) if ins.has_attributes => {
                    let mut ins = ins.clone(); // 12 fields
                    loop {
                        match iter.next() {
                            Some(Entity { specific: EntityType::Attribute(att), .. }) => ins.attributes.push(att),
                            Some(ent) => {
                                // stop gathering on any non-ATTRIBUTE
                                iter.put_back(ent);
                                break;
                            },
                            None => break,
                        }
                    }

                    try!(Drawing::swallow_seqend(&mut iter));

                    // and finally keep the INSERT
                    self.entities.push(Entity {
                        common: common.clone(), // 18 fields
                        specific: EntityType::Insert(ins),
                    })
                },
                Some(Entity { common, specific: EntityType::Polyline(poly) }) => {
                    let mut poly = poly.clone(); // 13 fields
                    loop {
                        match iter.next() {
                            Some(Entity { specific: EntityType::Vertex(vertex), .. }) => poly.vertices.push(vertex),
                            Some(ent) => {
                                // stop gathering on any non-VERTEX
                                iter.put_back(ent);
                                break;
                            },
                            None => break,
                        }
                    }

                    try!(Drawing::swallow_seqend(&mut iter));

                    // and finally keep the POLYLINE
                    self.entities.push(Entity {
                        common: common.clone(), // 18 fields
                        specific: EntityType::Polyline(poly),
                    });
                },
                Some(entity) => self.entities.push(entity),
                None => break,
            }
        }

        Ok(())
    }
    fn read_objects<I>(&mut self, iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>> {

        let mut iter = PutBack::new(ObjectIter { iter: iter });
        loop {
            match iter.next() {
                Some(obj) => self.objects.push(obj),
                None => break,
            }
        }

        Ok(())
    }
    fn swallow_seqend<I>(iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = Entity> {

        match iter.next() {
            Some(Entity { specific: EntityType::Seqend(_), .. }) => (),
            Some(ent) => iter.put_back(ent),
            None => (),
        }

        Ok(())
    }
    fn read_section_item<I, F>(&mut self, iter: &mut PutBack<I>, item_type: &str, callback: F) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>>,
              F: Fn(&mut Drawing, &mut PutBack<I>) -> DxfResult<()> {

        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 {
                        match &*try!(pair.value.assert_string()) {
                            "ENDSEC" => {
                                iter.put_back(Ok(pair));
                                break;
                            },
                            val => {
                                if val == item_type {
                                    try!(callback(self, iter));
                                }
                                else {
                                    return Err(DxfError::UnexpectedCodePair(pair, String::new()));
                                }
                            },
                        }
                    }
                    else {
                        return Err(DxfError::UnexpectedCodePair(pair, String::new()));
                    }
                },
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }

        Ok(())
    }
    #[doc(hidden)]
    pub fn swallow_table<I>(iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>> {

        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 {
                        match &*try!(pair.value.assert_string()) {
                            "TABLE" | "ENDSEC" | "ENDTAB" => {
                                iter.put_back(Ok(pair));
                                break;
                            },
                            _ => (), // swallow the code pair
                        }
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }

        Ok(())
    }
}
