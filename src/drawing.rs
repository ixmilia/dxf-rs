use encoding_rs::Encoding;

use image::DynamicImage;

use crate::code_pair_put_back::CodePairPutBack;
use crate::drawing_item::{DrawingItem, DrawingItemMut};
use crate::entities::*;
use crate::enums::*;
use crate::header::*;
use crate::objects::*;
use crate::tables::*;

use crate::{CodePair, CodePairValue, DxfError, DxfResult, Handle};

use crate::dxb_reader::DxbReader;
use crate::dxb_writer::DxbWriter;
use crate::entity_iter::EntityIter;
use crate::helper_functions::*;
use crate::object_iter::ObjectIter;

use crate::block::Block;
use crate::class::Class;

use crate::code_pair_iter::{new_code_pair_iter_from_reader, CodePairIter};
use crate::code_pair_writer::CodePairWriter;

use crate::thumbnail;

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};

use itertools::put_back;
use std::collections::HashSet;
use std::iter::Iterator;
use std::path::Path;

pub(crate) const AUTO_REPLACE_HANDLE: Handle = Handle(0xFFFF_FFFF_FFFF_FFFF);

/// Represents a DXF drawing.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
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

    /// Internal collection of blocks.
    __blocks: Vec<Block>,

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
            __blocks: vec![],
            __entities: vec![],
            __objects: vec![],
            thumbnail: None,
        };
        drawing.normalize();
        drawing
    }
    /// Loads a `Drawing` from anything that implements the `Read` trait.
    pub fn load<'a, T>(reader: &mut T) -> DxfResult<Drawing>
    where
        T: Read + 'a + ?Sized,
    {
        Drawing::load_with_encoding(reader, encoding_rs::WINDOWS_1252)
    }
    /// Loads a `Drawing` from anything that implements the `Read` trait using the specified text encoding.
    pub fn load_with_encoding<T>(reader: &mut T, encoding: &'static Encoding) -> DxfResult<Drawing>
    where
        T: Read + ?Sized,
    {
        let first_line = read_line(reader, true, encoding)?;
        match &*first_line {
            "AutoCAD DXB 1.0" => {
                let mut reader = DxbReader::new(reader);
                reader.load()
            }
            _ => {
                let iter = new_code_pair_iter_from_reader(reader, encoding, first_line)?;
                Drawing::load_from_iter(iter)
            }
        }
    }
    /// Loads a `Drawing` from the specified `CodePairIter`.
    pub(crate) fn load_from_iter(iter: Box<dyn CodePairIter>) -> DxfResult<Drawing> {
        let mut drawing = Drawing::new();
        drawing.clear();
        let mut iter = CodePairPutBack::from_code_pair_iter(iter);
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
    /// Loads a `Drawing` from disk, using a `BufReader`.
    pub fn load_file(path: impl AsRef<Path>) -> DxfResult<Drawing> {
        Drawing::load_file_with_encoding(path, encoding_rs::WINDOWS_1252)
    }
    /// Loads a `Drawing` from disk, using a `BufReader` with the specified text encoding.
    pub fn load_file_with_encoding(
        path: impl AsRef<Path>,
        encoding: &'static Encoding,
    ) -> DxfResult<Drawing> {
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
    /// Gets all code pairs that will be written.
    pub(crate) fn code_pairs(&self) -> DxfResult<Vec<CodePair>> {
        let write_handles = self.header.version >= AcadVersion::R13 || self.header.handles_enabled;
        let mut pairs = Vec::new();
        self.header.add_code_pairs(&mut pairs);
        self.add_classes_pairs(&mut pairs);
        self.add_tables_pairs(&mut pairs, write_handles);
        self.add_blocks_pairs(&mut pairs, write_handles);
        self.add_entities_pairs(&mut pairs, write_handles);
        self.add_objects_pairs(&mut pairs);
        self.add_thumbnail_pairs(&mut pairs)?;
        pairs.push(CodePair::new_str(0, "EOF"));
        Ok(pairs)
    }
    fn save_internal<T>(&self, writer: &mut T, as_ascii: bool) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        let pairs = self.code_pairs()?;
        let text_as_ascii = self.header.version <= AcadVersion::R2004;
        let mut code_pair_writer =
            CodePairWriter::new(writer, as_ascii, text_as_ascii, self.header.version);
        code_pair_writer.write_prelude()?;
        for pair in pairs {
            code_pair_writer.write_code_pair(&pair)?;
        }
        Ok(())
    }
    /// Writes a `Drawing` to disk, using a `BufWriter`.
    pub fn save_file(&self, path: impl AsRef<Path>) -> DxfResult<()> {
        self.save_file_internal(path, true)
    }
    /// Writes a `Drawing` as binary to disk, using a `BufWriter`.
    pub fn save_file_binary(&self, path: impl AsRef<Path>) -> DxfResult<()> {
        self.save_file_internal(path, false)
    }
    fn save_file_internal(&self, path: impl AsRef<Path>, as_ascii: bool) -> DxfResult<()> {
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
    pub fn save_file_dxb(&self, path: impl AsRef<Path>) -> DxfResult<()> {
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
    /// Removes the specified `AppId` from the `Drawing`.
    pub fn remove_app_id(&mut self, index: usize) -> Option<AppId> {
        Drawing::remove_item(&mut self.__app_ids, index)
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
    /// Removes the specified `BlockRecord` from the `Drawing`.
    pub fn remove_block_record(&mut self, index: usize) -> Option<BlockRecord> {
        Drawing::remove_item(&mut self.__block_records, index)
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
    /// Removes the specified `DimStyle` from the `Drawing`.
    pub fn remove_dim_style(&mut self, index: usize) -> Option<DimStyle> {
        Drawing::remove_item(&mut self.__dim_styles, index)
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
    /// Removes the specified `Layer` from the `Drawing`.
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        Drawing::remove_item(&mut self.__layers, index)
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
    /// Removes the specified `LineType` from the `Drawing`.
    pub fn remove_line_type(&mut self, index: usize) -> Option<LineType> {
        Drawing::remove_item(&mut self.__line_types, index)
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
    /// Removes the specified `Style` from the `Drawing`.
    pub fn remove_style(&mut self, index: usize) -> Option<Style> {
        Drawing::remove_item(&mut self.__styles, index)
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
    /// Removes the specified `Ucs` from the `Drawing`.
    pub fn remove_ucs(&mut self, index: usize) -> Option<Ucs> {
        Drawing::remove_item(&mut self.__ucss, index)
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
    /// Removes the specified `View` from the `Drawing`.
    pub fn remove_view(&mut self, index: usize) -> Option<View> {
        Drawing::remove_item(&mut self.__views, index)
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
    /// Removes the specified `ViewPort` from the `Drawing`.
    pub fn remove_view_port(&mut self, index: usize) -> Option<ViewPort> {
        Drawing::remove_item(&mut self.__view_ports, index)
    }
    /// Returns an iterator for all blocks.
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.__blocks.iter()
    }
    /// Returns an iterator for all mutable blocks.
    pub fn blocks_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        self.__blocks.iter_mut()
    }
    /// Add a block to the `Drawing`.
    pub fn add_block(&mut self, mut block: Block) -> &Block {
        block.handle = self.next_handle();
        self.add_block_no_handle_set(block)
    }
    /// Removes the specified `Block` from the `Drawing`.
    pub fn remove_block(&mut self, index: usize) -> Option<Block> {
        Drawing::remove_item(&mut self.__blocks, index)
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
                ins.__seqend_handle = self.next_handle();
                for a in ins.__attributes_and_handles.iter_mut() {
                    if a.1 == AUTO_REPLACE_HANDLE {
                        a.1 = self.next_handle();
                    }
                }
            }
            EntityType::Polyline(ref mut poly) => {
                poly.__seqend_handle = self.next_handle();
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
    /// Removes the specified `Entity` from the `Drawing`.
    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        Drawing::remove_item(&mut self.__entities, index)
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
    /// Removes the specified `Object` from the `Drawing`.
    pub fn remove_object(&mut self, index: usize) -> Option<Object> {
        Drawing::remove_item(&mut self.__objects, index)
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
        self.__blocks.clear();
        self.__entities.clear();
        self.__objects.clear();
        self.thumbnail = None;

        self.header.next_available_handle = Handle(1);
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
    pub fn item_by_handle(&'_ self, handle: Handle) -> Option<DrawingItem<'_>> {
        for item in &self.__app_ids {
            if item.handle == handle {
                return Some(DrawingItem::AppId(item));
            }
        }
        for item in &self.__blocks {
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
    pub fn item_by_handle_mut(&'_ mut self, handle: Handle) -> Option<DrawingItemMut<'_>> {
        for item in &mut self.__app_ids {
            if item.handle == handle {
                return Some(DrawingItemMut::AppId(item));
            }
        }
        for item in &mut self.__blocks {
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
    pub(crate) fn assign_and_get_handle(&mut self, item: &mut DrawingItemMut) -> Handle {
        if item.handle().is_empty() {
            item.set_handle(self.header.next_available_handle);
            self.header.next_available_handle =
                self.header.next_available_handle.next_handle_value();
        }

        item.handle()
    }
}

// private implementation
impl Drawing {
    pub(crate) fn next_handle(&mut self) -> Handle {
        let result = self.header.next_available_handle;
        self.header.next_available_handle = self.header.next_available_handle.next_handle_value();
        result
    }
    fn remove_item<T>(collection: &mut Vec<T>, index: usize) -> Option<T> {
        if index < collection.len() {
            Some(collection.remove(index))
        } else {
            None
        }
    }
    pub(crate) fn add_block_no_handle_set(&mut self, mut block: Block) -> &Block {
        self.ensure_layer_is_present_for_block(&block);
        self.ensure_line_type_is_present_for_block(&block);
        self.ensure_block_record_is_present_for_block(&mut block);
        self.ensure_block_entity_handles_are_set(&mut block);
        self.__blocks.push(block);
        self.__blocks.last().unwrap()
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
        self.ensure_line_type_is_present(&layer.line_type_name);
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
            self.ensure_dimension_style_is_present(dim_style_name);
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
    fn ensure_layer_is_present_for_block(&mut self, block: &Block) {
        self.ensure_layer_is_present(&block.layer);
        for ent in &block.entities {
            self.ensure_layer_is_present(&ent.common.layer);
        }
    }
    fn ensure_layer_is_present_for_object(&mut self, obj: &Object) {
        match &obj.specific {
            ObjectType::LayerFilter(ref l) => {
                for layer_name in &l.layer_names {
                    self.ensure_layer_is_present(layer_name);
                }
            }
            ObjectType::LayerIndex(ref l) => {
                for layer_name in &l.layer_names {
                    self.ensure_layer_is_present(layer_name);
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
    fn ensure_line_type_is_present_for_block(&mut self, block: &Block) {
        for ent in &block.entities {
            self.ensure_line_type_is_present(&ent.common.line_type_name);
        }
    }
    fn ensure_block_record_is_present_for_block(&mut self, block: &mut Block) {
        self.add_block_record(BlockRecord {
            name: String::from(&block.name),
            ..Default::default()
        });
    }
    fn ensure_block_entity_handles_are_set(&mut self, block: &mut Block) {
        for ent in &mut block.entities {
            ent.common.handle = self.next_handle();
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
            self.ensure_text_style_is_present(text_style_name);
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
    pub(crate) fn add_classes_pairs(&self, pairs: &mut Vec<CodePair>) {
        if self.classes.is_empty() {
            return;
        }

        pairs.push(CodePair::new_str(0, "SECTION"));
        pairs.push(CodePair::new_str(2, "CLASSES"));
        for c in &self.classes {
            c.add_code_pairs(pairs, self.header.version);
        }

        pairs.push(CodePair::new_str(0, "ENDSEC"));
    }
    pub(crate) fn add_tables_pairs(&self, pairs: &mut Vec<CodePair>, write_handles: bool) {
        pairs.push(CodePair::new_str(0, "SECTION"));
        pairs.push(CodePair::new_str(2, "TABLES"));
        add_table_code_pairs(self, pairs, write_handles);
        pairs.push(CodePair::new_str(0, "ENDSEC"));
    }
    pub(crate) fn add_blocks_pairs(&self, pairs: &mut Vec<CodePair>, write_handles: bool) {
        if self.__blocks.is_empty() {
            return;
        }

        pairs.push(CodePair::new_str(0, "SECTION"));
        pairs.push(CodePair::new_str(2, "BLOCKS"));
        for b in &self.__blocks {
            b.add_code_pairs(pairs, self.header.version, write_handles);
        }

        pairs.push(CodePair::new_str(0, "ENDSEC"));
    }
    pub(crate) fn add_entities_pairs(&self, pairs: &mut Vec<CodePair>, write_handles: bool) {
        pairs.push(CodePair::new_str(0, "SECTION"));
        pairs.push(CodePair::new_str(2, "ENTITIES"));
        for e in &self.__entities {
            e.add_code_pairs(pairs, self.header.version, write_handles);
        }

        pairs.push(CodePair::new_str(0, "ENDSEC"));
    }
    pub(crate) fn add_objects_pairs(&self, pairs: &mut Vec<CodePair>) {
        if self.header.version >= AcadVersion::R13 {
            pairs.push(CodePair::new_str(0, "SECTION"));
            pairs.push(CodePair::new_str(2, "OBJECTS"));
            for o in &self.__objects {
                o.add_code_pairs(pairs, self.header.version);
            }

            pairs.push(CodePair::new_str(0, "ENDSEC"));
        }
    }
    pub(crate) fn add_thumbnail_pairs(&self, pairs: &mut Vec<CodePair>) -> DxfResult<()> {
        if self.header.version >= AcadVersion::R2000 {
            if let Some(ref img) = self.thumbnail {
                pairs.push(CodePair::new_str(0, "SECTION"));
                pairs.push(CodePair::new_str(2, "THUMBNAILIMAGE"));
                let mut data = vec![];
                img.write_to(&mut Cursor::new(&mut data), image::ImageFormat::Bmp)?;
                let length = data.len() - 14; // skip 14 byte bmp header
                pairs.push(CodePair::new_i32(90, length as i32));
                for s in data[14..].chunks(128) {
                    let pair = CodePair::new_binary(310, s.to_vec());
                    pairs.push(pair);
                }
                pairs.push(CodePair::new_str(0, "ENDSEC"));
            }
        }
        Ok(())
    }
    fn read_sections(drawing: &mut Drawing, iter: &mut CodePairPutBack) -> DxfResult<()> {
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
                                    drawing.thumbnail = thumbnail::read_thumbnail(iter)?;
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
    fn swallow_section(iter: &mut CodePairPutBack) -> DxfResult<()> {
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
    fn read_entities(&mut self, iter: &mut CodePairPutBack) -> DxfResult<()> {
        let mut iter = EntityIter { iter };
        let mut entities = vec![];
        iter.read_entities_into_vec(&mut entities)?;
        for e in entities {
            if e.common.handle.is_empty() {
                self.add_entity(e);
            } else {
                self.add_entity_no_handle_set(e);
            }
        }
        Ok(())
    }
    fn read_objects(&mut self, iter: &mut CodePairPutBack) -> DxfResult<()> {
        let iter = put_back(ObjectIter { iter });
        for o in iter {
            if o.common.handle.is_empty() {
                self.add_object(o);
            } else {
                self.add_object_no_handle_set(o);
            }
        }

        Ok(())
    }
    fn read_section_item<F>(
        &mut self,
        iter: &mut CodePairPutBack,
        item_type: &str,
        callback: F,
    ) -> DxfResult<()>
    where
        F: Fn(&mut Drawing, &mut CodePairPutBack) -> DxfResult<()>,
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
    pub(crate) fn swallow_table(iter: &mut CodePairPutBack) -> DxfResult<()> {
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
        for b in self.blocks_mut() {
            b.normalize();
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
        self.ensure_layer_is_present("0");
    }
    fn ensure_line_types(&mut self) {
        // ensure all line_types that should exist do
        self.ensure_line_type_is_present("BYLAYER");
        self.ensure_line_type_is_present("BYBLOCK");
        self.ensure_line_type_is_present("CONTINUOUS");
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
    use crate::enums::AcadVersion;
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
    fn block_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        let block = Block::default();
        assert_eq!(Handle(0), block.handle);

        let block = drawing.add_block(block);
        assert_ne!(Handle(0), block.handle);
    }

    #[test]
    fn entity_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        let ent = Entity {
            common: Default::default(),
            specific: EntityType::Line(Default::default()),
        };
        assert_eq!(Handle(0), ent.common.handle);

        let ent = drawing.add_entity(ent);
        assert_ne!(Handle(0), ent.common.handle);
    }

    #[test]
    fn object_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        let obj = Object {
            common: Default::default(),
            specific: ObjectType::PlaceHolder(Default::default()),
        };
        assert_eq!(Handle(0), obj.common.handle);

        let obj = drawing.add_object(obj);
        assert_ne!(Handle(0), obj.common.handle);
    }

    #[test]
    fn layer_handle_is_set_on_add() {
        let mut drawing = Drawing::new();
        drawing.clear();
        let layer = Layer::default();
        assert_eq!(Handle(0), layer.handle);

        let layer = drawing.add_layer(layer);
        assert_ne!(Handle(0), layer.handle);
    }

    #[test]
    fn objects_section_is_not_written_on_r12() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R12;
        assert_not_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "OBJECTS"),
            ],
        );
    }

    #[test]
    fn objects_section_is_written_on_r13() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R13;
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "OBJECTS"),
            ],
        );
    }

    #[test]
    fn block_handle_is_set_during_read_if_not_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "BLOCKS"),
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(0, "ENDBLK"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let block = drawing.blocks().next().unwrap();
        assert_ne!(Handle(0), block.handle);
    }

    #[test]
    fn entity_handle_is_set_during_read_if_not_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let line = drawing.entities().next().unwrap();
        assert_ne!(Handle(0), line.common.handle);
    }

    #[test]
    fn object_handle_is_set_during_read_if_not_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "ACDBPLACEHOLDER"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let obj = drawing.objects().next().unwrap();
        assert_ne!(Handle(0), obj.common.handle);
    }

    #[test]
    fn layer_handle_is_set_during_read_if_not_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "TABLES"),
            CodePair::new_str(0, "TABLE"),
            CodePair::new_str(2, "LAYER"),
            CodePair::new_str(0, "LAYER"),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let layer = drawing.layers().next().unwrap();
        assert_ne!(Handle(0), layer.handle);
    }

    #[test]
    fn block_handle_is_honored_during_read_if_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "BLOCKS"),
            CodePair::new_str(0, "BLOCK"),
            CodePair::new_str(5, "3333"),
            CodePair::new_str(0, "ENDBLK"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let block = drawing.blocks().next().unwrap();
        assert_eq!(Handle(0x3333), block.handle);
    }

    #[test]
    fn entity_handle_is_honored_during_read_if_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(5, "3333"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let line = drawing.entities().next().unwrap();
        assert_eq!(Handle(0x3333), line.common.handle);
    }

    #[test]
    fn object_handle_is_honored_during_read_if_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "ACDBPLACEHOLDER"),
            CodePair::new_str(5, "3333"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let obj = drawing.objects().next().unwrap();
        assert_eq!(Handle(0x3333), obj.common.handle);
    }

    #[test]
    fn layer_handle_is_honored_during_read_if_specified() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "TABLES"),
            CodePair::new_str(0, "TABLE"),
            CodePair::new_str(2, "LAYER"),
            CodePair::new_str(0, "LAYER"),
            CodePair::new_str(5, "3333"),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let layer = drawing.layers().next().unwrap();
        assert_eq!(Handle(0x3333), layer.handle);
    }

    #[test]
    fn next_available_handle_is_reset_on_clear() {
        let mut drawing = Drawing::new();
        drawing.add_entity(Entity {
            common: EntityCommon::default(),
            specific: EntityType::Line(Line::default()),
        });
        assert_eq!(1, drawing.entities().count());
        assert_ne!(Handle(0), drawing.header.next_available_handle);
        assert_ne!(Handle(1), drawing.header.next_available_handle);

        drawing.clear();
        assert_eq!(0, drawing.entities().count());
        assert_eq!(Handle(1), drawing.header.next_available_handle);
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
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "MLINE"),
            CodePair::new_str(2, "some-mline-style"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let mline_styles = drawing.objects().filter(|&o| match o.specific {
            ObjectType::MLineStyle(ref mline_style) => mline_style.style_name == "some-mline-style",
            _ => false,
        });
        assert_eq!(1, mline_styles.count());
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
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "DIMENSION"),
            CodePair::new_str(3, "some-dim-style"),
            CodePair::new_str(100, "AcDbRadialDimension"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let dim_styles = drawing.dim_styles().filter(|&d| d.name == "some-dim-style");
        assert_eq!(1, dim_styles.count());
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
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(8, "some-layer"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let layers = drawing.layers().filter(|&l| l.name == "some-layer");
        assert_eq!(1, layers.count());
    }

    #[test]
    fn layer_is_added_with_object_on_file_read() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "LAYER_FILTER"),
            CodePair::new_str(8, "some-layer"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let layers = drawing.layers().filter(|&l| l.name == "some-layer");
        assert_eq!(1, layers.count());
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
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(6, "some-line-type"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let line_types = drawing
            .line_types()
            .filter(|&lt| lt.name == "some-line-type");
        assert_eq!(1, line_types.count());
    }

    #[test]
    fn line_type_is_added_with_object_on_file_read() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "MLINESTYLE"),
            CodePair::new_str(2, "some-line-type"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let line_types = drawing
            .line_types()
            .filter(|&lt| lt.name == "some-line-type");
        assert_eq!(1, line_types.count());
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
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "ENTITIES"),
            CodePair::new_str(0, "TEXT"),
            CodePair::new_str(7, "some-text-style"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let text_styles = drawing.styles().filter(|&s| s.name == "some-text-style");
        assert_eq!(1, text_styles.count());
    }

    #[test]
    fn text_style_is_added_with_object_on_file_read() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "MLINESTYLE"),
            CodePair::new_str(2, "some-text-style"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let text_styles = drawing.styles().filter(|&s| s.name == "some-text-style");
        assert_eq!(1, text_styles.count());
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
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "PLOTSETTINGS"),
            CodePair::new_str(6, "some-view"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let views = drawing.views().filter(|&v| v.name == "some-view");
        assert_eq!(1, views.count());
    }
}
