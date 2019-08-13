// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate byteorder;
use self::byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

extern crate image;
use self::image::DynamicImage;

use code_pair_put_back::CodePairPutBack;
use drawing_item::{DrawingItem, DrawingItemMut};
use entities::*;
use enums::*;
use header::*;
use objects::*;
use tables::*;

use {CodePair, CodePairValue, DxfError, DxfResult};

use dxb_reader::DxbReader;
use dxb_writer::DxbWriter;
use entity_iter::EntityIter;
use handle_tracker::HandleTracker;
use helper_functions::*;
use object_iter::ObjectIter;

use block::Block;
use class::Class;

use code_pair_iter::CodePairIter;
use code_pair_writer::CodePairWriter;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

use itertools::put_back;
use std::collections::HashSet;
use std::iter::Iterator;
use std::path::Path;

/// Represents a DXF drawing.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
    pub ucss: Vec<Ucs>,
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
    /// The thumbnail image preview of the drawing.
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub thumbnail: Option<DynamicImage>,
}

impl Default for Drawing {
    fn default() -> Self {
        Drawing {
            header: Header::default(),
            classes: vec![],
            app_ids: vec![],
            block_records: vec![],
            dim_styles: vec![],
            layers: vec![],
            line_types: vec![],
            styles: vec![],
            ucss: vec![],
            views: vec![],
            view_ports: vec![],
            blocks: vec![],
            entities: vec![],
            objects: vec![],
            thumbnail: None,
        }
    }
}

// public implementation
impl Drawing {
    /// Loads a `Drawing` from anything that implements the `Read` trait.
    pub fn load<T>(reader: &mut T) -> DxfResult<Drawing>
    where
        T: Read + ?Sized,
    {
        let first_line = match read_line(reader) {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(e),
            None => return Err(DxfError::UnexpectedEndOfInput),
        };
        match &*first_line {
            "AutoCAD DXB 1.0" => {
                let mut reader = DxbReader::new(reader);
                reader.load()
            }
            _ => {
                let reader = CodePairIter::new(reader, first_line);
                let mut drawing = Drawing::default();
                drawing.clear();
                let mut iter = CodePairPutBack::from_code_pair_iter(reader);
                Drawing::read_sections(&mut drawing, &mut iter)?;
                match iter.next() {
                    Some(Ok(CodePair {
                        code: 0,
                        value: CodePairValue::Str(ref s),
                        ..
                    })) if s == "EOF" => Ok(drawing),
                    Some(Ok(pair)) => Err(DxfError::UnexpectedCodePair(
                        pair,
                        String::from("expected 0/EOF"),
                    )),
                    Some(Err(e)) => Err(e),
                    None => Ok(drawing),
                }
            }
        }
    }
    /// Loads a `Drawing` from disk, using a `BufReader`.
    pub fn load_file(file_name: &str) -> DxfResult<Drawing> {
        let path = Path::new(file_name);
        let file = File::open(&path)?;
        let mut buf_reader = BufReader::new(file);
        Drawing::load(&mut buf_reader)
    }
    /// Writes a `Drawing` to anything that implements the `Write` trait.
    pub fn save<T>(&self, writer: &mut T) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        self.save_internal(writer, true)
    }
    /// Writes a `Drawing` as binary to anything that implements the `Write` trait.
    pub fn save_binary<T>(&self, writer: &mut T) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        self.save_internal(writer, false)
    }
    fn save_internal<T>(&self, writer: &mut T, as_ascii: bool) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        let text_as_ascii = self.header.version <= AcadVersion::R2004;

        // write to memory while tracking the used handle values
        let mut buf = vec![];
        let mut handle_tracker = HandleTracker::new(self.header.next_available_handle);
        {
            let mut code_pair_writer =
                CodePairWriter::new(&mut buf, as_ascii, text_as_ascii, self.header.version);
            let write_handles =
                self.header.version >= AcadVersion::R13 || self.header.handles_enabled;
            self.write_classes(&mut code_pair_writer)?;
            self.write_tables(write_handles, &mut code_pair_writer, &mut handle_tracker)?;
            self.write_blocks(write_handles, &mut code_pair_writer, &mut handle_tracker)?;
            self.write_entities(write_handles, &mut code_pair_writer, &mut handle_tracker)?;
            self.write_objects(&mut code_pair_writer, &mut handle_tracker)?;
            self.write_thumbnail(&mut code_pair_writer)?;
            code_pair_writer.write_code_pair(&CodePair::new_str(0, "EOF"))?;
        }

        // write header to the final location
        {
            let mut final_writer =
                CodePairWriter::new(writer, as_ascii, text_as_ascii, self.header.version);
            final_writer.write_prelude()?;
            self.header
                .write(&mut final_writer, handle_tracker.get_current_next_handle())?;
        }

        // copy memory to final location
        writer.write_all(&*buf)?;
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
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        self.save_internal(&mut writer, as_ascii)
    }
    /// Writes a `Drawing` as DXB to anything that implements the `Write` trait.
    pub fn save_dxb<T>(&self, writer: &mut T) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        let mut writer = DxbWriter::new(writer);
        writer.write(self)
    }
    /// Writes a `Drawing` as DXB to disk, using a `BufWriter`.
    pub fn save_file_dxb(&self, file_name: &str) -> DxfResult<()> {
        let path = Path::new(file_name);
        let file = File::create(&path)?;
        let mut buf_writer = BufWriter::new(file);
        self.save_dxb(&mut buf_writer)
    }
    /// Clears all items from the `Drawing`.
    pub fn clear(&mut self) {
        self.classes.clear();
        self.app_ids.clear();
        self.block_records.clear();
        self.dim_styles.clear();
        self.layers.clear();
        self.line_types.clear();
        self.styles.clear();
        self.ucss.clear();
        self.views.clear();
        self.view_ports.clear();
        self.blocks.clear();
        self.entities.clear();
        self.objects.clear();
        self.thumbnail = None;
    }
    /// Normalizes the `Drawing` by ensuring expected items are present.
    pub fn normalize(&mut self) {
        // TODO: check for duplicates
        self.header.normalize();
        self.normalize_blocks();
        self.normalize_entities();
        self.normalize_objects();
        self.normalize_app_ids();
        self.normalize_block_records();
        self.normalize_layers();
        self.normalize_text_styles();
        self.normalize_view_ports();
        self.normalize_views();
        self.ensure_mline_styles();
        self.ensure_dimension_styles();
        self.ensure_layers();
        self.ensure_line_types();
        self.ensure_text_styles();
        self.ensure_view_ports();
        self.ensure_views();
        self.ensure_ucs();

        self.app_ids.sort_by(|a, b| a.name.cmp(&b.name));
        self.block_records.sort_by(|a, b| a.name.cmp(&b.name));
        self.dim_styles.sort_by(|a, b| a.name.cmp(&b.name));
        self.layers.sort_by(|a, b| a.name.cmp(&b.name));
        self.line_types.sort_by(|a, b| a.name.cmp(&b.name));
        self.styles.sort_by(|a, b| a.name.cmp(&b.name));
        self.ucss.sort_by(|a, b| a.name.cmp(&b.name));
        self.views.sort_by(|a, b| a.name.cmp(&b.name));
        self.view_ports.sort_by(|a, b| a.name.cmp(&b.name));
    }
    /// Gets a `DrawingItem` with the appropriate handle or `None`.
    pub fn get_item_by_handle(&'_ self, handle: u32) -> Option<DrawingItem<'_>> {
        for item in &self.app_ids {
            if item.handle == handle {
                return Some(DrawingItem::AppId(item));
            }
        }
        for item in &self.blocks {
            if item.handle == handle {
                return Some(DrawingItem::Block(item));
            }
        }
        for item in &self.block_records {
            if item.handle == handle {
                return Some(DrawingItem::BlockRecord(item));
            }
        }
        for item in &self.dim_styles {
            if item.handle == handle {
                return Some(DrawingItem::DimStyle(item));
            }
        }
        for item in &self.entities {
            if item.common.handle == handle {
                return Some(DrawingItem::Entity(item));
            }
        }
        for item in &self.layers {
            if item.handle == handle {
                return Some(DrawingItem::Layer(item));
            }
        }
        for item in &self.line_types {
            if item.handle == handle {
                return Some(DrawingItem::LineType(item));
            }
        }
        for item in &self.objects {
            if item.common.handle == handle {
                return Some(DrawingItem::Object(item));
            }
        }
        for item in &self.styles {
            if item.handle == handle {
                return Some(DrawingItem::Style(item));
            }
        }
        for item in &self.ucss {
            if item.handle == handle {
                return Some(DrawingItem::Ucs(item));
            }
        }
        for item in &self.views {
            if item.handle == handle {
                return Some(DrawingItem::View(item));
            }
        }
        for item in &self.view_ports {
            if item.handle == handle {
                return Some(DrawingItem::ViewPort(item));
            }
        }

        None
    }
    /// Gets a `DrawingItemMut` with the appropriate handle or `None`.
    pub fn get_item_by_handle_mut(&'_ mut self, handle: u32) -> Option<DrawingItemMut<'_>> {
        for item in &mut self.app_ids {
            if item.handle == handle {
                return Some(DrawingItemMut::AppId(item));
            }
        }
        for item in &mut self.blocks {
            if item.handle == handle {
                return Some(DrawingItemMut::Block(item));
            }
        }
        for item in &mut self.block_records {
            if item.handle == handle {
                return Some(DrawingItemMut::BlockRecord(item));
            }
        }
        for item in &mut self.dim_styles {
            if item.handle == handle {
                return Some(DrawingItemMut::DimStyle(item));
            }
        }
        for item in &mut self.entities {
            if item.common.handle == handle {
                return Some(DrawingItemMut::Entity(item));
            }
        }
        for item in &mut self.layers {
            if item.handle == handle {
                return Some(DrawingItemMut::Layer(item));
            }
        }
        for item in &mut self.line_types {
            if item.handle == handle {
                return Some(DrawingItemMut::LineType(item));
            }
        }
        for item in &mut self.objects {
            if item.common.handle == handle {
                return Some(DrawingItemMut::Object(item));
            }
        }
        for item in &mut self.styles {
            if item.handle == handle {
                return Some(DrawingItemMut::Style(item));
            }
        }
        for item in &mut self.ucss {
            if item.handle == handle {
                return Some(DrawingItemMut::Ucs(item));
            }
        }
        for item in &mut self.views {
            if item.handle == handle {
                return Some(DrawingItemMut::View(item));
            }
        }
        for item in &mut self.view_ports {
            if item.handle == handle {
                return Some(DrawingItemMut::ViewPort(item));
            }
        }

        None
    }
    pub(crate) fn assign_and_get_handle(&mut self, item: &mut DrawingItemMut) -> u32 {
        if item.get_handle() == 0 {
            item.set_handle(self.header.next_available_handle);
            self.header.next_available_handle += 1;
        }

        item.get_handle()
    }
}

// private implementation
impl Drawing {
    fn write_classes<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write,
    {
        if self.classes.is_empty() {
            return Ok(());
        }

        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "CLASSES"))?;
        for c in &self.classes {
            c.write(self.header.version, writer)?;
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_tables<T>(
        &self,
        write_handles: bool,
        writer: &mut CodePairWriter<T>,
        handle_tracker: &mut HandleTracker,
    ) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "TABLES"))?;
        write_tables(&self, write_handles, writer, handle_tracker)?;
        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_blocks<T>(
        &self,
        write_handles: bool,
        writer: &mut CodePairWriter<T>,
        handle_tracker: &mut HandleTracker,
    ) -> DxfResult<()>
    where
        T: Write,
    {
        if self.blocks.is_empty() {
            return Ok(());
        }

        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "BLOCKS"))?;
        for b in &self.blocks {
            b.write(self.header.version, write_handles, writer, handle_tracker)?;
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_entities<T>(
        &self,
        write_handles: bool,
        writer: &mut CodePairWriter<T>,
        handle_tracker: &mut HandleTracker,
    ) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "ENTITIES"))?;
        for e in &self.entities {
            e.write(self.header.version, write_handles, writer, handle_tracker)?;
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_objects<T>(
        &self,
        writer: &mut CodePairWriter<T>,
        handle_tracker: &mut HandleTracker,
    ) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "OBJECTS"))?;
        for o in &self.objects {
            o.write(self.header.version, writer, handle_tracker)?;
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_thumbnail<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write,
    {
        if &self.header.version >= &AcadVersion::R2000 {
            if let Some(ref img) = self.thumbnail {
                writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
                writer.write_code_pair(&CodePair::new_str(2, "THUMBNAILIMAGE"))?;
                let mut data = vec![];
                img.save(&mut data, image::ImageFormat::BMP)?;
                let length = data.len() - 14; // skip 14 byte bmp header
                writer.write_code_pair(&CodePair::new_i32(90, length as i32))?;
                for s in data[14..].chunks(128) {
                    let mut line = String::new();
                    for b in s {
                        line.push_str(&format!("{:02X}", b));
                    }
                    writer.write_code_pair(&CodePair::new_string(310, &line))?;
                }
                writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
            }
        }
        Ok(())
    }
    fn read_sections<T>(drawing: &mut Drawing, iter: &mut CodePairPutBack<T>) -> DxfResult<()>
    where
        T: Read,
    {
        loop {
            match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => match &*pair.assert_string()? {
                    "EOF" => {
                        iter.put_back(Ok(pair));
                        break;
                    }
                    "SECTION" => match iter.next() {
                        Some(Ok(CodePair {
                            code: 2,
                            value: CodePairValue::Str(s),
                            ..
                        })) => {
                            match &*s {
                                "HEADER" => drawing.header = Header::read(iter)?,
                                "CLASSES" => Class::read_classes(drawing, iter)?,
                                "TABLES" => {
                                    drawing.read_section_item(iter, "TABLE", read_specific_table)?
                                }
                                "BLOCKS" => {
                                    drawing.read_section_item(iter, "BLOCK", Block::read_block)?
                                }
                                "ENTITIES" => drawing.read_entities(iter)?,
                                "OBJECTS" => drawing.read_objects(iter)?,
                                "THUMBNAILIMAGE" => {
                                    let _ = drawing.read_thumbnail(iter)?;
                                }
                                _ => Drawing::swallow_section(iter)?,
                            }

                            match iter.next() {
                                Some(Ok(CodePair {
                                    code: 0,
                                    value: CodePairValue::Str(ref s),
                                    ..
                                })) if s == "ENDSEC" => (),
                                Some(Ok(pair)) => {
                                    return Err(DxfError::UnexpectedCodePair(
                                        pair,
                                        String::from("expected 0/ENDSEC"),
                                    ))
                                }
                                Some(Err(e)) => return Err(e),
                                None => return Err(DxfError::UnexpectedEndOfInput),
                            }
                        }
                        Some(Ok(pair)) => {
                            return Err(DxfError::UnexpectedCodePair(
                                pair,
                                String::from("expected 2/<section-name>"),
                            ))
                        }
                        Some(Err(e)) => return Err(e),
                        None => return Err(DxfError::UnexpectedEndOfInput),
                    },
                    _ => {
                        return Err(DxfError::UnexpectedCodePair(
                            pair,
                            String::from("expected 0/SECTION"),
                        ))
                    }
                },
                Some(Ok(pair)) => {
                    return Err(DxfError::UnexpectedCodePair(
                        pair,
                        String::from("expected 0/SECTION or 0/EOF"),
                    ))
                }
                Some(Err(e)) => return Err(e),
                None => break, // ideally should have been 0/EOF
            }
        }

        Ok(())
    }
    fn swallow_section<T>(iter: &mut CodePairPutBack<T>) -> DxfResult<()>
    where
        T: Read,
    {
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 && pair.assert_string()? == "ENDSEC" {
                        iter.put_back(Ok(pair));
                        break;
                    }
                }
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(())
    }
    fn read_entities<T>(&mut self, iter: &mut CodePairPutBack<T>) -> DxfResult<()>
    where
        T: Read,
    {
        let mut iter = EntityIter { iter };
        iter.read_entities_into_vec(&mut self.entities)?;
        Ok(())
    }
    fn read_objects<T>(&mut self, iter: &mut CodePairPutBack<T>) -> DxfResult<()>
    where
        T: Read,
    {
        let iter = put_back(ObjectIter { iter });
        for obj in iter {
            self.objects.push(obj);
        }

        Ok(())
    }
    fn read_thumbnail<T>(&mut self, iter: &mut CodePairPutBack<T>) -> DxfResult<bool>
    where
        T: Read,
    {
        // get the length; we don't really care about this since we'll just read whatever's there
        let length_pair = next_pair!(iter);
        let _length = match length_pair.code {
            90 => length_pair.assert_i32()? as usize,
            _ => {
                return Err(DxfError::UnexpectedCode(
                    length_pair.code,
                    length_pair.offset,
                ))
            }
        };

        // prepend the BMP header that always seems to be missing from DXF files
        let mut data: Vec<u8> = vec![
            b'B', b'M', // magic number
            0x00, 0x00, 0x00, 0x00, // file length (calculated later)
            0x00, 0x00, // reserved
            0x00, 0x00, // reserved
            0x00, 0x00, 0x00, 0x00, // image data offset (calculated later)
        ];
        let header_length = data.len();
        let file_length_offset = 2;
        let image_data_offset_offset = 10;

        // read the hex data
        loop {
            match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    // likely 0/ENDSEC
                    iter.put_back(Ok(pair));
                    break;
                }
                Some(Ok(pair @ CodePair { code: 310, .. })) => {
                    parse_hex_string(&pair.assert_string()?, &mut data, pair.offset)?;
                }
                Some(Ok(pair)) => {
                    return Err(DxfError::UnexpectedCode(pair.code, pair.offset));
                }
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        // set the file length
        let mut length_bytes = vec![];
        length_bytes.write_i32::<LittleEndian>(data.len() as i32)?;
        data[file_length_offset] = length_bytes[0];
        data[file_length_offset + 1] = length_bytes[1];
        data[file_length_offset + 2] = length_bytes[2];
        data[file_length_offset + 3] = length_bytes[3];

        // calculate the image data offset
        let dib_header_size = LittleEndian::read_i32(&data[header_length..]) as usize;

        // calculate the palette size
        let palette_size = match dib_header_size {
            40 => {
                // BITMAPINFOHEADER
                let bpp = LittleEndian::read_u16(&data[header_length + 14..]) as usize;
                let palette_color_count =
                    LittleEndian::read_u32(&data[header_length + 32..]) as usize;
                bpp * palette_color_count
            }
            _ => return Ok(false),
        };

        // set the image data offset
        let image_data_offset = header_length + dib_header_size + palette_size;
        let mut offset_bytes = vec![];
        offset_bytes.write_i32::<LittleEndian>(image_data_offset as i32)?;
        data[image_data_offset_offset] = offset_bytes[0];
        data[image_data_offset_offset + 1] = offset_bytes[1];
        data[image_data_offset_offset + 2] = offset_bytes[2];
        data[image_data_offset_offset + 3] = offset_bytes[3];

        let image = image::load_from_memory(&data)?;
        self.thumbnail = Some(image);
        Ok(true)
    }
    fn read_section_item<I, F>(
        &mut self,
        iter: &mut CodePairPutBack<I>,
        item_type: &str,
        callback: F,
    ) -> DxfResult<()>
    where
        I: Read,
        F: Fn(&mut Drawing, &mut CodePairPutBack<I>) -> DxfResult<()>,
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
                            val => {
                                if val == item_type {
                                    callback(self, iter)?;
                                } else {
                                    return Err(DxfError::UnexpectedCodePair(pair, String::new()));
                                }
                            }
                        }
                    } else {
                        return Err(DxfError::UnexpectedCodePair(pair, String::new()));
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }

        Ok(())
    }
    pub(crate) fn swallow_table<I>(iter: &mut CodePairPutBack<I>) -> DxfResult<()>
    where
        I: Read,
    {
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 {
                        match &*pair.assert_string()? {
                            "TABLE" | "ENDSEC" | "ENDTAB" => {
                                iter.put_back(Ok(pair));
                                break;
                            }
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
    fn normalize_blocks(&mut self) {
        for i in 0..self.blocks.len() {
            self.blocks[i].normalize();
        }
    }
    fn normalize_entities(&mut self) {
        for i in 0..self.entities.len() {
            self.entities[i].normalize();
        }
    }
    fn normalize_objects(&mut self) {
        for i in 0..self.objects.len() {
            self.objects[i].normalize();
        }
    }
    fn normalize_app_ids(&mut self) {
        // gather existing app ids
        let mut existing_app_ids = HashSet::new();
        for app_id in &self.app_ids {
            add_to_existing(&mut existing_app_ids, &app_id.name);
        }

        // prepare app ids that should exist
        let should_exist = vec![
            String::from("ACAD"),
            String::from("ACADANNOTATIVE"),
            String::from("ACAD_NAV_VCDISPLAY"),
            String::from("ACAD_MLEADERVER"),
        ];

        // ensure all app ids that should exist do
        for name in &should_exist {
            if !existing_app_ids.contains(name) {
                existing_app_ids.insert(name.clone());
                self.app_ids.push(AppId {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn normalize_block_records(&mut self) {
        // gather existing block records
        let mut existing_block_records = HashSet::new();
        for block_record in &self.block_records {
            add_to_existing(&mut existing_block_records, &block_record.name);
        }

        // prepare block records that should exist
        let should_exist = vec![String::from("*MODEL_SPACE"), String::from("*PAPER_SPACE")];

        // ensure all block records that should exist do
        for name in &should_exist {
            if !existing_block_records.contains(name) {
                existing_block_records.insert(name.clone());
                self.block_records.push(BlockRecord {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn normalize_layers(&mut self) {
        for i in 0..self.layers.len() {
            self.layers[i].normalize();
        }
    }
    fn normalize_text_styles(&mut self) {
        for i in 0..self.styles.len() {
            self.styles[i].normalize();
        }
    }
    fn normalize_view_ports(&mut self) {
        for i in 0..self.view_ports.len() {
            self.view_ports[i].normalize();
        }
    }
    fn normalize_views(&mut self) {
        for i in 0..self.views.len() {
            self.views[i].normalize();
        }
    }
    fn ensure_mline_styles(&mut self) {
        // gather existing mline style names
        let mut existing_mline_styles = HashSet::new();
        for obj in &self.objects {
            if let ObjectType::MLineStyle(ref ml) = &obj.specific {
                add_to_existing(&mut existing_mline_styles, &ml.style_name);
            }
        }

        // find mline style names that should exist
        let mut to_add = HashSet::new();
        for ent in &self.entities {
            if let EntityType::MLine(ref ml) = &ent.specific {
                add_to_existing(&mut to_add, &ml.style_name);
            }
        }

        // ensure all mline styles that should exist do
        for name in &to_add {
            if !existing_mline_styles.contains(name) {
                existing_mline_styles.insert(name.clone());
                self.objects
                    .push(Object::new(ObjectType::MLineStyle(MLineStyle {
                        style_name: name.clone(),
                        ..Default::default()
                    })));
            }
        }
    }
    fn ensure_dimension_styles(&mut self) {
        // gather existing dimension style names
        let mut existing_dim_styles = HashSet::new();
        for dim_style in &self.dim_styles {
            add_to_existing(&mut existing_dim_styles, &dim_style.name);
        }

        // find dimension style names that should exist
        let mut to_add = HashSet::new();
        add_to_existing(&mut to_add, &String::from("STANDARD"));
        add_to_existing(&mut to_add, &String::from("ANNOTATIVE"));
        for ent in &self.entities {
            match &ent.specific {
                EntityType::RotatedDimension(ref d) => {
                    add_to_existing(&mut to_add, &d.dimension_base.dimension_style_name)
                }
                EntityType::RadialDimension(ref d) => {
                    add_to_existing(&mut to_add, &d.dimension_base.dimension_style_name)
                }
                EntityType::DiameterDimension(ref d) => {
                    add_to_existing(&mut to_add, &d.dimension_base.dimension_style_name)
                }
                EntityType::AngularThreePointDimension(ref d) => {
                    add_to_existing(&mut to_add, &d.dimension_base.dimension_style_name)
                }
                EntityType::OrdinateDimension(ref d) => {
                    add_to_existing(&mut to_add, &d.dimension_base.dimension_style_name)
                }
                EntityType::Leader(ref l) => add_to_existing(&mut to_add, &l.dimension_style_name),
                EntityType::Tolerance(ref t) => {
                    add_to_existing(&mut to_add, &t.dimension_style_name)
                }
                _ => (),
            }
        }

        // ensure all dimension styles that should exist do
        for name in &to_add {
            if !existing_dim_styles.contains(name) {
                existing_dim_styles.insert(name.clone());
                self.dim_styles.push(DimStyle {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_layers(&mut self) {
        // gather existing layer names
        let mut existing_layers = HashSet::new();
        for layer in &self.layers {
            add_to_existing(&mut existing_layers, &layer.name);
        }

        // find layer names that should exist
        let mut to_add = HashSet::new();
        add_to_existing(&mut to_add, &String::from("0"));
        add_to_existing(&mut to_add, &self.header.current_layer);
        for block in &self.blocks {
            add_to_existing(&mut to_add, &block.layer);
            for ent in &block.entities {
                add_to_existing(&mut to_add, &ent.common.layer);
            }
        }
        for ent in &self.entities {
            add_to_existing(&mut to_add, &ent.common.layer);
        }
        for obj in &self.objects {
            match &obj.specific {
                ObjectType::LayerFilter(ref l) => {
                    for layer_name in &l.layer_names {
                        add_to_existing(&mut to_add, &layer_name);
                    }
                }
                ObjectType::LayerIndex(ref l) => {
                    for layer_name in &l.layer_names {
                        add_to_existing(&mut to_add, &layer_name);
                    }
                }
                _ => (),
            }
        }

        // ensure all layers that should exist do
        for name in &to_add {
            if !existing_layers.contains(name) {
                existing_layers.insert(name.clone());
                self.layers.push(Layer {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_line_types(&mut self) {
        // gather existing line type names
        let mut existing_line_types = HashSet::new();
        for line_type in &self.line_types {
            add_to_existing(&mut existing_line_types, &line_type.name);
        }

        // find line_types that should exist
        let mut to_add = HashSet::new();
        add_to_existing(&mut to_add, &String::from("BYLAYER"));
        add_to_existing(&mut to_add, &String::from("BYBLOCK"));
        add_to_existing(&mut to_add, &String::from("CONTINUOUS"));
        add_to_existing(&mut to_add, &self.header.current_entity_line_type);
        add_to_existing(&mut to_add, &self.header.dimension_line_type);
        for layer in &self.layers {
            add_to_existing(&mut to_add, &layer.line_type_name);
        }
        for block in &self.blocks {
            for ent in &block.entities {
                add_to_existing(&mut to_add, &ent.common.line_type_name);
            }
        }
        for ent in &self.entities {
            add_to_existing(&mut to_add, &ent.common.line_type_name);
        }
        for obj in &self.objects {
            if let ObjectType::MLineStyle(ref style) = &obj.specific {
                add_to_existing(&mut to_add, &style.style_name);
            }
        }

        // ensure all line_types that should exist do
        for name in &to_add {
            if !existing_line_types.contains(name) {
                existing_line_types.insert(name.clone());
                self.line_types.push(LineType {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_text_styles(&mut self) {
        // gather existing text style names
        let mut existing_styles = HashSet::new();
        for style in &self.styles {
            add_to_existing(&mut existing_styles, &style.name);
        }

        // find styles that should exist
        let mut to_add = HashSet::new();
        add_to_existing(&mut to_add, &String::from("STANDARD"));
        add_to_existing(&mut to_add, &String::from("ANNOTATIVE"));
        for entity in &self.entities {
            match &entity.specific {
                EntityType::ArcAlignedText(ref e) => {
                    add_to_existing(&mut to_add, &e.text_style_name)
                }
                EntityType::Attribute(ref e) => add_to_existing(&mut to_add, &e.text_style_name),
                EntityType::AttributeDefinition(ref e) => {
                    add_to_existing(&mut to_add, &e.text_style_name)
                }
                EntityType::MText(ref e) => add_to_existing(&mut to_add, &e.text_style_name),
                EntityType::Text(ref e) => add_to_existing(&mut to_add, &e.text_style_name),
                _ => (),
            }
        }
        for obj in &self.objects {
            if let ObjectType::MLineStyle(ref o) = &obj.specific {
                add_to_existing(&mut to_add, &o.style_name);
            }
        }

        // ensure all styles that should exist do
        for name in &to_add {
            if !existing_styles.contains(name) {
                existing_styles.insert(name.clone());
                self.styles.push(Style {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_view_ports(&mut self) {
        // gather existing view port names
        let mut existing_view_ports = HashSet::new();
        for vp in &self.view_ports {
            add_to_existing(&mut existing_view_ports, &vp.name);
        }

        // find view ports that should exist
        let mut to_add = HashSet::new();
        add_to_existing(&mut to_add, &String::from("*ACTIVE"));

        // ensure all view ports that should exist do
        for name in &to_add {
            if !existing_view_ports.contains(name) {
                existing_view_ports.insert(name.clone());
                self.view_ports.push(ViewPort {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_views(&mut self) {
        // gather existing view names
        let mut existing_views = HashSet::new();
        for view in &self.views {
            add_to_existing(&mut existing_views, &view.name);
        }

        // find views that should exist
        let mut to_add = HashSet::new();
        for obj in &self.objects {
            if let ObjectType::PlotSettings(ref ps) = &obj.specific {
                add_to_existing(&mut to_add, &ps.plot_view_name);
            }
        }

        // ensure all views that should exist do
        for name in &to_add {
            if !existing_views.contains(name) {
                existing_views.insert(name.clone());
                self.views.push(View {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_ucs(&mut self) {
        // gather existing ucs names
        let mut existing_ucs = HashSet::new();
        for ucs in &self.ucss {
            add_to_existing(&mut existing_ucs, &ucs.name);
        }

        // find ucs that should exist
        let mut to_add = HashSet::new();
        add_to_existing(&mut to_add, &self.header.ucs_definition_name);
        add_to_existing(&mut to_add, &self.header.ucs_name);
        add_to_existing(&mut to_add, &self.header.ortho_ucs_reference);
        add_to_existing(&mut to_add, &self.header.paperspace_ucs_definition_name);
        add_to_existing(&mut to_add, &self.header.paperspace_ucs_name);
        add_to_existing(&mut to_add, &self.header.paperspace_ortho_ucs_reference);

        // ensure all ucs that should exist do
        for name in &to_add {
            if !name.is_empty() && !existing_ucs.contains(name) {
                existing_ucs.insert(name.clone());
                self.ucss.push(Ucs {
                    name: name.clone(),
                    ..Default::default()
                });
            }
        }
    }
}

fn add_to_existing(set: &mut HashSet<String>, val: &str) {
    if !set.contains(val) {
        set.insert(val.to_string());
    }
}
