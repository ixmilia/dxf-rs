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

mod code_pair;
pub use code_pair::CodePair;

mod code_pair_value;
pub use code_pair_value::CodePairValue;

mod data_table_value;
pub use data_table_value::DataTableValue;

mod drawing;
pub use drawing::Drawing;

mod section_geometry_settings;
pub use section_geometry_settings::SectionGeometrySettings;

mod section_type_settings;
pub use section_type_settings::SectionTypeSettings;

mod table_cell_style;
pub use table_cell_style::TableCellStyle;

mod transformation_matrix;
pub use transformation_matrix::TransformationMatrix;

pub mod enums;

mod generated;
pub mod entities {
    pub use generated::entities::*;
}
pub mod tables {
    pub use generated::tables::*;
}
pub mod objects {
    pub use generated::objects::*;
}

use entities::*;
use tables::*;
use objects::*;

use itertools::PutBack;

include!("expected_type.rs");

mod code_pair_iter;
mod code_pair_writer;

mod block;
pub use block::Block;

mod class;
pub use class::Class;

mod helper_functions;

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

mod header;

mod entity;
pub use entity::LwPolylineVertex;

mod object;
pub use object::{
    GeoMeshPoint,
    MLineStyleElement,
};

mod dxf_error;
pub use dxf_error::DxfError;

mod dxf_result;
pub use dxf_result::DxfResult;

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
