// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

//! This crate provides the ability to read and write DXF CAD files.
//!
//! # Examples
//!
//! Open a DXF file from disk:
//!
//! ``` rust
//! # fn main() { }
//! # fn ex() -> dxf::DxfResult<()> {
//! use dxf::Drawing;
//! use dxf::entities::*;
//!
//! let drawing = try!(Drawing::load_file("path/to/file.dxf"));
//! for e in drawing.entities {
//!     println!("found entity on layer {}", e.common.layer);
//!     match e.specific {
//!         EntityType::Circle(ref circle) => {
//!             // do something with the circle
//!         },
//!         EntityType::Line(ref line) => {
//!             // do something with the line
//!         },
//!         _ => (),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Saving a DXF file to disk:
//!
//! ``` rust
//! # fn main() { }
//! # fn ex() -> dxf::DxfResult<()> {
//! use dxf::Drawing;
//! use dxf::entities::*;
//!
//! let mut drawing = Drawing::new();
//! drawing.entities.push(Entity::new(EntityType::Line(Line::default())));
//! try!(drawing.save_file("path/to/file.dxf"));
//! # Ok(())
//! # }
//! ```
//!
//! # Reference
//!
//! Since I don't want to fall afoul of Autodesk's lawyers, this repo can't include the actual DXF documentation.  It can,
//! however contain links to the official documents that I've been able to scrape together.  For most scenarios the 2014
//! documentation should suffice, but all other versions are included here for backwards compatibility and reference
//! between versions.
//!
//! [R10 (non-Autodesk source)](http://www.martinreddy.net/gfx/3d/DXF10.spec)
//!
//! [R11 (differences between R10 and R11)](http://autodesk.blogs.com/between_the_lines/ACAD_R11.html)
//!
//! [R12 (non-Autodesk source)](http://www.martinreddy.net/gfx/3d/DXF12.spec)
//!
//! [R13 (self-extracting 16-bit executable)](http://www.autodesk.com/techpubs/autocad/dxf/dxf13_hlp.exe)
//!
//! [R14](http://www.autodesk.com/techpubs/autocad/acadr14/dxf/index.htm)
//!
//! [2000](http://www.autodesk.com/techpubs/autocad/acad2000/dxf/index.htm)
//!
//! [2002](http://www.autodesk.com/techpubs/autocad/dxf/dxf2002.pdf)
//!
//! [2004](http://download.autodesk.com/prodsupp/downloads/dxf.pdf)
//!
//! [2005](http://download.autodesk.com/prodsupp/downloads/acad_dxf.pdf)
//!
//! [2006](http://images.autodesk.com/adsk/files/dxf_format.pdf)
//!
//! 2007 (Autodesk's link erroneously points to the R2008 documentation)
//!
//! [2008](http://images.autodesk.com/adsk/files/acad_dxf0.pdf)
//!
//! [2009](http://images.autodesk.com/adsk/files/acad_dxf.pdf)
//!
//! [2010](http://images.autodesk.com/adsk/files/acad_dxf1.pdf)
//!
//! [2011](http://images.autodesk.com/adsk/files/acad_dxf2.pdf)
//!
//! [2012](http://images.autodesk.com/adsk/files/autocad_2012_pdf_dxf-reference_enu.pdf)
//!
//! [2013](http://images.autodesk.com/adsk/files/autocad_2013_pdf_dxf_reference_enu.pdf)
//!
//! [2014](http://images.autodesk.com/adsk/files/autocad_2014_pdf_dxf_reference_enu.pdf)
//!
//! These links were compiled from the archive.org May 9, 2013 snapshot of http://usa.autodesk.com/adsk/servlet/item?siteID=123112&id=12272454&linkID=10809853
//! (https://web.archive.org/web/20130509144333/http://usa.autodesk.com/adsk/servlet/item?siteID=123112&id=12272454&linkID=10809853)

#[macro_use] extern crate enum_primitive;

extern crate itertools;

use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::Write;
use std::num;
use std::ops::Add;

mod code_pair;
pub use code_pair::CodePair;

mod code_pair_value;
pub use code_pair_value::CodePairValue;

mod data_table_value;
pub use data_table_value::DataTableValue;

mod drawing;
pub use drawing::Drawing;

//mod entity;

mod section_geometry_settings;
pub use section_geometry_settings::SectionGeometrySettings;

mod section_type_settings;
pub use section_type_settings::SectionTypeSettings;

mod table_cell_style;
pub use table_cell_style::TableCellStyle;

mod transformation_matrix;
pub use transformation_matrix::TransformationMatrix;

pub mod enums;
use enums::*;

mod generated;
pub mod entities {
    pub use generated::entities::*;
}
pub mod header {
    pub use generated::header::*;
}
pub mod tables {
    pub use generated::tables::*;
}
pub mod objects {
    pub use generated::objects::*;
}

use entities::*;
use header::*;
use tables::*;
use objects::*;

extern crate chrono;
use chrono::Duration;
use enum_primitive::FromPrimitive;

use itertools::{
    Itertools,
    PutBack,
};

include!("expected_type.rs");

mod code_pair_iter;
mod code_pair_writer;
use code_pair_writer::CodePairWriter;

mod block;
pub use block::Block;

mod class;
pub use class::Class;

mod helper_functions;
use helper_functions::*;

// returns the next CodePair that's not 0, or bails out early
macro_rules! next_pair {
    ($expr : expr) => (
        match $expr.next() {
            Some(Ok(pair @ CodePair { code: 0, .. })) => {
                $expr.put_back(Ok(pair));
                return Ok(true);
            },
            Some(Ok(pair)) => pair,
            Some(Err(e)) => return Err(e),
            None => return Ok(true),
        }
    )
}
// Used to turn Option<T> into DxfResult<T>.
macro_rules! try_result {
    ($expr : expr) => (
        match $expr {
            Some(v) => v,
            None => return Err(DxfError::UnexpectedEnumValue)
        }
    )
}
// Used to safely access the last element in a Vec<T>
macro_rules! vec_last {
    ($expr : expr) => (
        match $expr.len() {
            0 => return Err(DxfError::UnexpectedEmptySet),
            l => &mut $expr[l - 1],
        }
    )
}

mod entity;

//------------------------------------------------------------------------------
//                                                                  DxfResult<T>
//------------------------------------------------------------------------------
pub type DxfResult<T> = Result<T, DxfError>;

#[derive(Debug)]
pub enum DxfError {
    IoError(io::Error),
    ParseFloatError(num::ParseFloatError),
    ParseIntError(num::ParseIntError),
    ParseError,
    UnexpectedCode(i32),
    UnexpectedCodePair(CodePair, String),
    UnexpectedByte(u8),
    UnexpectedEndOfInput,
    UnexpectedEnumValue,
    UnexpectedEmptySet,
    ExpectedTableType,
    WrongValueType,
}

impl From<io::Error> for DxfError {
    fn from(ioe: io::Error) -> DxfError {
        DxfError::IoError(ioe)
    }
}

impl Display for DxfError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DxfError::IoError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseFloatError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseIntError(ref e) => write!(formatter, "{}", e),
            &DxfError::ParseError => write!(formatter, "there was a general parsing error"),
            &DxfError::UnexpectedCode(c) => write!(formatter, "an unexpected code '{}' was encountered", c),
            &DxfError::UnexpectedCodePair(ref cp, ref s) => write!(formatter, "the code pair '{:?}' was not expected at this time: {}", cp, s),
            &DxfError::UnexpectedByte(ref b) => write!(formatter, "the byte '{:x}' was not expected at this time", b),
            &DxfError::UnexpectedEndOfInput => write!(formatter, "the input unexpectedly ended before the drawing was completely loaded"),
            &DxfError::UnexpectedEnumValue => write!(formatter, "the specified enum value does not fall into the expected range"),
            &DxfError::UnexpectedEmptySet => write!(formatter, "the set was not expected to be empty"),
            &DxfError::ExpectedTableType => write!(formatter, "a 2/<table-type> code pair was expected"),
            &DxfError::WrongValueType => write!(formatter, "the CodePairValue does not contain the requested type"),
        }
    }
}

impl std::error::Error for DxfError {
    fn description(&self) -> &str {
        match self {
            &DxfError::IoError(ref e) => e.description(),
            &DxfError::ParseFloatError(ref e) => e.description(),
            &DxfError::ParseIntError(ref e) => e.description(),
            &DxfError::ParseError => "there was a general parsing error",
            &DxfError::UnexpectedCode(_) => "an unexpected code was encountered",
            &DxfError::UnexpectedCodePair(_, _) => "an unexpected code pair was encountered",
            &DxfError::UnexpectedByte(_) => "an unexpected byte was encountered",
            &DxfError::UnexpectedEndOfInput => "the input unexpectedly ended before the drawing was completely loaded",
            &DxfError::UnexpectedEnumValue => "the specified enum value does not fall into the expected range",
            &DxfError::UnexpectedEmptySet => "the set was not expected to be empty",
            &DxfError::ExpectedTableType => "a 2/<table-type> code pair was expected",
            &DxfError::WrongValueType => "the CodePairValue does not contain the requested type",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match self {
            &DxfError::IoError(ref e) => Some(e),
            &DxfError::ParseFloatError(ref e) => Some(e),
            &DxfError::ParseIntError(ref e) => Some(e),
            _ => None,
        }
    }
}

//------------------------------------------------------------------------------
//                                                                        Header
//------------------------------------------------------------------------------
// implementation is in `generated/header.rs`
impl Header {
    #[doc(hidden)]
    pub fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Header>
        where I: Iterator<Item = DxfResult<CodePair>> {

        let mut header = Header::new();
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    match pair.code {
                        0 => {
                            iter.put_back(Ok(pair));
                            break;
                        },
                        9 => {
                            let last_header_variable = try!(pair.value.assert_string());
                            loop {
                                match iter.next() {
                                    Some(Ok(pair)) => {
                                        if pair.code == 0 || pair.code == 9 {
                                            // ENDSEC or a new header variable
                                            iter.put_back(Ok(pair));
                                            break;
                                        }
                                        else {
                                            try!(header.set_header_value(&last_header_variable, &pair));
                                        }
                                    },
                                    Some(Err(e)) => return Err(e),
                                    None => break,
                                }
                            }
                        },
                        _ => return Err(DxfError::UnexpectedCodePair(pair, String::from(""))),
                    }
                },
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(header)
    }
    #[doc(hidden)]
    pub fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "HEADER")));
        try!(self.write_code_pairs(writer));
        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                                  GeoMeshPoint
//------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct GeoMeshPoint {
    pub source: Point,
    pub destination: Point,
}

impl GeoMeshPoint {
    pub fn new(source: Point, destination: Point) -> Self {
        GeoMeshPoint {
            source: source,
            destination: destination,
        }
    }
}

//------------------------------------------------------------------------------
//                                                             MLineStyleElement
//------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct MLineStyleElement {
    pub offset: f64,
    pub color: Color,
    pub linetype: String,
}

impl MLineStyleElement {
    pub fn new(offset: f64, color: Color, linetype: String) -> Self {
        MLineStyleElement {
            offset: offset,
            color: color,
            linetype: linetype,
        }
    }
}

// implementation is in `generated/objects.rs`
impl Object {
    /// Creates a new `Object` with the default common values.
    pub fn new(specific: ObjectType) -> Self {
        Object {
            common: ObjectCommon::new(),
            specific: specific,
        }
    }
    #[doc(hidden)]
    pub fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Option<Object>>
        where I: Iterator<Item = DxfResult<CodePair>> {

        loop {
            match iter.next() {
                // first code pair must be 0/object-type
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    let type_string = try!(pair.value.assert_string());
                    if type_string == "ENDSEC" || type_string == "ENDBLK" {
                        iter.put_back(Ok(pair));
                        return Ok(None);
                    }

                    match ObjectType::from_type_string(&type_string) {
                        Some(e) => {
                            let mut obj = Object::new(e);
                            if !try!(obj.apply_custom_reader(iter)) {
                                // no custom reader, use the auto-generated one
                                loop {
                                    match iter.next() {
                                        Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                            // new object or ENDSEC
                                            iter.put_back(Ok(pair));
                                            break;
                                        },
                                        Some(Ok(pair)) => try!(obj.apply_code_pair(&pair)),
                                        Some(Err(e)) => return Err(e),
                                        None => return Err(DxfError::UnexpectedEndOfInput),
                                    }
                                }

                                try!(obj.post_parse());
                            }

                            return Ok(Some(obj));
                        },
                        None => {
                            // swallow unsupported object
                            loop {
                               match iter.next() {
                                    Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                        // found another object or ENDSEC
                                        iter.put_back(Ok(pair));
                                        break;
                                    },
                                    Some(Ok(_)) => (), // part of the unsupported object
                                    Some(Err(e)) => return Err(e),
                                    None => return Err(DxfError::UnexpectedEndOfInput),
                                }
                            }
                        }
                    }
                },
                Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/object-type or 0/ENDSEC"))),
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }
    }
    fn apply_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        if !try!(self.specific.try_apply_code_pair(&pair)) {
            try!(self.common.apply_individual_pair(&pair));
        }
        Ok(())
    }
    fn post_parse(&mut self) -> DxfResult<()> {
        match self.specific {
            ObjectType::AcadProxyObject(ref mut proxy) => {
                for item in &proxy._object_ids_a {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy._object_ids_b {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy._object_ids_c {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy._object_ids_d {
                    proxy.object_ids.push(item.clone());
                }
                proxy._object_ids_a.clear();
                proxy._object_ids_b.clear();
                proxy._object_ids_c.clear();
                proxy._object_ids_d.clear();
            },
            ObjectType::GeoData(ref mut geo) => {
                let mut source_points = vec![];
                let mut destination_points = vec![];
                combine_points_2(&mut geo._source_mesh_x_points, &mut geo._source_mesh_y_points, &mut source_points, Point::new);
                combine_points_2(&mut geo._destination_mesh_x_points, &mut geo._destination_mesh_y_points, &mut destination_points, Point::new);
                for (s, d) in source_points.drain(..).zip(destination_points.drain(..)) {
                    geo.geo_mesh_points.push(GeoMeshPoint::new(s, d));
                }

                combine_points_3(&mut geo._face_point_index_x, &mut geo._face_point_index_y, &mut geo._face_point_index_z, &mut geo.face_indices, Point::new);
            },
            ObjectType::Material(ref mut material) => {
                material.diffuse_map_transformation_matrix.from_vec(&material._diffuse_map_transformation_matrix_values);
                material.specular_map_transformation_matrix.from_vec(&material._specular_map_transformation_matrix_values);
                material.reflection_map_transformation_matrix.from_vec(&material._reflection_map_transformation_matrix_values);
                material.opacity_map_transformation_matrix.from_vec(&material._opacity_map_transformation_matrix_values);
                material.bump_map_transformation_matrix.from_vec(&material._bump_map_transformation_matrix_values);
                material.refraction_map_transformation_matrix.from_vec(&material._refraction_map_transformation_matrix_values);
                material.normal_map_transformation_matrix.from_vec(&material._normal_map_transformation_matrix_values);
                material._diffuse_map_transformation_matrix_values.clear();
                material._specular_map_transformation_matrix_values.clear();
                material._reflection_map_transformation_matrix_values.clear();
                material._opacity_map_transformation_matrix_values.clear();
                material._bump_map_transformation_matrix_values.clear();
                material._refraction_map_transformation_matrix_values.clear();
                material._normal_map_transformation_matrix_values.clear();
            },
            ObjectType::MLineStyle(ref mut mline) => {
                for (o, (c, l)) in mline._element_offsets.drain(..).zip(mline._element_colors.drain(..).zip(mline._element_linetypes.drain(..))) {
                    mline.elements.push(MLineStyleElement::new(o, c, l));
                }
            },
            ObjectType::VbaProject(ref mut vba) => {
                // each char in each _hex_data should be added to `data` byte array
                let mut result = vec![];
                let mut complete_byte = false;
                let mut current_byte = 0u8;
                for s in &vba._hex_data {
                    for c in s.chars() {
                        let value = match c {
                            '0' => 0,
                            '1' => 1,
                            '2' => 2,
                            '3' => 3,
                            '4' => 4,
                            '5' => 5,
                            '6' => 6,
                            '7' => 7,
                            '8' => 8,
                            '9' => 9,
                            'A' | 'a' => 10,
                            'B' | 'b' => 11,
                            'C' | 'c' => 12,
                            'D' | 'd' => 13,
                            'E' | 'e' => 14,
                            'F' | 'f' => 15,
                            _ => return Err(DxfError::ParseError),
                        };
                        if complete_byte {
                            let x = current_byte * 16 + value;
                            result.push(x);
                        }
                        else {
                            current_byte = value;
                        }
                        complete_byte = !complete_byte;
                    }
                }
                vba.data = result;
                vba._hex_data.clear();
            },
            _ => (),
        }

        Ok(())
    }
    fn apply_custom_reader<I>(&mut self, iter: &mut PutBack<I>) -> DxfResult<bool>
        where I: Iterator<Item = DxfResult<CodePair>> {

        match self.specific {
            ObjectType::DataTable(ref mut data) => {
                let mut read_column_count = false;
                let mut read_row_count = false;
                let mut _current_column_code = 0;
                let mut current_column = 0;
                let mut current_row = 0;
                let mut created_table = false;
                let mut current_2d_point = Point::origin();
                let mut current_3d_point = Point::origin();

                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { data.name = try!(pair.value.assert_string()); },
                        70 => { data.field = try!(pair.value.assert_i16()); },
                        90 => {
                            data.column_count = try!(pair.value.assert_i32()) as usize;
                            read_column_count = true;
                        },
                        91 => {
                            data.row_count = try!(pair.value.assert_i32()) as usize;
                            read_row_count = true;
                        },

                        // column headers
                        2 => { data.column_names.push(try!(pair.value.assert_string())); },
                        92 => {
                            _current_column_code = try!(pair.value.assert_i32());
                            current_column += 1;
                            current_row = 0;
                        },

                        // column values
                        3 => { data.set_value(current_row, current_column, DataTableValue::Str(try!(pair.value.assert_string()))); },
                        40 => { data.set_value(current_row, current_column, DataTableValue::Double(try!(pair.value.assert_f64()))); },
                        71 => { data.set_value(current_row, current_column, DataTableValue::Boolean(as_bool(try!(pair.value.assert_i16())))); },
                        93 => { data.set_value(current_row, current_column, DataTableValue::Integer(try!(pair.value.assert_i32()))); },
                        10 => { current_2d_point.x = try!(pair.value.assert_f64()); },
                        20 => { current_2d_point.y = try!(pair.value.assert_f64()); },
                        30 => {
                            current_2d_point.z = try!(pair.value.assert_f64());
                            data.set_value(current_row, current_column, DataTableValue::Point2D(current_2d_point.clone()));
                            current_2d_point = Point::origin();
                        },
                        11 => { current_3d_point.x = try!(pair.value.assert_f64()); },
                        21 => { current_3d_point.y = try!(pair.value.assert_f64()); },
                        31 => {
                            current_3d_point.z = try!(pair.value.assert_f64());
                            data.set_value(current_row, current_column, DataTableValue::Point3D(current_3d_point.clone()));
                            current_3d_point = Point::origin();
                        },
                        330 | 331 | 340 | 350 | 360 => {
                            if read_row_count || read_column_count {
                                data.set_value(current_row, current_column, DataTableValue::Handle(try!(as_u32(try!(pair.value.assert_string())))));
                            }
                            else {
                                try!(self.common.apply_individual_pair(&pair));
                            }
                        }

                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }

                    if read_row_count && read_column_count && !created_table {
                        for row in 0..data.row_count {
                            data.values.push(vec![]);
                            for _ in 0..data.column_count {
                                data.values[row].push(None);
                            }
                        }
                        created_table = true;
                    }
                }
            },
            ObjectType::Dictionary(ref mut dict) => {
                let mut last_entry_name = String::new();
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        3 => { last_entry_name = try!(pair.value.assert_string()); },
                        280 => { dict.is_hard_owner = as_bool(try!(pair.value.assert_i16())); },
                        281 => { dict.duplicate_record_handling = try_result!(DictionaryDuplicateRecordHandling::from_i16(try!(pair.value.assert_i16()))); },
                        350 | 360 => {
                            let handle = try!(as_u32(try!(pair.value.assert_string())));
                            dict.value_handles.insert(last_entry_name.clone(), handle);
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::DictionaryWithDefault(ref mut dict) => {
                let mut last_entry_name = String::new();
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        3 => { last_entry_name = try!(pair.value.assert_string()); },
                        281 => { dict.duplicate_record_handling = try_result!(DictionaryDuplicateRecordHandling::from_i16(try!(pair.value.assert_i16()))); },
                        340 => { dict.default_handle = try!(as_u32(try!(pair.value.assert_string()))); },
                        350 | 360 => {
                            let handle = try!(as_u32(try!(pair.value.assert_string())));
                            dict.value_handles.insert(last_entry_name.clone(), handle);
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::Layout(ref mut layout) => {
                let mut is_reading_plot_settings = true;
                loop {
                    let pair = next_pair!(iter);
                    if is_reading_plot_settings {
                        if pair.code == 100 && try!(pair.value.assert_string()) == "AcDbLayout" {
                            is_reading_plot_settings = false;
                        }
                        else {
                            try!(self.common.apply_individual_pair(&pair));
                        }
                    }
                    else {
                        match pair.code {
                            1 => { layout.layout_name = try!(pair.value.assert_string()); },
                            10 => { layout.minimum_limits.x = try!(pair.value.assert_f64()); },
                            20 => { layout.minimum_limits.y = try!(pair.value.assert_f64()); },
                            11 => { layout.maximum_limits.x = try!(pair.value.assert_f64()); },
                            21 => { layout.maximum_limits.y = try!(pair.value.assert_f64()); },
                            12 => { layout.insertion_base_point.x = try!(pair.value.assert_f64()); },
                            22 => { layout.insertion_base_point.y = try!(pair.value.assert_f64()); },
                            32 => { layout.insertion_base_point.z = try!(pair.value.assert_f64()); },
                            13 => { layout.ucs_origin.x = try!(pair.value.assert_f64()); },
                            23 => { layout.ucs_origin.y = try!(pair.value.assert_f64()); },
                            33 => { layout.ucs_origin.z = try!(pair.value.assert_f64()); },
                            14 => { layout.minimum_extents.x = try!(pair.value.assert_f64()); },
                            24 => { layout.minimum_extents.y = try!(pair.value.assert_f64()); },
                            34 => { layout.minimum_extents.z = try!(pair.value.assert_f64()); },
                            15 => { layout.maximum_extents.x = try!(pair.value.assert_f64()); },
                            25 => { layout.maximum_extents.y = try!(pair.value.assert_f64()); },
                            35 => { layout.maximum_extents.z = try!(pair.value.assert_f64()); },
                            16 => { layout.ucs_x_axis.x = try!(pair.value.assert_f64()); },
                            26 => { layout.ucs_x_axis.y = try!(pair.value.assert_f64()); },
                            36 => { layout.ucs_x_axis.z = try!(pair.value.assert_f64()); },
                            17 => { layout.ucs_y_axis.x = try!(pair.value.assert_f64()); },
                            27 => { layout.ucs_y_axis.y = try!(pair.value.assert_f64()); },
                            37 => { layout.ucs_y_axis.z = try!(pair.value.assert_f64()); },
                            70 => { layout.layout_flags = try!(pair.value.assert_i16()) as i32; },
                            71 => { layout.tab_order = try!(pair.value.assert_i16()) as i32; },
                            76 => { layout.ucs_orthographic_type = try_result!(UcsOrthographicType::from_i16(try!(pair.value.assert_i16()))); },
                            146 => { layout.elevation = try!(pair.value.assert_f64()); },
                            330 => { layout.viewport = try!(as_u32(try!(pair.value.assert_string()))); },
                            345 => { layout.table_record = try!(as_u32(try!(pair.value.assert_string()))); },
                            346 => { layout.table_record_base = try!(as_u32(try!(pair.value.assert_string()))); },
                            _ => { try!(self.common.apply_individual_pair(&pair)); },
                        }
                    }
                }
            },
            ObjectType::LightList(ref mut ll) => {
                let mut read_version_number = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { }, // don't worry about the light's name; it'll be read from the light entity directly
                        5 => {
                            if read_version_number {
                                // pointer to a new light
                                ll.lights.push(try!(as_u32(try!(pair.value.assert_string()))));
                            }
                            else {
                                // might still be the handle
                                try!(self.common.apply_individual_pair(&pair));;
                            }
                        },
                        90 => {
                            if read_version_number {
                                // count of lights is ignored since it's implicitly set by reading the values
                            }
                            else {
                                ll.version = try!(pair.value.assert_i32());
                                read_version_number = false;
                            }
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::Material(ref mut mat) => {
                let mut read_diffuse_map_file_name = false;
                let mut is_reading_normal = false;
                let mut read_diffuse_map_blend_factor = false;
                let mut read_image_file_diffuse_map = false;
                let mut read_diffuse_map_projection_method = false;
                let mut read_diffuse_map_tiling_method = false;
                let mut read_diffuse_map_auto_transform_method = false;
                let mut read_ambient_color_value = false;
                let mut read_bump_map_projection_method = false;
                let mut read_luminance_mode = false;
                let mut read_bump_map_tiling_method = false;
                let mut read_normal_map_method = false;
                let mut read_bump_map_auto_transform_method = false;
                let mut read_use_image_file_for_refraction_map = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { mat.name = try!(pair.value.assert_string()); },
                        2 => { mat.description = try!(pair.value.assert_string()); },
                        3 => {
                            if !read_diffuse_map_file_name {
                                mat.diffuse_map_file_name = try!(pair.value.assert_string());
                                read_diffuse_map_file_name = true;
                            }
                            else {
                                mat.normal_map_file_name = try!(pair.value.assert_string());
                                is_reading_normal = true;
                            }
                        },
                        4 => { mat.normal_map_file_name = try!(pair.value.assert_string()); },
                        6 => { mat.reflection_map_file_name = try!(pair.value.assert_string()); },
                        7 => { mat.opacity_map_file_name = try!(pair.value.assert_string()); },
                        8 => { mat.bump_map_file_name = try!(pair.value.assert_string()); },
                        9 => { mat.refraction_map_file_name = try!(pair.value.assert_string()); },
                        40 => { mat.ambient_color_factor = try!(pair.value.assert_f64()); },
                        41 => { mat.diffuse_color_factor = try!(pair.value.assert_f64()); },
                        42 => {
                            if !read_diffuse_map_blend_factor {
                                mat.diffuse_map_blend_factor = try!(pair.value.assert_f64());
                                read_diffuse_map_blend_factor = true;
                            }
                            else {
                                mat.normal_map_blend_factor = try!(pair.value.assert_f64());
                                is_reading_normal = true;
                            }
                        },
                        43 => {
                            if is_reading_normal {
                                mat._normal_map_transformation_matrix_values.push(try!(pair.value.assert_f64()));
                            }
                            else {
                                mat._diffuse_map_transformation_matrix_values.push(try!(pair.value.assert_f64()));
                            }
                        },
                        44 => { mat.specular_gloss_factor = try!(pair.value.assert_f64()); },
                        45 => { mat.specular_color_factor = try!(pair.value.assert_f64()); },
                        46 => { mat.specular_map_blend_factor = try!(pair.value.assert_f64()); },
                        47 => { mat._specular_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        48 => { mat.reflection_map_blend_factor = try!(pair.value.assert_f64()); },
                        49 => { mat._reflection_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        62 => { mat.gen_proc_color_index_value = Color::from_raw_value(try!(pair.value.assert_i16())); },
                        70 => { mat.override_ambient_color = as_bool(try!(pair.value.assert_i16())); },
                        71 => { mat.override_diffuse_color = as_bool(try!(pair.value.assert_i16())); },
                        72 => {
                            if !read_image_file_diffuse_map {
                                mat.use_image_file_for_diffuse_map = as_bool(try!(pair.value.assert_i16()));
                                read_image_file_diffuse_map = true;
                            }
                            else {
                                mat.use_image_file_for_normal_map = as_bool(try!(pair.value.assert_i16()));
                            }
                        },
                        73 => {
                            if !read_diffuse_map_projection_method {
                                mat.diffuse_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16())));
                                read_diffuse_map_projection_method = true;
                            }
                            else {
                                mat.normal_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16())));
                                is_reading_normal = true;
                            }
                        },
                        74 => {
                            if !read_diffuse_map_tiling_method {
                                mat.diffuse_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16())));
                                read_diffuse_map_tiling_method = true;
                            }
                            else {
                                mat.normal_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16())));
                                is_reading_normal = true;
                            }
                        },
                        75 => {
                            if !read_diffuse_map_auto_transform_method {
                                mat.diffuse_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16())));
                                read_diffuse_map_auto_transform_method = true;
                            }
                            else {
                                mat.normal_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16())));
                                is_reading_normal = true;
                            }
                        },
                        76 => { mat.override_specular_color = as_bool(try!(pair.value.assert_i16())); },
                        77 => { mat.use_image_file_for_specular_map = as_bool(try!(pair.value.assert_i16())); },
                        78 => { mat.specular_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        79 => { mat.specular_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        90 => {
                            if !read_ambient_color_value {
                                mat.ambient_color_value = try!(pair.value.assert_i32());
                                read_ambient_color_value = true;
                            }
                            else {
                                mat.self_illumination = try!(pair.value.assert_i32());
                            }
                        },
                        91 => { mat.diffuse_color_value = try!(pair.value.assert_i32()); },
                        92 => { mat.specular_color_value = try!(pair.value.assert_i32()); },
                        93 => { mat.illumination_model = try!(pair.value.assert_i32()); },
                        94 => { mat.channel_flags = try!(pair.value.assert_i32()); },
                        140 => { mat.opacity_factor = try!(pair.value.assert_f64()); },
                        141 => { mat.opacity_map_blend_factor = try!(pair.value.assert_f64()); },
                        142 => { mat._opacity_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        143 => { mat.bump_map_blend_factor = try!(pair.value.assert_f64()); },
                        144 => { mat._bump_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        145 => { mat.refraction_index = try!(pair.value.assert_f64()); },
                        146 => { mat.refraction_map_blend_factor = try!(pair.value.assert_f64()); },
                        147 => { mat._refraction_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        148 => { mat.translucence = try!(pair.value.assert_f64()); },
                        170 => { mat.specular_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        171 => { mat.use_image_file_for_reflection_map = as_bool(try!(pair.value.assert_i16())); },
                        172 => { mat.reflection_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        173 => { mat.reflection_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        174 => { mat.reflection_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        175 => { mat.use_image_file_for_opacity_map = as_bool(try!(pair.value.assert_i16())); },
                        176 => { mat.opacity_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        177 => { mat.opacity_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        178 => { mat.opacity_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        179 => { mat.use_image_file_for_bump_map = as_bool(try!(pair.value.assert_i16())); },
                        270 => {
                            if !read_bump_map_projection_method {
                                mat.bump_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16())));
                                read_bump_map_projection_method = true;
                            }
                            else if !read_luminance_mode {
                                mat.luminance_mode = try!(pair.value.assert_i16());
                                read_luminance_mode = true;
                            }
                            else {
                                mat.map_u_tile = try!(pair.value.assert_i16());
                            }
                        },
                        271 => {
                            if !read_bump_map_tiling_method {
                                mat.bump_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16())));
                                read_bump_map_tiling_method = true;
                            }
                            else if !read_normal_map_method {
                                mat.normal_map_method = try!(pair.value.assert_i16());
                                read_normal_map_method = true;
                            }
                            else {
                                mat.gen_proc_integer_value = try!(pair.value.assert_i16());
                            }
                        },
                        272 => {
                            if !read_bump_map_auto_transform_method {
                                mat.bump_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16())));
                                read_bump_map_auto_transform_method = true;
                            }
                            else {
                                mat.global_illumination_mode = try!(pair.value.assert_i16());
                            }
                        },
                        273 => {
                            if !read_use_image_file_for_refraction_map {
                                mat.use_image_file_for_refraction_map = as_bool(try!(pair.value.assert_i16()));
                                read_use_image_file_for_refraction_map = true;
                            }
                            else {
                                mat.final_gather_mode = try!(pair.value.assert_i16());
                            }
                        },
                        274 => { mat.refraction_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        275 => { mat.refraction_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        276 => { mat.refraction_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        290 => { mat.is_two_sided = try!(pair.value.assert_bool()); },
                        291 => { mat.gen_proc_boolean_value = try!(pair.value.assert_bool()); },
                        292 => { mat.gen_proc_table_end = try!(pair.value.assert_bool()); },
                        293 => { mat.is_anonymous = try!(pair.value.assert_bool()); },
                        300 => { mat.gen_proc_name = try!(pair.value.assert_string()); },
                        301 => { mat.gen_proc_text_value = try!(pair.value.assert_string()); },
                        420 => { mat.gen_proc_color_rgb_value = try!(pair.value.assert_i32()); },
                        430 => { mat.gen_proc_color_name = try!(pair.value.assert_string()); },
                        460 => { mat.color_bleed_scale = try!(pair.value.assert_f64()); },
                        461 => { mat.indirect_dump_scale = try!(pair.value.assert_f64()); },
                        462 => { mat.reflectance_scale = try!(pair.value.assert_f64()); },
                        463 => { mat.transmittance_scale = try!(pair.value.assert_f64()); },
                        464 => { mat.luminance = try!(pair.value.assert_f64()); },
                        465 => {
                            mat.normal_map_strength = try!(pair.value.assert_f64());
                            is_reading_normal = true;
                        },
                        468 => { mat.reflectivity = try!(pair.value.assert_f64()); },
                        469 => { mat.gen_proc_real_value = try!(pair.value.assert_f64()); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::MLineStyle(ref mut mline) => {
                let mut read_element_count = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        2 => { mline.style_name = try!(pair.value.assert_string()); },
                        3 => { mline.description = try!(pair.value.assert_string()); },
                        6 => { mline._element_linetypes.push(try!(pair.value.assert_string())); },
                        49 => { mline._element_offsets.push(try!(pair.value.assert_f64())); },
                        51 => { mline.start_angle = try!(pair.value.assert_f64()); },
                        52 => { mline.end_angle = try!(pair.value.assert_f64()); },
                        62 => {
                            if read_element_count {
                                mline._element_colors.push(Color::from_raw_value(try!(pair.value.assert_i16())));
                            }
                            else {
                                mline.fill_color = Color::from_raw_value(try!(pair.value.assert_i16()));
                            }
                        },
                        70 => { mline._flags = try!(pair.value.assert_i16()) as i32; },
                        71 => {
                            mline._element_count = try!(pair.value.assert_i16()) as i32;
                            read_element_count = true;
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::SectionSettings(ref mut ss) => {
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => {
                            // value should be "SectionTypeSettings", but it doesn't realy matter
                            loop {
                                match try!(SectionTypeSettings::read(iter)) {
                                    Some(ts) => ss.geometry_settings.push(ts),
                                    None => break,
                                }
                            }
                        },
                        90 => { ss.section_type = try!(pair.value.assert_i32()); }
                        91 => (), // generation settings count; we just read as many as we're given
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::SortentsTable(ref mut sort) => {
                let mut is_ready_for_sort_handles = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        5 => {
                            if is_ready_for_sort_handles {
                                sort.sort_items.push(try!(as_u32(try!(pair.value.assert_string()))));
                            }
                            else {
                                self.common.handle = try!(as_u32(try!(pair.value.assert_string())));
                                is_ready_for_sort_handles = true;
                            }
                        },
                        100 => { is_ready_for_sort_handles = true; },
                        330 => {
                            self.common.owner_handle = try!(as_u32(try!(pair.value.assert_string())));
                            is_ready_for_sort_handles = true;
                        },
                        331 => {
                            sort.entities.push(try!(as_u32(try!(pair.value.assert_string()))));
                            is_ready_for_sort_handles = true;
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::SpatialFilter(ref mut sf) => {
                let mut read_front_clipping_plane = false;
                let mut set_inverse_matrix = false;
                let mut matrix_list = vec![];
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        10 => {
                            // code 10 always starts a new point
                            sf.clip_boundary_definition_points.push(Point::origin());
                            vec_last!(sf.clip_boundary_definition_points).x = try!(pair.value.assert_f64());
                        },
                        20 => { vec_last!(sf.clip_boundary_definition_points).y = try!(pair.value.assert_f64()); },
                        30 => { vec_last!(sf.clip_boundary_definition_points).z = try!(pair.value.assert_f64()); },
                        11 => { sf.clip_boundary_origin.x = try!(pair.value.assert_f64()); },
                        21 => { sf.clip_boundary_origin.y = try!(pair.value.assert_f64()); },
                        31 => { sf.clip_boundary_origin.z = try!(pair.value.assert_f64()); },
                        40 => {
                            if !read_front_clipping_plane {
                                sf.front_clipping_plane_distance = try!(pair.value.assert_f64());
                                read_front_clipping_plane = true;
                            }
                            else {
                                matrix_list.push(try!(pair.value.assert_f64()));
                                if matrix_list.len() == 12 {
                                    let mut matrix = TransformationMatrix::default();
                                    matrix.from_vec(&vec![
                                        matrix_list[0], matrix_list[1], matrix_list[2], 0.0,
                                        matrix_list[3], matrix_list[4], matrix_list[5], 0.0,
                                        matrix_list[6], matrix_list[7], matrix_list[8], 0.0,
                                        matrix_list[9], matrix_list[10], matrix_list[11], 0.0,
                                    ]);
                                    matrix_list.clear();
                                    if !set_inverse_matrix {
                                        sf.inverse_transformation_matrix = matrix;
                                        set_inverse_matrix = true;
                                    }
                                    else {
                                        sf.transformation_matrix = matrix;
                                    }
                                }
                            }
                        },
                        41 => { sf.back_clipping_plane_distance = try!(pair.value.assert_f64()); },
                        70 => (), // boundary point count; we just read as many as we're given
                        71 => { sf.is_clip_boundary_enabled = as_bool(try!(pair.value.assert_i16())); },
                        72 => { sf.is_front_clipping_plane = as_bool(try!(pair.value.assert_i16())); },
                        73 => { sf.is_back_clipping_plane = as_bool(try!(pair.value.assert_i16())); },
                        210 => { sf.clip_boundary_normal.x = try!(pair.value.assert_f64()); },
                        220 => { sf.clip_boundary_normal.y = try!(pair.value.assert_f64()); },
                        230 => { sf.clip_boundary_normal.z = try!(pair.value.assert_f64()); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::SunStudy(ref mut ss) => {
                let mut seen_version = false;
                let mut reading_hours = false;
                let mut julian_day = None;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { ss.sun_setup_name = try!(pair.value.assert_string()); },
                        2 => { ss.description = try!(pair.value.assert_string()); },
                        3 => { ss.sheet_set_name = try!(pair.value.assert_string()); },
                        4 => { ss.sheet_subset_name = try!(pair.value.assert_string()); },
                        40 => { ss.spacing = try!(pair.value.assert_f64()); },
                        70 => { ss.output_type = try!(pair.value.assert_i16()); },
                        73 => { reading_hours = true; },
                        74 => { ss.shade_plot_type = try!(pair.value.assert_i16()); },
                        75 => { ss.viewports_per_page = try!(pair.value.assert_i16()) as i32; },
                        76 => { ss.viewport_distribution_row_count = try!(pair.value.assert_i16()) as i32; },
                        77 => { ss.viewport_distribution_column_count = try!(pair.value.assert_i16()) as i32; },
                        90 => {
                            if !seen_version {
                                ss.version = try!(pair.value.assert_i32());
                                seen_version = true;
                            }
                            else {
                                // after the version, 90 pairs come in julian_day/seconds_past_midnight duals
                                match julian_day {
                                    Some(jd) => {
                                        let date = as_datetime_local(jd as f64);
                                        let date = date.add(Duration::seconds(try!(pair.value.assert_i32()) as i64));
                                        ss.dates.push(date);
                                        julian_day = None;
                                    },
                                    None => {
                                        julian_day = Some(try!(pair.value.assert_i32()));
                                    },
                                }
                            }
                        },
                        93 => { ss.start_time_seconds_past_midnight = try!(pair.value.assert_i32()); },
                        94 => { ss.end_time_seconds_past_midnight = try!(pair.value.assert_i32()); },
                        95 => { ss.interval_in_seconds = try!(pair.value.assert_i32()); },
                        290 => {
                            if !reading_hours {
                                ss.use_subset = try!(pair.value.assert_bool());
                                reading_hours = true;
                            }
                            else {
                                ss.hours.push(try!(pair.value.assert_i16()) as i32);
                            }
                        },
                        291 => { ss.select_dates_from_calendar = try!(pair.value.assert_bool()); },
                        292 => { ss.select_range_of_dates = try!(pair.value.assert_bool()); },
                        293 => { ss.lock_viewports = try!(pair.value.assert_bool()); },
                        294 => { ss.label_viewports = try!(pair.value.assert_bool()); },
                        340 => { ss.page_setup_wizard = try!(as_u32(try!(pair.value.assert_string()))); },
                        341 => { ss.view = try!(as_u32(try!(pair.value.assert_string()))); },
                        342 => { ss.visual_style = try!(as_u32(try!(pair.value.assert_string()))); },
                        343 => { ss.text_style = try!(as_u32(try!(pair.value.assert_string()))); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::TableStyle(ref mut ts) => {
                let mut read_version = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        3 => { ts.description = try!(pair.value.assert_string()); },
                        7 => {
                            iter.put_back(Ok(pair)); // let the TableCellStyle reader parse this
                            if let Some(style) = try!(TableCellStyle::read(iter)) {
                                ts.cell_styles.push(style);
                            }
                        },
                        40 => { ts.horizontal_cell_margin = try!(pair.value.assert_f64()); },
                        41 => { ts.vertical_cell_margin = try!(pair.value.assert_f64()); },
                        70 => { ts.flow_direction = try_result!(FlowDirection::from_i16(try!(pair.value.assert_i16()))); },
                        71 => { ts.flags = try!(pair.value.assert_i16()) as i32; },
                        280 => {
                            if !read_version {
                                ts.version = try_result!(Version::from_i16(try!(pair.value.assert_i16())));
                                read_version = true;
                            }
                            else {
                                ts.is_title_suppressed = as_bool(try!(pair.value.assert_i16()));
                            }
                        },
                        281 => { ts.is_column_heading_suppressed = as_bool(try!(pair.value.assert_i16())); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            ObjectType::XRecordObject(ref mut xr) => {
                let mut reading_data = false;
                loop {
                    let pair = next_pair!(iter);
                    if reading_data {
                        xr.data_pairs.push(pair);
                    }
                    else {
                        if pair.code == 280 {
                            xr.duplicate_record_handling = try_result!(DictionaryDuplicateRecordHandling::from_i16(try!(pair.value.assert_i16())));
                            reading_data = true;
                            continue;
                        }

                        if try!(self.common.apply_individual_pair(&pair)) {
                            continue;
                        }

                        match pair.code {
                            100 => { continue; }, // value should be "AcDbXrecord", but it doesn't really matter
                            5 | 105 => (), // these codes aren't allowed here
                            _ => {
                                xr.data_pairs.push(pair);
                                reading_data = true;
                            },
                        }
                    }
                }
            },
            _ => return Ok(false), // no custom reader
        }

        Ok(true)
    }
    #[doc(hidden)]
    pub fn write<T>(&self, version: &AcadVersion, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        if self.specific.is_supported_on_version(version) {
            try!(writer.write_code_pair(&CodePair::new_str(0, self.specific.to_type_string())));
            try!(self.common.write(writer));
            if !try!(self.apply_custom_writer(version, writer)) {
                try!(self.specific.write(version, writer));
                try!(self.post_write(&version, writer));
            }
        }

        Ok(())
    }
    fn apply_custom_writer<T>(&self, version: &AcadVersion, writer: &mut CodePairWriter<T>) -> DxfResult<bool>
        where T: Write {

        match self.specific {
            ObjectType::DataTable(ref data) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbDataTable")));
                try!(writer.write_code_pair(&CodePair::new_i16(70, data.field)));
                try!(writer.write_code_pair(&CodePair::new_i32(90, data.column_count as i32)));
                try!(writer.write_code_pair(&CodePair::new_i32(91, data.row_count as i32)));
                try!(writer.write_code_pair(&CodePair::new_string(1, &data.name)));
                for col in 0..data.column_count {
                    let column_code = match &data.values[0][col] {
                        &Some(DataTableValue::Boolean(_)) => Some(71),
                        &Some(DataTableValue::Integer(_)) => Some(93),
                        &Some(DataTableValue::Double(_)) => Some(40),
                        &Some(DataTableValue::Str(_)) => Some(3),
                        &Some(DataTableValue::Point2D(_)) => Some(10),
                        &Some(DataTableValue::Point3D(_)) => Some(11),
                        &Some(DataTableValue::Handle(_)) => Some(331),
                        &None => None,
                    };
                    if let Some(column_code) = column_code {
                        try!(writer.write_code_pair(&CodePair::new_i32(92, column_code)));
                        try!(writer.write_code_pair(&CodePair::new_string(2, &data.column_names[col])));
                        for row in 0..data.row_count {
                            match &data.values[row][col] {
                                &Some(DataTableValue::Boolean(val)) => { try!(writer.write_code_pair(&CodePair::new_i16(71, as_i16(val)))); },
                                &Some(DataTableValue::Integer(val)) => { try!(writer.write_code_pair(&CodePair::new_i32(93, val))); },
                                &Some(DataTableValue::Double(val)) => { try!(writer.write_code_pair(&CodePair::new_f64(40, val))); },
                                &Some(DataTableValue::Str(ref val)) => { try!(writer.write_code_pair(&CodePair::new_string(3, val))); },
                                &Some(DataTableValue::Point2D(ref val)) => {
                                    try!(writer.write_code_pair(&CodePair::new_f64(10, val.x)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(20, val.y)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(30, val.z)));
                                },
                                &Some(DataTableValue::Point3D(ref val)) => {
                                    try!(writer.write_code_pair(&CodePair::new_f64(11, val.x)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(21, val.y)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(31, val.z)));
                                },
                                &Some(DataTableValue::Handle(val)) => { try!(writer.write_code_pair(&CodePair::new_string(331, &as_handle(val)))); },
                                &None => (),
                            }
                        }
                    }
                }
            },
            ObjectType::Dictionary(ref dict) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbDictionary")));
                if *version >= AcadVersion::R2000 && !dict.is_hard_owner {
                    try!(writer.write_code_pair(&CodePair::new_i16(280, as_i16(dict.is_hard_owner))));
                }
                if *version >= AcadVersion::R2000 {
                    try!(writer.write_code_pair(&CodePair::new_i16(281, dict.duplicate_record_handling as i16)));
                }
                let code = if dict.is_hard_owner { 360 } else { 350 };
                for key in dict.value_handles.keys().sorted_by(|a, b| Ord::cmp(a, b)) {
                    if let Some(value) = dict.value_handles.get(key) {
                        try!(writer.write_code_pair(&CodePair::new_string(3, key)));
                        try!(writer.write_code_pair(&CodePair::new_string(code, &as_handle(*value))));
                    }
                }
            },
            ObjectType::DictionaryWithDefault(ref dict) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbDictionary")));
                if *version >= AcadVersion::R2000 {
                    try!(writer.write_code_pair(&CodePair::new_i16(281, dict.duplicate_record_handling as i16)));
                }
                try!(writer.write_code_pair(&CodePair::new_string(340, &as_handle(dict.default_handle))));
                for key in dict.value_handles.keys().sorted_by(|a, b| Ord::cmp(a, b)) {
                    if let Some(value) = dict.value_handles.get(key) {
                        try!(writer.write_code_pair(&CodePair::new_string(3, key)));
                        try!(writer.write_code_pair(&CodePair::new_string(350, &as_handle(*value))));
                    }
                }
            },
            ObjectType::LightList(ref ll) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbLightList")));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ll.version)));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ll.lights.len() as i32)));
                for light in &ll.lights {
                    try!(writer.write_code_pair(&CodePair::new_string(5, &as_handle(*light))));
                    try!(writer.write_code_pair(&CodePair::new_string(1, &String::new()))); // TODO: write the light's real name
                }
            },
            ObjectType::SectionSettings(ref ss) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbSectionSettings")));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ss.section_type)));
                try!(writer.write_code_pair(&CodePair::new_i32(91, ss.geometry_settings.len() as i32)));
                for settings in &ss.geometry_settings {
                    try!(settings.write(writer));
                }
            },
            ObjectType::SunStudy(ref ss) => {
                try!(writer.write_code_pair(&CodePair::new_string(100, &String::from("AcDbSunStudy"))));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ss.version)));
                try!(writer.write_code_pair(&CodePair::new_string(1, &ss.sun_setup_name)));
                try!(writer.write_code_pair(&CodePair::new_string(2, &ss.description)));
                try!(writer.write_code_pair(&CodePair::new_i16(70, ss.output_type)));
                try!(writer.write_code_pair(&CodePair::new_string(3, &ss.sheet_set_name)));
                try!(writer.write_code_pair(&CodePair::new_bool(290, ss.use_subset)));
                try!(writer.write_code_pair(&CodePair::new_string(4, &ss.sheet_subset_name)));
                try!(writer.write_code_pair(&CodePair::new_bool(291, ss.select_dates_from_calendar)));
                try!(writer.write_code_pair(&CodePair::new_i32(91, ss.dates.len() as i32)));
                for item in &ss.dates {
                    try!(writer.write_code_pair(&CodePair::new_i32(90, as_double_local(*item) as i32)));
                }
                try!(writer.write_code_pair(&CodePair::new_bool(292, ss.select_range_of_dates)));
                try!(writer.write_code_pair(&CodePair::new_i32(93, ss.start_time_seconds_past_midnight)));
                try!(writer.write_code_pair(&CodePair::new_i32(94, ss.end_time_seconds_past_midnight)));
                try!(writer.write_code_pair(&CodePair::new_i32(95, ss.interval_in_seconds)));
                try!(writer.write_code_pair(&CodePair::new_i16(73, ss.hours.len() as i16)));
                for v in &ss.hours {
                    try!(writer.write_code_pair(&CodePair::new_i16(290, *v as i16)));
                }
                try!(writer.write_code_pair(&CodePair::new_string(340, &as_handle(ss.page_setup_wizard))));
                try!(writer.write_code_pair(&CodePair::new_string(341, &as_handle(ss.view))));
                try!(writer.write_code_pair(&CodePair::new_string(342, &as_handle(ss.visual_style))));
                try!(writer.write_code_pair(&CodePair::new_i16(74, ss.shade_plot_type)));
                try!(writer.write_code_pair(&CodePair::new_i16(75, ss.viewports_per_page as i16)));
                try!(writer.write_code_pair(&CodePair::new_i16(76, ss.viewport_distribution_row_count as i16)));
                try!(writer.write_code_pair(&CodePair::new_i16(77, ss.viewport_distribution_column_count as i16)));
                try!(writer.write_code_pair(&CodePair::new_f64(40, ss.spacing)));
                try!(writer.write_code_pair(&CodePair::new_bool(293, ss.lock_viewports)));
                try!(writer.write_code_pair(&CodePair::new_bool(294, ss.label_viewports)));
                try!(writer.write_code_pair(&CodePair::new_string(343, &as_handle(ss.text_style))));
            },
            ObjectType::XRecordObject(ref xr) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbXrecord")));
                try!(writer.write_code_pair(&CodePair::new_i16(280, xr.duplicate_record_handling as i16)));
                for pair in &xr.data_pairs {
                    try!(writer.write_code_pair(&pair));
                }
            },
            _ => return Ok(false), // no custom writer
        }

        Ok(true)
    }
    fn post_write<T>(&self, _version: &AcadVersion, _writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        match self.specific {
            _ => (),
        }

        Ok(())
    }
}

impl DataTable {
    #[doc(hidden)]
    pub fn set_value(&mut self, row: usize, col: usize, val: DataTableValue) {
        if row <= self.row_count && col <= self.column_count {
            self.values[row][col] = Some(val);
        }
    }
}

impl VbaProject {
    #[doc(hidden)]
    pub fn get_hex_strings(&self) -> DxfResult<Vec<String>> {
        let mut result = vec![];
        for s in self.data.chunks(128) {
            let mut line = String::new();
            for b in s {
                line.push_str(&format!("{:X}", b));
            }
            result.push(line);
        }

        Ok(result)
    }
}

//------------------------------------------------------------------------------
//                                                                    EntityIter
//------------------------------------------------------------------------------
struct EntityIter<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> {
    iter: &'a mut PutBack<I>,
}

impl<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> Iterator for EntityIter<'a, I> {
    type Item = Entity;
    fn next(&mut self) -> Option<Entity> {
        match Entity::read(self.iter) {
            Ok(Some(e)) => Some(e),
            Ok(None) | Err(_) => None,
        }
    }
}

//------------------------------------------------------------------------------
//                                                                    ObjectIter
//------------------------------------------------------------------------------
struct ObjectIter<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> {
    iter: &'a mut PutBack<I>,
}

impl<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> Iterator for ObjectIter<'a, I> {
    type Item = Object;
    fn next(&mut self) -> Option<Object> {
        match Object::read(self.iter) {
            Ok(Some(o)) => Some(o),
            Ok(None) | Err(_) => None,
        }
    }
}

//------------------------------------------------------------------------------
//                                                                         Point
//------------------------------------------------------------------------------
/// Represents a simple point in Cartesian space.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    /// The X value of the point.
    pub x: f64,
    /// The Y value of the point.
    pub y: f64,
    /// The Z value of the point.
    pub z: f64,
}

impl Point {
    /// Creates a new `Point` with the specified values.
    pub fn new(x: f64, y: f64, z: f64) -> Point {
        Point{
            x: x,
            y: y,
            z: z,
        }
    }
    /// Returns a point representing the origin of (0, 0, 0).
    pub fn origin() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }
    #[doc(hidden)]
    pub fn set(&mut self, pair: &CodePair) -> DxfResult<()> {
        match pair.code {
            10 => self.x = try!(pair.value.assert_f64()),
            20 => self.y = try!(pair.value.assert_f64()),
            30 => self.z = try!(pair.value.assert_f64()),
            _ => return Err(DxfError::UnexpectedCodePair(pair.clone(), String::from("expected code [10, 20, 30] for point"))),
        }

        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                                        Vector
//------------------------------------------------------------------------------
/// Represents a simple vector in Cartesian space.
#[derive(Clone, Debug, PartialEq)]
pub struct Vector {
    /// The X component of the vector.
    pub x: f64,
    /// The Y component of the vector.
    pub y: f64,
    /// The Z component of the vector.
    pub z: f64,
}

impl Vector {
    /// Creates a new `Vector` with the specified values.
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector {
            x: x,
            y: y,
            z: z,
        }
    }
    /// Returns a new zero vector representing (0, 0, 0).
    pub fn zero() -> Vector {
        Vector::new(0.0, 0.0, 0.0)
    }
    /// Returns a new vector representing the X axis.
    pub fn x_axis() -> Vector {
        Vector::new(1.0, 0.0, 0.0)
    }
    /// Returns a new vector representing the Y axis.
    pub fn y_axis() -> Vector {
        Vector::new(0.0, 1.0, 0.0)
    }
    /// Returns a new vector representing the Z axis.
    pub fn z_axis() -> Vector {
        Vector::new(0.0, 0.0, 1.0)
    }
    #[doc(hidden)]
    pub fn set(&mut self, pair: &CodePair) -> DxfResult<()> {
        match pair.code {
            10 => self.x = try!(pair.value.assert_f64()),
            20 => self.y = try!(pair.value.assert_f64()),
            30 => self.z = try!(pair.value.assert_f64()),
            _ => return Err(DxfError::UnexpectedCodePair(pair.clone(), String::from("expected code [10, 20, 30] for vector"))),
        }

        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                              LwPolylineVertex
//------------------------------------------------------------------------------
/// Represents a single vertex of a `LwPolyline`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LwPolylineVertex {
    pub x: f64,
    pub y: f64,
    pub id: i32,
    pub starting_width: f64,
    pub ending_width: f64,
    pub bulge: f64,
}

impl LwPolylineVertex {
    pub fn new() -> Self {
        Default::default()
    }
}

//------------------------------------------------------------------------------
//                                                                         Color
//------------------------------------------------------------------------------
/// Represents an indexed color.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Color {
    raw_value: i16,
}

impl Color {
    /// Returns `true` if the color defaults back to the item's layer's color.
    pub fn is_by_layer(&self) -> bool {
        self.raw_value == 256
    }
    /// Returns `true` if the color defaults back to the entity's color.
    pub fn is_by_entity(&self) -> bool {
        self.raw_value == 257
    }
    /// Returns `true` if the color defaults back to the containing block's color.
    pub fn is_by_block(&self) -> bool {
        self.raw_value == 0
    }
    /// Returns `true` if the color represents a `Layer` that is turned off.
    pub fn is_turned_off(&self) -> bool {
        self.raw_value < 0
    }
    /// Sets the color to default back to the item's layer's color.
    pub fn set_by_layer(&mut self) {
        self.raw_value = 256
    }
    /// Sets the color to default back to the containing block's color.
    pub fn set_by_block(&mut self) {
        self.raw_value = 0
    }
    /// Sets the color to default back to the containing entity's color.
    pub fn set_by_entity(&mut self) {
        self.raw_value = 257
    }
    /// Sets the color to represent a `Layer` that is turned off.
    pub fn turn_off(&mut self) {
        self.raw_value = -1
    }
    /// Returns `true` if the color represents a proper color index.
    pub fn is_index(&self) -> bool {
        self.raw_value >= 1 && self.raw_value <= 255
    }
    /// Gets an `Option<u8>` of the indexable value of the color.
    pub fn index(&self) -> Option<u8> {
        if self.is_index() {
            Some(self.raw_value as u8)
        }
        else {
            None
        }
    }
    #[doc(hidden)]
    pub fn get_raw_value(&self) -> i16 {
        self.raw_value
    }
    #[doc(hidden)]
    pub fn from_raw_value(val: i16) -> Color {
        Color { raw_value: val }
    }
    /// Creates a `Color` that defaults to the item's layer's color.
    pub fn by_layer() -> Color {
        Color { raw_value: 256 }
    }
    /// Creates a `Color` that defaults back to the containing block's color.
    pub fn by_block() -> Color {
        Color { raw_value: 0 }
    }
    /// Creates a `Color` that defaults back to the containing entity's color.
    pub fn by_entity() -> Color {
        Color { raw_value: 257 }
    }
    /// Creates a `Color` from the specified index.
    pub fn from_index(i: u8) -> Color {
        Color { raw_value: i as i16 }
    }
    #[doc(hidden)]
    pub fn get_writable_color_value(&self, layer: &Layer) -> i16 {
       let value = match self.get_raw_value().abs() {
            0 | 256 => 7i16, // BYLAYER and BYBLOCK aren't valid
            v => v,
        };
        let value = match layer.is_layer_on {
            true => value,
            false => -value,
        };

        value
    }
}

//------------------------------------------------------------------------------
//                                                                    LineWeight
//------------------------------------------------------------------------------
/// Represents a line weight.
pub struct LineWeight {
    raw_value: i16,
}

impl LineWeight {
    /// Creates a new `LineWeight`.
    pub fn new() -> LineWeight {
        LineWeight::from_raw_value(0)
    }
    #[doc(hidden)]
    pub fn from_raw_value(v: i16) -> LineWeight {
        LineWeight { raw_value: v }
    }
    /// Creates a new `LineWeight` that defaults back to the containing block's line weight.
    pub fn by_block() -> LineWeight {
        LineWeight::from_raw_value(-1)
    }
    /// Creates a new `LineWeight` that defaults back to the item's layer's line weight.
    pub fn by_layer() -> LineWeight {
        LineWeight::from_raw_value(-2)
    }
    #[doc(hidden)]
    pub fn get_raw_value(&self) -> i16 {
        self.raw_value
    }
}
