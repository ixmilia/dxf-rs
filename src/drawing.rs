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

    //------------------------------------------------------------------- tables
    /// Internal collection of app ids.
    __app_ids: Vec<AppId>,
    /// Internal collection of block records.
    __block_records: Vec<BlockRecord>,
    /// Internal collection of dimension styles.
    __dim_styles: Vec<DimStyle>,
    /// Internal collection of layers.
    __layers: Vec<Layer>,
    /// Internal collection of line types.
    __line_types: Vec<LineType>,
    /// Internal collection of visual styles.
    __styles: Vec<Style>,
    /// Internal collection of user coordinate systems (UCS).
    __ucss: Vec<Ucs>,
    /// Internal collection of views.
    __views: Vec<View>,
    /// Internal collection of view ports.
    __view_ports: Vec<ViewPort>,

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
            __app_ids: vec![],
            __block_records: vec![],
            __dim_styles: vec![],
            __layers: vec![],
            __line_types: vec![],
            __styles: vec![],
            __ucss: vec![],
            __views: vec![],
            __view_ports: vec![],
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
            self.write_tables(write_handles, &mut code_pair_writer)?;
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
    /// Returns an iterator for all app ids.
    pub fn app_ids(&self) -> impl Iterator<Item = &AppId> {
        self.__app_ids.iter()
    }
    /// Returns an iterator for all mutable app ids.
    pub fn app_ids_mut(&mut self) -> impl Iterator<Item = &mut AppId> {
        self.__app_ids.iter_mut()
    }
    /// Adds an app id to the `Drawing`.
    pub fn add_app_id(&mut self, mut app_id: AppId) -> &AppId {
        app_id.handle = self.next_handle();
        self.add_app_id_no_handle_set(app_id)
    }
    /// Returns an iterator for all block records.
    pub fn block_records(&self) -> impl Iterator<Item = &BlockRecord> {
        self.__block_records.iter()
    }
    /// Returns an iterator for all mutable block records.
    pub fn block_records_mut(&mut self) -> impl Iterator<Item = &mut BlockRecord> {
        self.__block_records.iter_mut()
    }
    /// Adds a block record to the `Drawing`.
    pub fn add_block_record(&mut self, mut block_record: BlockRecord) -> &BlockRecord {
        block_record.handle = self.next_handle();
        self.add_block_record_no_handle_set(block_record)
    }
    /// Returns an iterator for all dimension styles.
    pub fn dim_styles(&self) -> impl Iterator<Item = &DimStyle> {
        self.__dim_styles.iter()
    }
    /// Returns an iterator for all mutable dimension styles.
    pub fn dim_styles_mut(&mut self) -> impl Iterator<Item = &mut DimStyle> {
        self.__dim_styles.iter_mut()
    }
    /// Adds a dimension style to the `Drawing`.
    pub fn add_dim_style(&mut self, mut dim_style: DimStyle) -> &DimStyle {
        dim_style.handle = self.next_handle();
        self.add_dim_style_no_handle_set(dim_style)
    }
    /// Returns an iterator for all layers.
    pub fn layers(&self) -> impl Iterator<Item = &Layer> {
        self.__layers.iter()
    }
    /// Returns an iterator for all mutable layers.
    pub fn layers_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        self.__layers.iter_mut()
    }
    /// Adds a layer to the `Drawing`.
    pub fn add_layer(&mut self, mut layer: Layer) -> &Layer {
        layer.handle = self.next_handle();
        self.add_layer_no_handle_set(layer)
    }
    /// Returns an iterator for all line types.
    pub fn line_types(&self) -> impl Iterator<Item = &LineType> {
        self.__line_types.iter()
    }
    /// Returns an iterator for all mutable line types.
    pub fn line_types_mut(&mut self) -> impl Iterator<Item = &mut LineType> {
        self.__line_types.iter_mut()
    }
    /// Adds a line type to the `Drawing`.
    pub fn add_line_type(&mut self, mut line_type: LineType) -> &LineType {
        line_type.handle = self.next_handle();
        self.add_line_type_no_handle_set(line_type)
    }
    /// Returns an iterator for all styles.
    pub fn styles(&self) -> impl Iterator<Item = &Style> {
        self.__styles.iter()
    }
    /// Returns an iterator for all mutable styles.
    pub fn styles_mut(&mut self) -> impl Iterator<Item = &mut Style> {
        self.__styles.iter_mut()
    }
    /// Adds a style to the `Drawing`.
    pub fn add_style(&mut self, mut style: Style) -> &Style {
        style.handle = self.next_handle();
        self.add_style_no_handle_set(style)
    }
    /// Returns an iterator for all ucss.
    pub fn ucss(&self) -> impl Iterator<Item = &Ucs> {
        self.__ucss.iter()
    }
    /// Returns an iterator for all mutable ucss.
    pub fn ucss_mut(&mut self) -> impl Iterator<Item = &mut Ucs> {
        self.__ucss.iter_mut()
    }
    /// Add a ucs to the `Drawing`.
    pub fn add_ucs(&mut self, mut ucs: Ucs) -> &Ucs {
        ucs.handle = self.next_handle();
        self.add_ucs_no_handle_set(ucs)
    }
    /// Returns an iterator for all views.
    pub fn views(&self) -> impl Iterator<Item = &View> {
        self.__views.iter()
    }
    /// Returns an iterator for all mutable views.
    pub fn views_mut(&mut self) -> impl Iterator<Item = &mut View> {
        self.__views.iter_mut()
    }
    /// Add a view to the `Drawing`.
    pub fn add_view(&mut self, mut view: View) -> &View {
        view.handle = self.next_handle();
        self.add_view_no_handle_set(view)
    }
    /// Returns an iterator for all view ports.
    pub fn view_ports(&self) -> impl Iterator<Item = &ViewPort> {
        self.__view_ports.iter()
    }
    /// Returns an iterator for all mutable view ports.
    pub fn view_ports_mut(&mut self) -> impl Iterator<Item = &mut ViewPort> {
        self.__view_ports.iter_mut()
    }
    /// Add a view port to the `Drawing`.
    pub fn add_view_port(&mut self, mut view_port: ViewPort) -> &ViewPort {
        view_port.handle = self.next_handle();
        self.add_view_port_no_handle_set(view_port)
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
        self.__app_ids.clear();
        self.__block_records.clear();
        self.__dim_styles.clear();
        self.__layers.clear();
        self.__line_types.clear();
        self.__styles.clear();
        self.__ucss.clear();
        self.__views.clear();
        self.__view_ports.clear();
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

        self.__app_ids.sort_by(|a, b| a.name.cmp(&b.name));
        self.__block_records.sort_by(|a, b| a.name.cmp(&b.name));
        self.__dim_styles.sort_by(|a, b| a.name.cmp(&b.name));
        self.__layers.sort_by(|a, b| a.name.cmp(&b.name));
        self.__line_types.sort_by(|a, b| a.name.cmp(&b.name));
        self.__styles.sort_by(|a, b| a.name.cmp(&b.name));
        self.__ucss.sort_by(|a, b| a.name.cmp(&b.name));
        self.__views.sort_by(|a, b| a.name.cmp(&b.name));
        self.__view_ports.sort_by(|a, b| a.name.cmp(&b.name));
    }
    /// Gets a `DrawingItem` with the appropriate handle or `None`.
    pub fn get_item_by_handle(&'_ self, handle: u32) -> Option<DrawingItem<'_>> {
        for item in &self.__app_ids {
            if item.handle == handle {
                return Some(DrawingItem::AppId(item));
            }
        }
        for item in &self.blocks {
            if item.handle == handle {
                return Some(DrawingItem::Block(item));
            }
        }
        for item in &self.__block_records {
            if item.handle == handle {
                return Some(DrawingItem::BlockRecord(item));
            }
        }
        for item in &self.__dim_styles {
            if item.handle == handle {
                return Some(DrawingItem::DimStyle(item));
            }
        }
        for item in &self.__entities {
            if item.common.handle == handle {
                return Some(DrawingItem::Entity(item));
            }
        }
        for item in &self.__layers {
            if item.handle == handle {
                return Some(DrawingItem::Layer(item));
            }
        }
        for item in &self.__line_types {
            if item.handle == handle {
                return Some(DrawingItem::LineType(item));
            }
        }
        for item in &self.__objects {
            if item.common.handle == handle {
                return Some(DrawingItem::Object(item));
            }
        }
        for item in &self.__styles {
            if item.handle == handle {
                return Some(DrawingItem::Style(item));
            }
        }
        for item in &self.__ucss {
            if item.handle == handle {
                return Some(DrawingItem::Ucs(item));
            }
        }
        for item in &self.__views {
            if item.handle == handle {
                return Some(DrawingItem::View(item));
            }
        }
        for item in &self.__view_ports {
            if item.handle == handle {
                return Some(DrawingItem::ViewPort(item));
            }
        }

        None
    }
    /// Gets a `DrawingItemMut` with the appropriate handle or `None`.
    pub fn get_item_by_handle_mut(&'_ mut self, handle: u32) -> Option<DrawingItemMut<'_>> {
        for item in &mut self.__app_ids {
            if item.handle == handle {
                return Some(DrawingItemMut::AppId(item));
            }
        }
        for item in &mut self.blocks {
            if item.handle == handle {
                return Some(DrawingItemMut::Block(item));
            }
        }
        for item in &mut self.__block_records {
            if item.handle == handle {
                return Some(DrawingItemMut::BlockRecord(item));
            }
        }
        for item in &mut self.__dim_styles {
            if item.handle == handle {
                return Some(DrawingItemMut::DimStyle(item));
            }
        }
        for item in &mut self.__entities {
            if item.common.handle == handle {
                return Some(DrawingItemMut::Entity(item));
            }
        }
        for item in &mut self.__layers {
            if item.handle == handle {
                return Some(DrawingItemMut::Layer(item));
            }
        }
        for item in &mut self.__line_types {
            if item.handle == handle {
                return Some(DrawingItemMut::LineType(item));
            }
        }
        for item in &mut self.__objects {
            if item.common.handle == handle {
                return Some(DrawingItemMut::Object(item));
            }
        }
        for item in &mut self.__styles {
            if item.handle == handle {
                return Some(DrawingItemMut::Style(item));
            }
        }
        for item in &mut self.__ucss {
            if item.handle == handle {
                return Some(DrawingItemMut::Ucs(item));
            }
        }
        for item in &mut self.__views {
            if item.handle == handle {
                return Some(DrawingItemMut::View(item));
            }
        }
        for item in &mut self.__view_ports {
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
    pub(crate) fn add_app_id_no_handle_set(&mut self, app_id: AppId) -> &AppId {
        // TODO: ensure invariants
        self.__app_ids.push(app_id);
        self.__app_ids.last().unwrap()
    }
    pub(crate) fn add_block_record_no_handle_set(
        &mut self,
        block_record: BlockRecord,
    ) -> &BlockRecord {
        // TODO: ensure invariants
        self.__block_records.push(block_record);
        self.__block_records.last().unwrap()
    }
    pub(crate) fn add_dim_style_no_handle_set(&mut self, dim_style: DimStyle) -> &DimStyle {
        // TODO: ensure invariants
        self.__dim_styles.push(dim_style);
        self.__dim_styles.last().unwrap()
    }
    pub(crate) fn add_layer_no_handle_set(&mut self, layer: Layer) -> &Layer {
        // TODO: ensure invariants
        self.__layers.push(layer);
        self.__layers.last().unwrap()
    }
    pub(crate) fn add_line_type_no_handle_set(&mut self, line_type: LineType) -> &LineType {
        // TODO: ensure invariants
        self.__line_types.push(line_type);
        self.__line_types.last().unwrap()
    }
    pub(crate) fn add_style_no_handle_set(&mut self, style: Style) -> &Style {
        // TODO: ensure invariants
        self.__styles.push(style);
        self.__styles.last().unwrap()
    }
    pub(crate) fn add_ucs_no_handle_set(&mut self, ucs: Ucs) -> &Ucs {
        // TODO: ensure invariants
        self.__ucss.push(ucs);
        self.__ucss.last().unwrap()
    }
    pub(crate) fn add_view_no_handle_set(&mut self, view: View) -> &View {
        // TODO: ensure invariants
        self.__views.push(view);
        self.__views.last().unwrap()
    }
    pub(crate) fn add_view_port_no_handle_set(&mut self, view_port: ViewPort) -> &ViewPort {
        // TODO: ensure invariants
        self.__view_ports.push(view_port);
        self.__view_ports.last().unwrap()
    }
    fn ensure_app_id_is_present(&mut self, name: &str) {
        if !self.app_ids().any(|a| a.name == *name) {
            self.add_app_id(AppId {
                name: String::from(name),
                ..Default::default()
            });
        }
    }
    fn ensure_block_record_is_present(&mut self, name: &str) {
        if !self.block_records().any(|b| b.name == name) {
            self.add_block_record(BlockRecord {
                name: String::from(name),
                ..Default::default()
            });
        }
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
            self.ensure_dimension_style_is_present(&dim_style_name);
        }
    }
    fn ensure_dimension_style_is_present(&mut self, dim_style_name: &str) {
        if !self.dim_styles().any(|d| d.name == dim_style_name) {
            self.add_dim_style(DimStyle {
                name: String::from(dim_style_name),
                ..Default::default()
            });
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
        if !self.layers().any(|l| l.name == *layer_name) {
            self.add_layer(Layer {
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
        if !self.line_types().any(|lt| lt.name == *line_type_name) {
            self.add_line_type(LineType {
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
        if !self.styles().any(|s| s.name == text_style_name) {
            self.add_style(Style {
                name: String::from(text_style_name),
                ..Default::default()
            });
        }
    }
    fn ensure_ucs_is_present(&mut self, ucs_name: &str) {
        if !self.ucss().any(|u| u.name == ucs_name) {
            self.add_ucs(Ucs {
                name: String::from(ucs_name),
                ..Default::default()
            });
        }
    }
    fn ensure_view_is_present(&mut self, obj: &Object) {
        if let ObjectType::PlotSettings(ref ps) = &obj.specific {
            if !self.views().any(|v| v.name == ps.plot_view_name) {
                self.add_view(View {
                    name: ps.plot_view_name.clone(),
                    ..Default::default()
                });
            }
        }
    }
    fn ensure_view_port_is_present(&mut self, name: &str) {
        if !self.view_ports().any(|v| v.name == name) {
            self.add_view_port(ViewPort {
                name: String::from(name),
                ..Default::default()
            });
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
    fn write_tables<T>(&self, write_handles: bool, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write,
    {
        writer.write_code_pair(&CodePair::new_str(0, "SECTION"))?;
        writer.write_code_pair(&CodePair::new_str(2, "TABLES"))?;
        write_tables(&self, write_handles, writer)?;
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
        // ensure all app ids that should exist do
        self.ensure_app_id_is_present("ACAD");
        self.ensure_app_id_is_present("ACADANNOTATIVE");
        self.ensure_app_id_is_present("ACAD_NAV_VCDISPLAY");
        self.ensure_app_id_is_present("ACAD_MLEADERVER");
    }
    fn normalize_block_records(&mut self) {
        // ensure all block records that should exist do
        self.ensure_block_record_is_present("*MODEL_SPACE");
        self.ensure_block_record_is_present("*PAPER_SPACE");
    }
    fn normalize_layers(&mut self) {
        self.ensure_layer_is_present(&self.header.current_layer.clone());
        for l in self.layers_mut() {
            l.normalize();
        }
    }
    fn normalize_text_styles(&mut self) {
        for s in self.styles_mut() {
            s.normalize();
        }
    }
    fn normalize_view_ports(&mut self) {
        for v in self.view_ports_mut() {
            v.normalize();
        }
    }
    fn normalize_views(&mut self) {
        for v in self.views_mut() {
            v.normalize();
        }
    }
    fn ensure_dimension_styles(&mut self) {
        // ensure all dimension styles that should exist do
        self.ensure_dimension_style_is_present("STANDARD");
        self.ensure_dimension_style_is_present("ANNOTATIVE");
    }
    fn ensure_layers(&mut self) {
        // ensure all layers that should exist do
        let mut should_exist = HashSet::new();
        should_exist.insert(String::from("0"));
        for block in &self.blocks {
            should_exist.insert(block.layer.clone());
            for ent in &block.entities {
                should_exist.insert(ent.common.layer.clone());
            }
        }

        for name in &should_exist {
            self.ensure_layer_is_present(name);
        }
    }
    fn ensure_line_types(&mut self) {
        // ensure all line_types that should exist do
        let mut should_exist = HashSet::new();
        should_exist.insert(String::from("BYLAYER"));
        should_exist.insert(String::from("BYBLOCK"));
        should_exist.insert(String::from("CONTINUOUS"));
        for layer in self.layers() {
            should_exist.insert(layer.line_type_name.clone());
        }
        for block in &self.blocks {
            for ent in &block.entities {
                should_exist.insert(ent.common.line_type_name.clone());
            }
        }

        for name in &should_exist {
            self.ensure_line_type_is_present(name);
        }
    }
    fn ensure_text_styles(&mut self) {
        // ensure all styles that should exist do
        self.ensure_text_style_is_present("STANDARD");
        self.ensure_text_style_is_present("ANNOTATIVE");
    }
    fn ensure_view_ports(&mut self) {
        // ensure all view ports that should exist do
        self.ensure_view_port_is_present("*ACTIVE");
    }
    fn ensure_ucs(&mut self) {
        // ensure all ucs that should exist do
        let mut should_exist = HashSet::new();
        should_exist.insert(self.header.ucs_definition_name.clone());
        should_exist.insert(self.header.ucs_name.clone());
        should_exist.insert(self.header.ortho_ucs_reference.clone());
        should_exist.insert(self.header.paperspace_ucs_definition_name.clone());
        should_exist.insert(self.header.paperspace_ucs_name.clone());
        should_exist.insert(self.header.paperspace_ortho_ucs_reference.clone());

        for name in &should_exist {
            if !name.is_empty() {
                self.ensure_ucs_is_present(name);
            }
        }
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
        let layers = drawing.layers().collect::<Vec<_>>();
        assert_eq!(1, layers.len());
        assert_eq!("0", layers[0].name);
    }

    #[test]
    fn default_dim_styles_are_present() {
        let drawing = Drawing::new();
        let dim_styles = drawing.dim_styles().collect::<Vec<_>>();
        assert_eq!(2, dim_styles.len());
        assert_eq!("ANNOTATIVE", dim_styles[0].name);
        assert_eq!("STANDARD", dim_styles[1].name);
    }

    #[test]
    fn default_line_types_are_present() {
        let drawing = Drawing::new();
        let line_types = drawing.line_types().collect::<Vec<_>>();
        assert_eq!(3, line_types.len());
        assert_eq!("BYBLOCK", line_types[0].name);
        assert_eq!("BYLAYER", line_types[1].name);
        assert_eq!("CONTINUOUS", line_types[2].name);
    }

    #[test]
    fn default_text_styles_are_present() {
        let drawing = Drawing::new();
        let styles = drawing.styles().collect::<Vec<_>>();
        assert_eq!(2, styles.len());
        assert_eq!("ANNOTATIVE", styles[0].name);
        assert_eq!("STANDARD", styles[1].name);
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
    fn layer_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        drawing.clear();
        let layer = Layer::default();
        assert_eq!(0, layer.handle);

        let layer = drawing.add_layer(layer);
        assert_ne!(0, layer.handle);
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
    fn layer_handle_is_set_during_read_if_not_specified() {
        let drawing = parse_drawing(
            vec![
                "  0", "SECTION", "  2", "TABLES", "  0", "TABLE", "  2", "LAYER", "  0", "LAYER",
                "  0", "ENDTAB", "  0", "ENDSEC", "  0", "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let layer = drawing.layers().nth(0).unwrap();
        assert_ne!(0, layer.handle);
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
    fn layer_handle_is_honored_during_read_if_specified() {
        let drawing = parse_drawing(
            vec![
                "  0", "SECTION", "  2", "TABLES", "  0", "TABLE", "  2", "LAYER", "  0", "LAYER",
                "  5", "3333", "  0", "ENDTAB", "  0", "ENDSEC", "  0", "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        let layer = drawing.layers().nth(0).unwrap();
        assert_eq!(0x3333, layer.handle);
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
            .dim_styles()
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
            .dim_styles()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(1, dim_styles.len());
    }

    #[test]
    fn dim_style_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_dim_style(DimStyle {
            name: String::from("some-dim-style"),
            ..Default::default()
        });
        let dim_styles = drawing
            .dim_styles()
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
            .dim_styles()
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
            .dim_styles()
            .filter(|&d| d.name == "some-dim-style")
            .collect::<Vec<_>>();
        assert_eq!(1, dim_styles.len());
    }

    #[test]
    fn layer_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let layers = drawing
            .layers()
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
            .layers()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let layers = drawing
            .layers()
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
            .layers()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_layer(Layer {
            name: String::from("some-layer"),
            ..Default::default()
        });
        let layers = drawing
            .layers()
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
            .layers()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn layer_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_layer(Layer {
            name: String::from("some-layer"),
            ..Default::default()
        });
        let layers = drawing
            .layers()
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
            .layers()
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
            .layers()
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
            .layers()
            .filter(|&l| l.name == "some-layer")
            .collect::<Vec<_>>();
        assert_eq!(1, layers.len());
    }

    #[test]
    fn line_type_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let line_types = drawing
            .line_types()
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
            .line_types()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let line_types = drawing
            .line_types()
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
            .line_types()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_line_type(LineType {
            name: String::from("some-line-type"),
            ..Default::default()
        });
        let line_types = drawing
            .line_types()
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
            .line_types()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn line_type_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_line_type(LineType {
            name: String::from("some-line-type"),
            ..Default::default()
        });
        let line_types = drawing
            .line_types()
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
            .line_types()
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
            .line_types()
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
            .line_types()
            .filter(|&lt| lt.name == "some-line-type")
            .collect::<Vec<_>>();
        assert_eq!(1, line_types.len());
    }

    #[test]
    fn text_style_is_added_with_entity_if_not_already_present() {
        let mut drawing = Drawing::new();
        let text_styles = drawing
            .styles()
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
            .styles()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let text_styles = drawing
            .styles()
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
            .styles()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_not_added_with_entity_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_style(Style {
            name: String::from("some-text-style"),
            ..Default::default()
        });
        let text_styles = drawing
            .styles()
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
            .styles()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn text_style_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_style(Style {
            name: String::from("some-text-style"),
            ..Default::default()
        });
        let text_styles = drawing
            .styles()
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
            .styles()
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
            .styles()
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
            .styles()
            .filter(|&s| s.name == "some-text-style")
            .collect::<Vec<_>>();
        assert_eq!(1, text_styles.len());
    }

    #[test]
    fn view_is_added_with_object_if_not_already_present() {
        let mut drawing = Drawing::new();
        let views = drawing
            .views()
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
            .views()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(1, views.len());
    }

    #[test]
    fn view_is_not_added_with_object_if_already_present() {
        let mut drawing = Drawing::new();
        drawing.add_view(View {
            name: String::from("some-view"),
            ..Default::default()
        });
        let views = drawing
            .views()
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
            .views()
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
            .views()
            .filter(|&v| v.name == "some-view")
            .collect::<Vec<_>>();
        assert_eq!(1, views.len());
    }
}
