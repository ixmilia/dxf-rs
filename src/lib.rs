// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

//! This crate provides the ability to read and write DXF and DXB CAD files.
//!
//! # Usage
//!
//! Put this in your `Cargo.toml`:
//!
//! ``` toml
//! [dependencies]
//! dxf = "0.3.0"
//! ```
//!
//! Or if you want [serde](https://github.com/serde-rs/serde) support, enable the `serialize` feature:
//!
//! ``` toml
//! [dependencies]
//! dxf = { version = "0.3.0", features = ["serialize"] }
//! ```
//!
//! > Note that `serde` support is intended to aid in debugging and since the serialized format is heavily
//! dependent on the layout of the structures, it may change at any time.
//!
//! And finally add:
//!
//! ``` rust
//! extern crate dxf;
//! ```
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
//! let drawing = Drawing::load_file("path/to/file.dxf")?;
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
//! let mut drawing = Drawing::default();
//! drawing.entities.push(Entity::new(EntityType::Line(Line::default())));
//! drawing.save_file("path/to/file.dxf")?;
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

#[macro_use]
extern crate enum_primitive;

extern crate image;
extern crate itertools;

#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serialize")]
extern crate serde;

mod code_pair;
pub use code_pair::CodePair;

mod code_pair_value;
pub use code_pair_value::CodePairValue;

mod data_table_value;
pub use data_table_value::DataTableValue;

#[macro_use]
mod helper_functions;

mod dxb_item_type;
mod dxb_reader;
mod dxb_writer;

mod extension_data;
pub use extension_data::*;

mod x_data;
pub use x_data::*;

mod table;

mod handle_tracker;

mod drawing;
pub use drawing::Drawing;

mod drawing_item;
pub use drawing_item::{DrawingItem, DrawingItemMut};

mod section_geometry_settings;
pub use section_geometry_settings::SectionGeometrySettings;

mod section_type_settings;
pub use section_type_settings::SectionTypeSettings;

mod table_cell_style;
pub use table_cell_style::TableCellStyle;

mod transformation_matrix;
pub use transformation_matrix::TransformationMatrix;

pub mod enums;

mod color;
pub use color::Color;

mod point;
pub use point::Point;

mod vector;
pub use vector::Vector;

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

include!("expected_type.rs");

mod code_pair_iter;
mod code_pair_writer;

mod block;
pub use block::Block;

mod class;
pub use class::Class;

mod header;
pub use header::Header;

mod line_weight;
pub use line_weight::LineWeight;

mod entity;
pub use entity::LwPolylineVertex;

mod object;
pub use object::{GeoMeshPoint, MLineStyleElement};

mod dxf_error;
pub use dxf_error::DxfError;

mod dxf_result;
pub use dxf_result::DxfResult;

mod entity_iter;
mod object_iter;
