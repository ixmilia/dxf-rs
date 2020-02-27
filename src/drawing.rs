// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate byteorder;
use self::byteorder::{ByteOrder, LittleEndian, WriteBytesExt};

extern crate encoding_rs;
use self::encoding_rs::Encoding;

extern crate image;
use self::image::DynamicImage;

use crate::code_pair_put_back::CodePairPutBack;
use crate::drawing_item::{DrawingItem, DrawingItemMut};
use crate::entities::*;
use crate::enums::*;
use crate::header::*;
use crate::objects::*;
use crate::tables::*;

use crate::{CodePair, CodePairValue, DxfError, DxfResult};

use crate::dxb_reader::DxbReader;
use crate::dxb_writer::DxbWriter;
use crate::entity_iter::EntityIter;
use crate::handle_tracker::HandleTracker;
use crate::helper_functions::*;
use crate::object_iter::ObjectIter;

use crate::block::Block;
use crate::class::Class;

use crate::code_pair_iter::CodePairIter;
use crate::code_pair_writer::CodePairWriter;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

use itertools::put_back;
use std::collections::HashSet;
use std::iter::Iterator;
use std::path::Path;

pub(crate) const AUTO_REPLACE_HANDLE: u32 = 0xFFFF_FFFF;

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

    /// Internal collection of entities.
    __entities: Vec<Entity>,
    /// Internal collection of objects.
    __objects: Vec<Object>,

    /// The thumbnail image preview of the drawing.
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub thumbnail: Option<DynamicImage>,
}

// public implementation
impl Drawing {
    #[allow(clippy::new_without_default)] // default state of struct isn't valid
    pub fn new() -> Self {
        let mut drawing = Drawing {
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
            __entities: vec![],
            __objects: vec![],
            thumbnail: None,
        };
        drawing.normalize();
        drawing
    }
    /// Loads a `Drawing` from anything that implements the `Read` trait.
    pub fn load<T>(reader: &mut T) -> DxfResult<Drawing>
    where
        T: Read + ?Sized,
    {
        Drawing::load_with_encoding(reader, encoding_rs::WINDOWS_1252)
    }
    /// Loads a `Drawing` from anything that implements the `Read` trait using the specified text encoding.
    pub fn load_with_encoding<T>(reader: &mut T, encoding: &'static Encoding) -> DxfResult<Drawing>
    where
        T: Read + ?Sized,
    {
        let first_line = match read_line(reader, encoding) {
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
                let reader = CodePairIter::new(reader, encoding, first_line);
                let mut drawing = Drawing::new();
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
        Drawing::load_file_with_encoding(file_name, encoding_rs::WINDOWS_1252)
    }
    /// Loads a `Drawing` from disk, using a `BufReader` with the specified text encoding.
    pub fn load_file_with_encoding(
        file_name: &str,
        encoding: &'static Encoding,
    ) -> DxfResult<Drawing> {
        let path = Path::new(file_name);
        let file = File::open(&path)?;
        let mut buf_reader = BufReader::new(file);
        Drawing::load_with_encoding(&mut buf_reader, encoding)
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
            self.write_entities(write_handles, &mut code_pair_writer)?;
            self.write_objects(&mut code_pair_writer)?;
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
    /// Returns an iterator for all contained entities.
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.__entities.iter()
    }
    /// Returns an iterator for all mutable entities.
    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.__entities.iter_mut()
    }
    /// Adds an entity to the `Drawing`.
    pub fn add_entity(&mut self, mut entity: Entity) -> &Entity {
        entity.common.handle = self.next_handle();

        // set child handles
        match entity.specific {
            EntityType::Insert(ref mut ins) => {
                for a in ins.__attributes_and_handles.iter_mut() {
                    if a.1 == AUTO_REPLACE_HANDLE {
                        a.1 = self.next_handle();
                    }
                }
            }
            EntityType::Polyline(ref mut poly) => {
                for v in poly.__vertices_and_handles.iter_mut() {
                    if v.1 == AUTO_REPLACE_HANDLE {
                        v.1 = self.next_handle();
                    }
                }
            }
            _ => (),
        }

        // ensure invariants
        self.add_entity_no_handle_set(entity)
    }
    /// Returns an iterator for all contained objects.
    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        self.__objects.iter()
    }
    /// Returns an iterator for all mutable objects.
    pub fn objects_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.__objects.iter_mut()
    }
    /// Adds an object to the `Drawing`.
    pub fn add_object(&mut self, mut obj: Object) -> &Object {
        obj.common.handle = self.next_handle();

        // TODO: set child handles

        // ensure invariants
        self.add_object_no_handle_set(obj)
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
        self.__entities.clear();
        self.__objects.clear();
        self.thumbnail = None;

        self.header.next_available_handle = 1;
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
        self.ensure_dimension_styles();
        self.ensure_layers();
        self.ensure_line_types();
        self.ensure_text_styles();
        self.ensure_view_ports();
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
        for item in &self.__entities {
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
        for item in &self.__objects {
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
        for item in &mut self.__entities {
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
        for item in &mut self.__objects {
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
    pub(crate) fn next_handle(&mut self) -> u32 {
        let result = self.header.next_available_handle;
        self.header.next_available_handle += 1;
        result
    }
    fn add_entity_no_handle_set(&mut self, entity: Entity) -> &Entity {
        self.ensure_mline_style_is_present_for_entity(&entity);
        self.ensure_dimension_style_is_present_for_entity(&entity);
        self.ensure_layer_is_present(&entity.common.layer);
        self.ensure_line_type_is_present(&entity.common.line_type_name);
        self.ensure_text_style_is_present_for_entity(&entity);
        self.__entities.push(entity);
        self.__entities.last().unwrap()
    }
    fn add_object_no_handle_set(&mut self, obj: Object) -> &Object {
        self.ensure_layer_is_present_for_object(&obj);
        self.ensure_line_type_is_present_for_object(&obj);
        self.ensure_text_style_is_present_for_object(&obj);
        self.ensure_view_is_present(&obj);
        self.__objects.push(obj);
        self.__objects.last().unwrap()
    }
    fn ensure_mline_style_is_present_for_entity(&mut self, entity: &Entity) {
        if let EntityType::MLine(ref ml) = &entity.specific {
            if !self.objects().any(|o| match o.specific {
                ObjectType::MLineStyle(ref mline_style) => mline_style.style_name == ml.style_name,
                _ => false,
            }) {
                self.add_object(Object::new(ObjectType::MLineStyle(MLineStyle {
                    style_name: ml.style_name.clone(),
                    ..Default::default()
                })));
            }
        }
    }
    fn ensure_dimension_style_is_present_for_entity(&mut self, entity: &Entity) {
        // ensure corresponding dimension style is present
        let dim_style_name = match &entity.specific {
            EntityType::RotatedDimension(ref d) => Some(&d.dimension_base.dimension_style_name),
            EntityType::RadialDimension(ref d) => Some(&d.dimension_base.dimension_style_name),
            EntityType::DiameterDimension(ref d) => Some(&d.dimension_base.dimension_style_name),
            EntityType::AngularThreePointDimension(ref d) => {
                Some(&d.dimension_base.dimension_style_name)
            }
            EntityType::OrdinateDimension(ref d) => Some(&d.dimension_base.dimension_style_name),
            EntityType::Leader(ref l) => Some(&l.dimension_style_name),
            EntityType::Tolerance(ref t) => Some(&t.dimension_style_name),
            _ => None,
        };
        if let Some(dim_style_name) = dim_style_name {
            if !self.dim_styles.iter().any(|d| &d.name == dim_style_name) {
                self.dim_styles.push(DimStyle {
                    name: dim_style_name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_layer_is_present_for_object(&mut self, obj: &Object) {
        match &obj.specific {
            ObjectType::LayerFilter(ref l) => {
                for layer_name in &l.layer_names {
                    self.ensure_layer_is_present(&layer_name);
                }
            }
            ObjectType::LayerIndex(ref l) => {
                for layer_name in &l.layer_names {
                    self.ensure_layer_is_present(&layer_name);
                }
            }
            _ => (),
        }
    }
    fn ensure_layer_is_present(&mut self, layer_name: &str) {
        if !self.layers.iter().any(|l| l.name == *layer_name) {
            self.layers.push(Layer {
                name: String::from(layer_name),
                ..Default::default()
            });
        }
    }
    fn ensure_line_type_is_present_for_object(&mut self, obj: &Object) {
        if let ObjectType::MLineStyle(ref style) = &obj.specific {
            self.ensure_line_type_is_present(&style.style_name);
        }
    }
    fn ensure_line_type_is_present(&mut self, line_type_name: &str) {
        if !self.line_types.iter().any(|lt| lt.name == *line_type_name) {
            self.line_types.push(LineType {
                name: String::from(line_type_name),
                ..Default::default()
            });
        }
    }
    fn ensure_text_style_is_present_for_entity(&mut self, entity: &Entity) {
        let text_style_name = match &entity.specific {
            EntityType::ArcAlignedText(ref e) => Some(&e.text_style_name),
            EntityType::Attribute(ref e) => Some(&e.text_style_name),
            EntityType::AttributeDefinition(ref e) => Some(&e.text_style_name),
            EntityType::MText(ref e) => Some(&e.text_style_name),
            EntityType::Text(ref e) => Some(&e.text_style_name),
            _ => None,
        };
        if let Some(text_style_name) = text_style_name {
            self.ensure_text_style_is_present(&text_style_name);
        }
    }
    fn ensure_text_style_is_present_for_object(&mut self, obj: &Object) {
        if let ObjectType::MLineStyle(ref o) = &obj.specific {
            self.ensure_text_style_is_present(&o.style_name);
        }
    }
    fn ensure_text_style_is_present(&mut self, text_style_name: &str) {
        if !self.styles.iter().any(|s| s.name == text_style_name) {
            self.styles.push(Style {
                name: String::from(text_style_name),
                ..Default::default()
            });
        }
    }
    fn ensure_view_is_present(&mut self, obj: &Object) {
        if let ObjectType::PlotSettings(ref ps) = &obj.specific {
            if !self.views.iter().any(|v| v.name == ps.plot_view_name) {
                self.views.push(View {
                    name: ps.plot_view_name.clone(),
                    ..Default::default()
                });
            }
        }
    }
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
    ) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "ENTITIES"))?;
        for e in &self.__entities {
            e.write(self.header.version, write_handles, writer)?;
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_objects<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "OBJECTS"))?;
        for o in &self.__objects {
            o.write(self.header.version, writer)?;
        }

        writer.write_code_pair(&CodePair::new_str(0, "ENDSEC"))?;
        Ok(())
    }
    fn write_thumbnail<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write,
    {
        if self.header.version >= AcadVersion::R2000 {
            if let Some(ref img) = self.thumbnail {
                writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
                writer.write_code_pair(&CodePair::new_str(2, "THUMBNAILIMAGE"))?;
                let mut data = vec![];
                img.write_to(&mut data, image::ImageFormat::BMP)?;
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
        let mut entities = vec![];
        iter.read_entities_into_vec(&mut entities)?;
        for e in entities {
            if e.common.handle == 0 {
                self.add_entity(e);
            } else {
                self.add_entity_no_handle_set(e);
            }
        }
        Ok(())
    }
    fn read_objects<T>(&mut self, iter: &mut CodePairPutBack<T>) -> DxfResult<()>
    where
        T: Read,
    {
        let iter = put_back(ObjectIter { iter });
        for o in iter {
            if o.common.handle == 0 {
                self.add_object(o);
            } else {
                self.add_object_no_handle_set(o);
            }
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
        for e in self.__entities.iter_mut() {
            e.normalize();
        }
    }
    fn normalize_objects(&mut self) {
        for o in self.__objects.iter_mut() {
            o.normalize();
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
        for ent in &self.__entities {
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

#[cfg(test)]
mod tests {
    use crate::entities::*;
    use crate::helper_functions::tests::*;
    use crate::objects::*;
    use crate::tables::*;
    use crate::*;

    #[test]
    fn default_layers_are_present() {
        let drawing = Drawing::new();
        assert_eq!(1, drawing.layers.len());
        assert_eq!("0", drawing.layers[0].name);
    }

    #[test]
    fn default_dim_styles_are_present() {
        let drawing = Drawing::new();
        assert_eq!(2, drawing.dim_styles.len());
        assert_eq!("ANNOTATIVE", drawing.dim_styles[0].name);
        assert_eq!("STANDARD", drawing.dim_styles[1].name);
    }

    #[test]
    fn default_line_types_are_present() {
        let drawing = Drawing::new();
        assert_eq!(3, drawing.line_types.len());
        assert_eq!("BYBLOCK", drawing.line_types[0].name);
        assert_eq!("BYLAYER", drawing.line_types[1].name);
        assert_eq!("CONTINUOUS", drawing.line_types[2].name);
    }

    #[test]
    fn default_text_styles_are_present() {
        let drawing = Drawing::new();
        assert_eq!(2, drawing.styles.len());
        assert_eq!("ANNOTATIVE", drawing.styles[0].name);
        assert_eq!("STANDARD", drawing.styles[1].name);
    }

    #[test]
    fn entity_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        let ent = Entity {
            common: Default::default(),
            specific: EntityType::Line(Default::default()),
        };
        assert_eq!(0, ent.common.handle);

        let ent = drawing.add_entity(ent);
        assert_ne!(0, ent.common.handle);
    }

    #[test]
    fn object_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        let obj = Object {
            common: Default::default(),
            specific: ObjectType::PlaceHolder(Default::default()),
        };
        assert_eq!(0, obj.common.handle);

        let obj = drawing.add_object(obj);
        assert_ne!(0, obj.common.handle);
    }

    #[test]
    fn entity_handle_is_set_during_read_if_not_specified() {
        let drawing = parse_drawing(
            vec![
                "  0", "SECTION", "  2", "ENTITIES", "  0", "LINE", "  0", "ENDSEC", "  0", "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let line = drawing.entities().nth(0).unwrap();
        assert_ne!(0, line.common.handle);
    }

    #[test]
    fn object_handle_is_set_during_read_if_not_specified() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "OBJECTS",
                "  0",
                "ACDBPLACEHOLDER",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let obj = drawing.objects().nth(0).unwrap();
        assert_ne!(0, obj.common.handle);
    }

    #[test]
    fn entity_handle_is_honored_during_read_if_specified() {
        let drawing = parse_drawing(
            vec![
                "  0", "SECTION", "  2", "ENTITIES", "  0", "LINE", "  5", "3333", "  0", "ENDSEC",
                "  0", "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let line = drawing.entities().nth(0).unwrap();
        assert_eq!(0x3333, line.common.handle);
    }

    #[test]
    fn object_handle_is_honored_during_read_if_specified() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "OBJECTS",
                "  0",
                "ACDBPLACEHOLDER",
                "  5",
                "3333",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let obj = drawing.objects().nth(0).unwrap();
        assert_eq!(0x3333, obj.common.handle);
    }

    #[test]
    fn next_available_handle_is_reset_on_clear() {
        let mut drawing = Drawing::new();
        drawing.add_entity(Entity {
            common: EntityCommon::default(),
            specific: EntityType::Line(Line::default()),
        });
        assert_eq!(1, drawing.entities().count());
        assert_ne!(0, drawing.header.next_available_handle);
        assert_ne!(1, drawing.header.next_available_handle);

        drawing.clear();
        assert_eq!(0, drawing.entities().count());
        assert_eq!(1, drawing.header.next_available_handle);
    }

    #[test]
    fn mline_style_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let mline_styles = drawing
            .objects()
            .filter(|&o| match o.specific {
                ObjectType::MLineStyle(ref mline_style) => {
                    mline_style.style_name == "some-mline-style"
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(0, mline_styles.len());

        drawing.add_entity(Entity {
            common: EntityCommon::default(),
            specific: EntityType::MLine(MLine {
                style_name: String::from("some-mline-style"),
                ..Default::default()
            }),
        });
        let mline_styles = drawing
            .objects()
            .filter(|&o| match o.specific {
                ObjectType::MLineStyle(ref mline_style) => {
                    mline_style.style_name == "some-mline-style"
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(1, mline_styles.len());
    }

    #[test]
    fn mline_style_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_object(Object {
            common: ObjectCommon::default(),
            specific: ObjectType::MLineStyle(MLineStyle {
                style_name: String::from("some-mline-style"),
                ..Default::default()
            }),
        });
        let mline_styles = drawing
            .objects()
            .filter(|&o| match o.specific {
                ObjectType::MLineStyle(ref mline_style) => {
                    mline_style.style_name == "some-mline-style"
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(1, mline_styles.len());

        drawing.add_entity(Entity {
            common: EntityCommon::default(),
            specific: EntityType::MLine(MLine {
                style_name: String::from("some-mline-style"),
                ..Default::default()
            }),
        });
        let mline_styles = drawing
            .objects()
            .filter(|&o| match o.specific {
                ObjectType::MLineStyle(ref mline_style) => {
                    mline_style.style_name == "some-mline-style"
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(1, mline_styles.len());
    }

    #[test]
    fn mline_style_is_added_with_entity_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "ENTITIES",
                "  0",
                "MLINE",
                "  2",
                "some-mline-style",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let mline_styles = drawing
            .objects()
            .filter(|&o| match o.specific {
                ObjectType::MLineStyle(ref mline_style) => {
                    mline_style.style_name == "some-mline-style"
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(1, mline_styles.len());
    }

    #[test]
    fn dim_style_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let dim_styles = drawing
            .dim_styles
            .iter()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(0, dim_styles.len());

        drawing.add_entity(Entity {
            common: EntityCommon::default(),
            specific: EntityType::RadialDimension(RadialDimension {
                dimension_base: DimensionBase {
                    dimension_style_name: String::from("some-dim-style"),
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        let dim_styles = drawing
            .dim_styles
            .iter()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(1, dim_styles.len());
    }

    #[test]
    fn dim_style_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.dim_styles.push(DimStyle {
            name: String::from("some-dim-style"),
            ..Default::default()
        });
        let dim_styles = drawing
            .dim_styles
            .iter()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(1, dim_styles.len());

        drawing.add_entity(Entity {
            common: EntityCommon::default(),
            specific: EntityType::RadialDimension(RadialDimension {
                dimension_base: DimensionBase {
                    dimension_style_name: String::from("some-dim-style"),
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        let dim_styles = drawing
            .dim_styles
            .iter()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(1, dim_styles.len());
    }

    #[test]
    fn dim_style_is_added_with_entity_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "ENTITIES",
                "  0",
                "DIMENSION",
                "  3",
                "some-dim-style",
                "100",
                "AcDbRadialDimension",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let dim_styles = drawing
            .dim_styles
            .iter()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(1, dim_styles.len());
    }

    #[test]
    fn layer_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(0, layers.len());

        drawing.add_entity(Entity {
            common: EntityCommon {
                layer: String::from("some-layer"),
                ..Default::default()
            },
            specific: EntityType::Line(Default::default()),
        });
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(0, layers.len());

        drawing.add_object(Object {
            common: ObjectCommon::default(),
            specific: ObjectType::LayerFilter(LayerFilter {
                layer_names: vec![String::from("some-layer")],
            }),
        });
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.layers.push(Layer {
            name: String::from("some-layer"),
            ..Default::default()
        });
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());

        drawing.add_entity(Entity {
            common: EntityCommon {
                layer: String::from("some-layer"),
                ..Default::default()
            },
            specific: EntityType::Line(Default::default()),
        });
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.layers.push(Layer {
            name: String::from("some-layer"),
            ..Default::default()
        });
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());

        drawing.add_object(Object {
            common: ObjectCommon::default(),
            specific: ObjectType::LayerFilter(LayerFilter {
                layer_names: vec![String::from("some-layer")],
            }),
        });
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_added_with_entity_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "ENTITIES",
                "  0",
                "LINE",
                "  8",
                "some-layer",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_added_with_object_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "OBJECTS",
                "  0",
                "LAYER_FILTER",
                "  8",
                "some-layer",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let layers = drawing
            .layers
            .iter()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn line_type_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(0, line_types.len());

        drawing.add_entity(Entity {
            common: EntityCommon {
                line_type_name: String::from("some-line-type"),
                ..Default::default()
            },
            specific: EntityType::Line(Default::default()),
        });
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(0, line_types.len());

        drawing.add_object(Object {
            common: ObjectCommon::default(),
            specific: ObjectType::MLineStyle(MLineStyle {
                style_name: String::from("some-line-type"),
                ..Default::default()
            }),
        });
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.line_types.push(LineType {
            name: String::from("some-line-type"),
            ..Default::default()
        });
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());

        drawing.add_entity(Entity {
            common: EntityCommon {
                line_type_name: String::from("some-line-type"),
                ..Default::default()
            },
            specific: EntityType::Line(Default::default()),
        });
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.line_types.push(LineType {
            name: String::from("some-line-type"),
            ..Default::default()
        });
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());

        drawing.add_object(Object {
            common: ObjectCommon::default(),
            specific: ObjectType::MLineStyle(MLineStyle {
                style_name: String::from("some-line-type"),
                ..Default::default()
            }),
        });
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_added_with_entity_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "ENTITIES",
                "  0",
                "LINE",
                "  6",
                "some-line-type",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_added_with_object_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "OBJECTS",
                "  0",
                "MLINESTYLE",
                "  2",
                "some-line-type",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let line_types = drawing
            .line_types
            .iter()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn text_style_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(0, text_styles.len());

        drawing.add_entity(Entity {
            common: Default::default(),
            specific: EntityType::Text(Text {
                text_style_name: String::from("some-text-style"),
                ..Default::default()
            }),
        });
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(0, text_styles.len());

        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::MLineStyle(MLineStyle {
                style_name: String::from("some-text-style"),
                ..Default::default()
            }),
        });
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.styles.push(Style {
            name: String::from("some-text-style"),
            ..Default::default()
        });
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());

        drawing.add_entity(Entity {
            common: Default::default(),
            specific: EntityType::Text(Text {
                text_style_name: String::from("some-text-style"),
                ..Default::default()
            }),
        });
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.styles.push(Style {
            name: String::from("some-text-style"),
            ..Default::default()
        });
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());

        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::MLineStyle(MLineStyle {
                style_name: String::from("some-text-style"),
                ..Default::default()
            }),
        });
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_added_with_entity_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "ENTITIES",
                "  0",
                "TEXT",
                "  7",
                "some-text-style",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_added_with_object_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "OBJECTS",
                "  0",
                "MLINESTYLE",
                "  2",
                "some-text-style",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let text_styles = drawing
            .styles
            .iter()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn view_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let views = drawing
            .views
            .iter()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(0, views.len());

        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::PlotSettings(PlotSettings {
                plot_view_name: String::from("some-view"),
                ..Default::default()
            }),
        });
        let views = drawing
            .views
            .iter()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(1, views.len());
    }

    #[test]
    fn view_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.views.push(View {
            name: String::from("some-view"),
            ..Default::default()
        });
        let views = drawing
            .views
            .iter()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(1, views.len());

        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::PlotSettings(PlotSettings {
                plot_view_name: String::from("some-view"),
                ..Default::default()
            }),
        });
        let views = drawing
            .views
            .iter()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(1, views.len());
    }

    #[test]
    fn view_is_added_with_object_on_file_read() {
        let drawing = parse_drawing(
            vec![
                "  0",
                "SECTION",
                "  2",
                "OBJECTS",
                "  0",
                "PLOTSETTINGS",
                "  6",
                "some-view",
                "  0",
                "ENDSEC",
                "  0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let views = drawing
            .views
            .iter()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(1, views.len());
    }
}
