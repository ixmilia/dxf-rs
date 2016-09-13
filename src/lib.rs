// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

//! This crate provides the ability to read and write DXF CAD files.
//!
//! # Examples
//!
//! Open a DXF file from disk:
//!
//! ``` rust
//! # fn main() { }
//! # fn ex() -> std::io::Result<()> {
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
//! # fn ex() -> std::io::Result<()> {
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

pub mod enums;
pub mod header;
pub mod entities;
pub mod tables;

use self::header::*;
use self::entities::*;
use self::tables::*;

use self::enums::*;
use enum_primitive::FromPrimitive;

use std::cmp::min;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use itertools::PutBack;

include!("expected_type.rs");

mod helper_functions;
use helper_functions::*;

//------------------------------------------------------------------------------
//                                                                 CodePairValue
//------------------------------------------------------------------------------
#[doc(hidden)]
#[derive(Debug)]
pub enum CodePairValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

impl CodePairValue {
    pub fn assert_bool(&self) -> bool {
        match self {
            &CodePairValue::Boolean(b) => b,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_i64(&self) -> i64 {
        match self {
            &CodePairValue::Long(l) => l,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_i32(&self) -> i32 {
        match self {
            &CodePairValue::Integer(i) => i,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_f64(&self) -> f64 {
        match self {
            &CodePairValue::Double(f) => f,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_string(&self) -> String {
        match self {
            &CodePairValue::Str(ref s) => s.clone(),
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_i16(&self) -> i16 {
        match self {
            &CodePairValue::Short(s) => s,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
}

//------------------------------------------------------------------------------
//                                                                      CodePair
//------------------------------------------------------------------------------
#[doc(hidden)]
pub struct CodePair {
    code: i32,
    value: CodePairValue,
}

impl CodePair {
    pub fn new(code: i32, val: CodePairValue) -> CodePair {
        CodePair { code: code, value: val }
    }
    pub fn new_str(code: i32, val: &str) -> CodePair {
        CodePair::new(code, CodePairValue::Str(val.to_string()))
    }
    pub fn new_string(code: i32, val: &String) -> CodePair {
        CodePair::new(code, CodePairValue::Str(val.clone()))
    }
    pub fn new_i16(code: i32, val: i16) -> CodePair {
        CodePair::new(code, CodePairValue::Short(val))
    }
    pub fn new_f64(code: i32, val: f64) -> CodePair {
        CodePair::new(code, CodePairValue::Double(val))
    }
    pub fn new_i64(code: i32, val: i64) -> CodePair {
        CodePair::new(code, CodePairValue::Long(val))
    }
    pub fn new_i32(code: i32, val: i32) -> CodePair {
        CodePair::new(code, CodePairValue::Integer(val))
    }
    pub fn new_bool(code: i32, val: bool) -> CodePair {
        CodePair::new(code, CodePairValue::Boolean(val))
    }
}

//------------------------------------------------------------------------------
//                                                             CodePairAsciiIter
//------------------------------------------------------------------------------
struct CodePairAsciiIter<T>
    where T: Read
{
    reader: T,
}

// Used to turn Result into Option<io::Result<T>>
macro_rules! try_option {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
        }
    )
}

// because I don't want to depend on BufRead here
fn read_line<T>(reader: &mut T, result: &mut String) -> io::Result<()>
    where T: Read {
    for c in reader.bytes() { // .bytes() is OK since DXF files must be ASCII encoded
        let c = try!(c) as char;
        result.push(c);
        if c == '\n' { break; }
    }

    Ok(())
}

impl<T: Read> Iterator for CodePairAsciiIter<T> {
    type Item = io::Result<CodePair>;
    fn next(&mut self) -> Option<io::Result<CodePair>> {
        loop {
            // Read code.  If no line is available, fail gracefully.
            let mut code_line = String::new();
            match read_line(&mut self.reader, &mut code_line) {
                Ok(_) => (),
                Err(_) => return None,
            }
            let code_line = code_line.trim();
            if code_line.is_empty() { return None; }
            let code = try_option!(code_line.parse::<i32>());

            // Read value.  If no line is available die horribly.
            let mut value_line = String::new();
            try_option!(read_line(&mut self.reader, &mut value_line));
            trim_trailing_newline(&mut value_line);

            // construct the value pair
            let value = match try_option!(get_expected_type(code)) {
                ExpectedType::Boolean => CodePairValue::Boolean(try_option!(parse_bool(value_line))),
                ExpectedType::Integer => CodePairValue::Integer(try_option!(parse_int(value_line))),
                ExpectedType::Long => CodePairValue::Long(try_option!(parse_long(value_line))),
                ExpectedType::Short => CodePairValue::Short(try_option!(parse_short(value_line))),
                ExpectedType::Double => CodePairValue::Double(try_option!(parse_double(value_line))),
                ExpectedType::Str => CodePairValue::Str(value_line), // TODO: un-escape
            };

            if code != 999 {
                return Some(Ok(CodePair {
                    code: code,
                    value: value,
                }));
            }
        }
    }
}

//------------------------------------------------------------------------------
//                                                           CodePairAsciiWriter
//------------------------------------------------------------------------------
#[doc(hidden)]
pub struct CodePairAsciiWriter<T>
    where T: Write {
    writer: T,
}

impl<T: Write> CodePairAsciiWriter<T> {
    pub fn write_code_pair(&mut self, pair: &CodePair) -> io::Result<()> {
        try!(self.writer.write_fmt(format_args!("{: >3}\r\n", pair.code)));
        let str_val = match &pair.value {
            &CodePairValue::Boolean(b) => String::from(if b { "1" } else { "0" }),
            &CodePairValue::Integer(i) => format!("{}", i),
            &CodePairValue::Long(l) => format!("{}", l),
            &CodePairValue::Short(s) => format!("{}", s),
            &CodePairValue::Double(d) => format!("{:.12}", d), // TODO: use proper precision
            &CodePairValue::Str(ref s) => s.clone(), // TODO: escape
        };
        try!(self.writer.write_fmt(format_args!("{}\r\n", &str_val)));
        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                                        Header
//------------------------------------------------------------------------------
// implementation is in `header.rs`
impl Header {
    #[doc(hidden)]
    pub fn read<I>(iter: &mut PutBack<I>) -> io::Result<Header>
        where I: Iterator<Item = io::Result<CodePair>> {
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
                            let last_header_variable = pair.value.assert_string();
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
                                    Some(Err(e)) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                                    None => break,
                                }
                            }
                        },
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected code pair")),
                    }
                },
                Some(Err(e)) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                None => break,
            }
        }

        Ok(header)
    }
    #[doc(hidden)]
    pub fn write<T>(&self, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write
    {
        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "HEADER")));
        try!(self.write_code_pairs(writer));
        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                                        Entity
//------------------------------------------------------------------------------
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
// Used to turn Option<T> into io::Result<T>.
macro_rules! try_result {
    ($expr : expr) => (
        match $expr {
            Some(v) => v,
            None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected enum value"))
        }
    )
}
// Used to safely access the last element in a Vec<T>
macro_rules! vec_last {
    ($expr : expr) => (
        match $expr.len() {
            0 => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected at least one item")),
            l => &mut $expr[l - 1],
        }
    )
}
// implementation is in `entity.rs`
impl Entity {
    /// Creates a new `Entity` with the default common values.
    pub fn new(specific: EntityType) -> Self {
        Entity {
            common: EntityCommon::new(),
            specific: specific,
        }
    }
    #[doc(hidden)]
    pub fn read<I>(iter: &mut PutBack<I>) -> io::Result<Option<Entity>>
        where I: Iterator<Item = io::Result<CodePair>>
    {
        loop {
            match iter.next() {
                // first code pair must be 0/entity-type
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    let type_string = pair.value.assert_string();
                    if type_string == "ENDSEC" {
                        iter.put_back(Ok(pair));
                        return Ok(None);
                    }

                    match EntityType::from_type_string(&type_string) {
                        Some(e) => {
                            let mut entity = Entity::new(e);
                            if !try!(entity.apply_custom_reader(iter)) {
                                // no custom reader, use the auto-generated one
                                loop {
                                    match iter.next() {
                                        Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                            // new entity or ENDSEC
                                            iter.put_back(Ok(pair));
                                            break;
                                        },
                                        Some(Ok(pair)) => try!(entity.apply_code_pair(&pair)),
                                        Some(Err(e)) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                                        None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
                                    }
                                }

                                try!(entity.post_parse());
                            }

                            return Ok(Some(entity));
                        },
                        None => {
                            // swallow unsupported entity
                            loop {
                               match iter.next() {
                                    Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                        // found another entity or ENDSEC
                                        iter.put_back(Ok(pair));
                                        break;
                                    },
                                    Some(Ok(_)) => (), // part of the unsupported entity
                                    Some(Err(e)) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                                    None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
                                }
                            }
                        }
                    }
                },
                Some(Ok(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected 0/entity-type or 0/ENDSEC")),
                Some(Err(e)) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
            }
        }
    }
    fn apply_code_pair(&mut self, pair: &CodePair) -> io::Result<()> {
        if !try!(self.specific.try_apply_code_pair(&pair)) {
            try!(self.common.apply_individual_pair(&pair));
        }
        Ok(())
    }
    fn post_parse(&mut self) -> io::Result<()> {
        match self.specific {
            EntityType::Image(ref mut image) => {
                combine_points_2(&mut image._clipping_vertices_x, &mut image._clipping_vertices_y, &mut image.clipping_vertices, Point::new);
            },
            EntityType::Leader(ref mut leader) => {
                combine_points_3(&mut leader._vertices_x, &mut leader._vertices_y, &mut leader._vertices_z, &mut leader.vertices, Point::new);
            },
            EntityType::MLine(ref mut mline) => {
                combine_points_3(&mut mline._vertices_x, &mut mline._vertices_y, &mut mline._vertices_z, &mut mline.vertices, Point::new);
                combine_points_3(&mut mline._segment_direction_x, &mut mline._segment_direction_y, &mut mline._segment_direction_z, &mut mline.segment_directions, Vector::new);
                combine_points_3(&mut mline._miter_direction_x, &mut mline._miter_direction_y, &mut mline._miter_direction_z, &mut mline.miter_directions, Vector::new);
            },
            EntityType::Section(ref mut section) => {
                combine_points_3(&mut section._vertices_x, &mut section._vertices_y, &mut section._vertices_z, &mut section.vertices, Point::new);
                combine_points_3(&mut section._back_line_vertices_x, &mut section._back_line_vertices_y, &mut section._back_line_vertices_z, &mut section.back_line_vertices, Point::new);
            },
            EntityType::Spline(ref mut spline) => {
                combine_points_3(&mut spline._control_point_x, &mut spline._control_point_y, &mut spline._control_point_z, &mut spline.control_points, Point::new);
                combine_points_3(&mut spline._fit_point_x, &mut spline._fit_point_y, &mut spline._fit_point_z, &mut spline.fit_points, Point::new);
            },
            EntityType::DgnUnderlay(ref mut underlay) => {
                combine_points_2(&mut underlay._point_x, &mut underlay._point_y, &mut underlay.points, Point::new);
            },
            EntityType::DwfUnderlay(ref mut underlay) => {
                combine_points_2(&mut underlay._point_x, &mut underlay._point_y, &mut underlay.points, Point::new);
            },
            EntityType::PdfUnderlay(ref mut underlay) => {
                combine_points_2(&mut underlay._point_x, &mut underlay._point_y, &mut underlay.points, Point::new);
            },
            EntityType::Wipeout(ref mut wo) => {
                combine_points_2(&mut wo._clipping_vertices_x, &mut wo._clipping_vertices_y, &mut wo.clipping_vertices, Point::new);
            },
            _ => (),
        }

        Ok(())
    }
    fn apply_custom_reader<I>(&mut self, iter: &mut PutBack<I>) -> io::Result<bool>
        where I: Iterator<Item = io::Result<CodePair>>
    {
        match self.specific {
            EntityType::Attribute(ref mut att) => {
                let xrecord_text = "AcDbXrecord";
                let mut last_subclass_marker = String::new();
                let mut is_version_set = false;
                let mut xrec_code_70_count = 0;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        100 => { last_subclass_marker = pair.value.assert_string(); },
                        1 => { att.value = pair.value.assert_string(); },
                        2 => {
                            if last_subclass_marker == xrecord_text {
                                att.x_record_tag = pair.value.assert_string();
                            }
                            else {
                                att.attribute_tag = pair.value.assert_string();
                            }
                        },
                        7 => { att.text_style_name = pair.value.assert_string(); },
                        10 => {
                            if last_subclass_marker == xrecord_text {
                                att.alignment_point.x = pair.value.assert_f64();
                            }
                            else {
                                att.location.x = pair.value.assert_f64();
                            }
                        },
                        20 => {
                            if last_subclass_marker == xrecord_text {
                                att.alignment_point.y = pair.value.assert_f64();
                            }
                            else {
                                att.location.y = pair.value.assert_f64();
                            }
                        },
                        30 => {
                            if last_subclass_marker == xrecord_text {
                                att.alignment_point.z = pair.value.assert_f64();
                            }
                            else {
                                att.location.z = pair.value.assert_f64();
                            }
                        },
                        11 => { att.second_alignment_point.x = pair.value.assert_f64(); },
                        21 => { att.second_alignment_point.y = pair.value.assert_f64(); },
                        31 => { att.second_alignment_point.z = pair.value.assert_f64(); },
                        39 => { att.thickness = pair.value.assert_f64(); },
                        40 => {
                            if last_subclass_marker == xrecord_text {
                                att.annotation_scale = pair.value.assert_f64();
                            }
                            else {
                                att.text_height = pair.value.assert_f64();
                            }
                        },
                        41 => { att.relative_x_scale_factor = pair.value.assert_f64(); },
                        50 => { att.rotation = pair.value.assert_f64(); },
                        51 => { att.oblique_angle = pair.value.assert_f64(); },
                        70 => {
                            if last_subclass_marker == xrecord_text {
                                match xrec_code_70_count {
                                    0 => att.m_text_flag = try_result!(MTextFlag::from_i16(pair.value.assert_i16())),
                                    1 => att.is_really_locked = as_bool(pair.value.assert_i16()),
                                    2 => att._secondary_attribute_count = pair.value.assert_i16() as i32,
                                    _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected extra value")),
                                }
                                xrec_code_70_count += 1;
                            }
                            else {
                                att.flags = pair.value.assert_i16() as i32;
                            }
                        },
                        71 => { att.text_generation_flags = pair.value.assert_i16() as i32; },
                        72 => { att.horizontal_text_justification = try_result!(HorizontalTextJustification::from_i16(pair.value.assert_i16())); },
                        73 => { att.field_length = pair.value.assert_i16(); },
                        74 => { att.vertical_text_justification = try_result!(VerticalTextJustification::from_i16(pair.value.assert_i16())); },
                        210 => { att.normal.x = pair.value.assert_f64(); },
                        220 => { att.normal.y = pair.value.assert_f64(); },
                        230 => { att.normal.z = pair.value.assert_f64(); },
                        280 => {
                            if last_subclass_marker == xrecord_text {
                                att.keep_duplicate_records = as_bool(pair.value.assert_i16());
                            }
                            else if !is_version_set {
                                att.version = try_result!(Version::from_i16(pair.value.assert_i16()));
                                is_version_set = true;
                            }
                            else {
                                att.is_locked_in_block = as_bool(pair.value.assert_i16());
                            }
                        },
                        340 => { att.secondary_attributes.push(try!(as_u32(pair.value.assert_string()))); },
                        -1 => { att.m_text = try!(as_u32(pair.value.assert_string())); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            EntityType::AttributeDefinition(ref mut att) => {
                let xrecord_text = "AcDbXrecord";
                let mut last_subclass_marker = String::new();
                let mut is_version_set = false;
                let mut xrec_code_70_count = 0;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        100 => { last_subclass_marker = pair.value.assert_string(); },
                        1 => { att.value = pair.value.assert_string(); },
                        2 => {
                            if last_subclass_marker == xrecord_text {
                                att.x_record_tag = pair.value.assert_string();
                            }
                            else {
                                att.text_tag = pair.value.assert_string();
                            }
                        },
                        3 => { att.prompt = pair.value.assert_string(); },
                        7 => { att.text_style_name = pair.value.assert_string(); },
                        10 => {
                            if last_subclass_marker == xrecord_text {
                                att.alignment_point.x = pair.value.assert_f64();
                            }
                            else {
                                att.location.x = pair.value.assert_f64();
                            }
                        },
                        20 => {
                            if last_subclass_marker == xrecord_text {
                                att.alignment_point.y = pair.value.assert_f64();
                            }
                            else {
                                att.location.y = pair.value.assert_f64();
                            }
                        },
                        30 => {
                            if last_subclass_marker == xrecord_text {
                                att.alignment_point.z = pair.value.assert_f64();
                            }
                            else {
                                att.location.z = pair.value.assert_f64();
                            }
                        },
                        11 => { att.second_alignment_point.x = pair.value.assert_f64(); },
                        21 => { att.second_alignment_point.y = pair.value.assert_f64(); },
                        31 => { att.second_alignment_point.z = pair.value.assert_f64(); },
                        39 => { att.thickness = pair.value.assert_f64(); },
                        40 => {
                            if last_subclass_marker == xrecord_text {
                                att.annotation_scale = pair.value.assert_f64();
                            }
                            else {
                                att.text_height = pair.value.assert_f64();
                            }
                        },
                        41 => { att.relative_x_scale_factor = pair.value.assert_f64(); },
                        50 => { att.rotation = pair.value.assert_f64(); },
                        51 => { att.oblique_angle = pair.value.assert_f64(); },
                        70 => {
                            if last_subclass_marker == xrecord_text {
                                match xrec_code_70_count {
                                    0 => att.m_text_flag = try_result!(MTextFlag::from_i16(pair.value.assert_i16())),
                                    1 => att.is_really_locked = as_bool(pair.value.assert_i16()),
                                    2 => att._secondary_attribute_count = pair.value.assert_i16() as i32,
                                    _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected extra value")),
                                }
                                xrec_code_70_count += 1;
                            }
                            else {
                                att.flags = pair.value.assert_i16() as i32;
                            }
                        },
                        71 => { att.text_generation_flags = pair.value.assert_i16() as i32; },
                        72 => { att.horizontal_text_justification = try_result!(HorizontalTextJustification::from_i16(pair.value.assert_i16())); },
                        73 => { att.field_length = pair.value.assert_i16(); },
                        74 => { att.vertical_text_justification = try_result!(VerticalTextJustification::from_i16(pair.value.assert_i16())); },
                        210 => { att.normal.x = pair.value.assert_f64(); },
                        220 => { att.normal.y = pair.value.assert_f64(); },
                        230 => { att.normal.z = pair.value.assert_f64(); },
                        280 => {
                            if last_subclass_marker == xrecord_text {
                                att.keep_duplicate_records = as_bool(pair.value.assert_i16());
                            }
                            else if !is_version_set {
                                att.version = try_result!(Version::from_i16(pair.value.assert_i16()));
                                is_version_set = true;
                            }
                            else {
                                att.is_locked_in_block = as_bool(pair.value.assert_i16());
                            }
                        },
                        340 => { att.secondary_attributes.push(try!(as_u32(pair.value.assert_string()))); },
                        -1 => { att.m_text = try!(as_u32(pair.value.assert_string())); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            EntityType::LwPolyline(ref mut poly) => {
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        // vertex-specific pairs
                        10 => {
                            // start a new vertex
                            poly.vertices.push(LwPolylineVertex::new());
                            vec_last!(poly.vertices).x = pair.value.assert_f64();
                        },
                        20 => { vec_last!(poly.vertices).y = pair.value.assert_f64(); },
                        40 => { vec_last!(poly.vertices).starting_width = pair.value.assert_f64(); },
                        41 => { vec_last!(poly.vertices).ending_width = pair.value.assert_f64(); },
                        42 => { vec_last!(poly.vertices).bulge = pair.value.assert_f64(); },
                        91 => { vec_last!(poly.vertices).id = pair.value.assert_i32(); },
                        // other pairs
                        39 => { poly.thickness = pair.value.assert_f64(); },
                        43 => { poly.constant_width = pair.value.assert_f64(); },
                        70 => { poly.flags = pair.value.assert_i16() as i32; },
                        210 => { poly.extrusion_direction.x = pair.value.assert_f64(); },
                        220 => { poly.extrusion_direction.y = pair.value.assert_f64(); },
                        230 => { poly.extrusion_direction.z = pair.value.assert_f64(); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            EntityType::MText(ref mut mtext) => {
                let mut reading_column_data = false;
                let mut read_column_count = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        10 => { mtext.insertion_point.x = pair.value.assert_f64(); },
                        20 => { mtext.insertion_point.y = pair.value.assert_f64(); },
                        30 => { mtext.insertion_point.z = pair.value.assert_f64(); },
                        40 => { mtext.initial_text_height = pair.value.assert_f64(); },
                        41 => { mtext.reference_rectangle_width = pair.value.assert_f64(); },
                        71 => { mtext.attachment_point = try_result!(AttachmentPoint::from_i16(pair.value.assert_i16())); },
                        72 => { mtext.drawing_direction = try_result!(DrawingDirection::from_i16(pair.value.assert_i16())); },
                        3 => { mtext.extended_text.push(pair.value.assert_string()); },
                        1 => { mtext.text = pair.value.assert_string(); },
                        7 => { mtext.text_style_name = pair.value.assert_string(); },
                        210 => { mtext.extrusion_direction.x = pair.value.assert_f64(); },
                        220 => { mtext.extrusion_direction.y = pair.value.assert_f64(); },
                        230 => { mtext.extrusion_direction.z = pair.value.assert_f64(); },
                        11 => { mtext.x_axis_direction.x = pair.value.assert_f64(); },
                        21 => { mtext.x_axis_direction.y = pair.value.assert_f64(); },
                        31 => { mtext.x_axis_direction.z = pair.value.assert_f64(); },
                        42 => { mtext.horizontal_width = pair.value.assert_f64(); },
                        43 => { mtext.vertical_height = pair.value.assert_f64(); },
                        50 => {
                            if reading_column_data {
                                if read_column_count {
                                    mtext.column_heights.push(pair.value.assert_f64());
                                }
                                else {
                                    mtext.column_count = pair.value.assert_f64() as i32;
                                    read_column_count = true;
                                }
                            }
                            else {
                                mtext.rotation_angle = pair.value.assert_f64();
                            }
                        },
                        73 => { mtext.line_spacing_style = try_result!(MTextLineSpacingStyle::from_i16(pair.value.assert_i16())); },
                        44 => { mtext.line_spacing_factor = pair.value.assert_f64(); },
                        90 => { mtext.background_fill_setting = try_result!(BackgroundFillSetting::from_i32(pair.value.assert_i32())); },
                        420 => { mtext.background_color_rgb = pair.value.assert_i32(); },
                        430 => { mtext.background_color_name = pair.value.assert_string(); },
                        45 => { mtext.fill_box_scale = pair.value.assert_f64(); },
                        63 => { mtext.background_fill_color = Color::from_raw_value(pair.value.assert_i16()); },
                        441 => { mtext.background_fill_color_transparency = pair.value.assert_i32(); },
                        75 => {
                            mtext.column_type = pair.value.assert_i16();
                            reading_column_data = true;
                        },
                        76 => { mtext.column_count = pair.value.assert_i16() as i32; },
                        78 => { mtext.is_column_flow_reversed = as_bool(pair.value.assert_i16()); },
                        79 => { mtext.is_column_auto_height = as_bool(pair.value.assert_i16()); },
                        48 => { mtext.column_width = pair.value.assert_f64(); },
                        49 => { mtext.column_gutter = pair.value.assert_f64(); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            _ => return Ok(false), // no custom reader
        }

        Ok(true)
    }
    #[doc(hidden)]
    pub fn write<T>(&self, version: &AcadVersion, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write {
        if self.specific.is_supported_on_version(version) {
            try!(writer.write_code_pair(&CodePair::new_str(0, self.specific.to_type_string())));
            try!(self.common.write(version, write_handles, writer));
            try!(self.specific.write(&self.common, version, writer));
            try!(self.post_write(&version, write_handles, writer));
        }

        Ok(())
    }
    fn post_write<T>(&self, version: &AcadVersion, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write {
        match self.specific {
            // TODO: write trailing MText on Attribute and AttributeDefinition
            EntityType::Polyline(ref poly) => {
                for v in &poly.vertices {
                    let v = Entity { common: Default::default(), specific: EntityType::Vertex(v.clone()) };
                    try!(v.write(&version, write_handles, writer));
                }
                let seqend = Entity { common: Default::default(), specific: EntityType::Seqend(Default::default()) };
                try!(seqend.write(&version, write_handles, writer));
            },
            _ => (),
        }

        Ok(())
    }
}

fn combine_points_2<F, T>(v1: &mut Vec<f64>, v2: &mut Vec<f64>, result: &mut Vec<T>, comb: F)
    where F: Fn(f64, f64, f64) -> T {
    for i in 0..min(v1.len(), v2.len()) {
        result.push(comb(v1[i], v2[i], 0.0));
    }
    v1.clear();
    v2.clear();
}

fn combine_points_3<F, T>(v1: &mut Vec<f64>, v2: &mut Vec<f64>, v3: &mut Vec<f64>, result: &mut Vec<T>, comb: F)
    where F: Fn(f64, f64, f64) -> T {
    for i in 0..min(v1.len(), min(v2.len(), v3.len())) {
        result.push(comb(v1[i], v2[i], v3[i]));
    }
    v1.clear();
    v2.clear();
    v3.clear();
}

//------------------------------------------------------------------------------
//                                                  ProxyEntity-specific methods
//------------------------------------------------------------------------------
impl ProxyEntity {
    // lower word
    pub fn get_object_drawing_format_version(&self) -> i32 {
        (self._object_drawing_format & 0xFFFF) as i32
    }
    pub fn set_object_drawing_format_version(&mut self, version: i32) {
        self._object_drawing_format |= version as u32 & 0xFFFF;
    }
    // upper word
    pub fn get_object_maintenance_release_version(&self) -> i32 {
        self._object_drawing_format as i32 >> 4
    }
    pub fn set_object_mainenance_release_version(&mut self, version: i32) {
        self._object_drawing_format = (version << 4) as u32 + (self._object_drawing_format & 0xFFFF);
    }
}

//------------------------------------------------------------------------------
//                                                                       Drawing
//------------------------------------------------------------------------------
/// Represents a DXF drawing.
pub struct Drawing {
    /// The drawing's header.  Contains various drawing-specific values and settings.
    pub header: Header,
    /// The entities contained by the drawing.
    pub entities: Vec<Entity>,
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
}

// Used to turn Result<T> into io::Result<T>
macro_rules! try_result {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    )
}

// public implementation
impl Drawing {
    /// Creates a new empty `Drawing`.
    pub fn new() -> Self {
        Drawing {
            header: Header::new(),
            entities: vec![],
            app_ids: vec![],
            block_records: vec![],
            dim_styles: vec![],
            layers: vec![],
            line_types: vec![],
            styles: vec![],
            ucs: vec![],
            views: vec![],
            view_ports: vec![],
        }
    }
    /// Loads a `Drawing` from anything that implements the `Read` trait.
    pub fn load<T>(reader: T) -> io::Result<Drawing>
        where T: Read {
        let reader = CodePairAsciiIter { reader: reader };
        let mut drawing = Drawing::new();
        let mut iter = PutBack::new(reader);
        try!(Drawing::read_sections(&mut drawing, &mut iter));
        match iter.next() {
            Some(Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "EOF" => Ok(drawing),
            Some(Ok(CodePair { code: c, value: v })) => Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/EOF but got {}/{:?}", c, v))),
            Some(Err(e)) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            None => Ok(drawing),
        }
    }
    /// Loads a `Drawing` from disk, using a `BufReader`.
    pub fn load_file(file_name: &str) -> io::Result<Drawing> {
        let path = Path::new(file_name);
        let file = try!(File::open(&path));
        let buf_reader = BufReader::new(file);
        Drawing::load(buf_reader)
    }
    /// Writes a `Drawing` to anything that implements the `Write` trait.
    pub fn save<T>(&self, writer: &mut T) -> io::Result<()>
        where T: Write {
        let mut writer = CodePairAsciiWriter { writer: writer };
        try!(self.header.write(&mut writer));
        let write_handles = self.header.version >= AcadVersion::R13 || self.header.handles_enabled;
        try!(self.write_tables(write_handles, &mut writer));
        try!(self.write_entities(write_handles, &mut writer));
        // TODO: write other sections
        try!(writer.write_code_pair(&CodePair::new_str(0, "EOF")));
        Ok(())
    }
    /// Writes a `Drawing` to disk, using a `BufWriter`.
    pub fn save_file(&self, file_name: &str) -> io::Result<()> {
        let path = Path::new(file_name);
        let file = try!(File::create(&path));
        let mut buf_writer = BufWriter::new(file);
        self.save(&mut buf_writer)
    }
}

// private implementation
impl Drawing {
    fn write_tables<T>(&self, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write {
        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "TABLES")));
        try!(write_tables(&self, write_handles, writer));
        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn write_entities<T>(&self, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> io::Result<()>
        where T: Write {
        try!(writer.write_code_pair(&CodePair::new_str(0, "SECTION")));
        try!(writer.write_code_pair(&CodePair::new_str(2, "ENTITIES")));
        for e in &self.entities {
            try!(e.write(&self.header.version, write_handles, writer));
        }

        try!(writer.write_code_pair(&CodePair::new_str(0, "ENDSEC")));
        Ok(())
    }
    fn read_sections<I>(drawing: &mut Drawing, iter: &mut PutBack<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        loop {
            match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    match &*pair.value.assert_string() {
                        "EOF" => {
                            iter.put_back(Ok(pair));
                            break;
                        },
                        "SECTION" => {
                            match iter.next() {
                               Some(Ok(CodePair { code: 2, value: CodePairValue::Str(s) })) => {
                                    match &*s {
                                        "HEADER" => drawing.header = try!(header::Header::read(iter)),
                                        "ENTITIES" => try!(drawing.read_entities(iter)),
                                        "TABLES" => try!(drawing.read_tables(iter)),
                                        // TODO: read other sections
                                        _ => try!(Drawing::swallow_section(iter)),
                                    }

                                    match iter.next() {
                                        Some(Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "ENDSEC" => (),
                                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected 0/ENDSEC")),
                                    }
                                },
                                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected section name")),
                            }
                        },
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/SECTION, got 0/{:?}", pair.value))),
                    }
                },
                Some(Ok(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected 0/SECTION or 0/EOF")),
                Some(Err(e)) => return Err(e),
                None => break, // ideally should have been 0/EOF
            }
        }

        Ok(())
    }
    fn swallow_section<I>(iter: &mut PutBack<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 && pair.value.assert_string() == "ENDSEC" {
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
    fn read_entities<I>(&mut self, iter: &mut PutBack<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        let mut iter = PutBack::new(EntityIter { iter: iter });
        loop {
            match iter.next() {
                Some(Ok(Entity { ref common, specific: EntityType::Insert(ref ins) })) if ins.has_attributes => {
                    let mut ins = ins.clone(); // 12 fields
                    loop {
                        match iter.next() {
                            Some(Ok(Entity { specific: EntityType::Attribute(att), .. })) => ins.attributes.push(att),
                            Some(Ok(ent)) => {
                                // stop gathering on any non-ATTRIBUTE
                                iter.put_back(Ok(ent));
                                break;
                            },
                            Some(Err(e)) => return Err(e),
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
                Some(Ok(Entity { common, specific: EntityType::Polyline(poly) })) => {
                    let mut poly = poly.clone(); // 13 fields
                    loop {
                        match iter.next() {
                            Some(Ok(Entity { specific: EntityType::Vertex(vertex), .. })) => poly.vertices.push(vertex),
                            Some(Ok(ent)) => {
                                // stop gathering on any non-VERTEX
                                iter.put_back(Ok(ent));
                                break;
                            },
                            Some(Err(e)) => return Err(e),
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
                Some(Ok(entity)) => self.entities.push(entity),
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(())
    }
    fn swallow_seqend<I>(iter: &mut PutBack<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<Entity>> {
        match iter.next() {
            Some(Ok(Entity { specific: EntityType::Seqend(_), .. })) => (),
            Some(Ok(ent)) => iter.put_back(Ok(ent)),
            Some(Err(e)) => return Err(e),
            None => (),
        }

        Ok(())
    }
    fn read_tables<I>(&mut self, iter: &mut PutBack<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 {
                        match &*pair.value.assert_string() {
                            "ENDSEC" => {
                                iter.put_back(Ok(pair));
                                break;
                            },
                            "TABLE" => try!(read_specific_table(self, iter)),
                            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected code pair")),
                        }
                    }
                    else {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected value pair"));
                    }
                },
                Some(Err(e)) => return Err(e),
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
            }
        }

        Ok(())
    }
    #[doc(hidden)]
    pub fn swallow_table<I>(iter: &mut PutBack<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    if pair.code == 0 {
                        match &*pair.value.assert_string() {
                            "TABLE" | "ENDSEC" | "ENDTAB" => {
                                iter.put_back(Ok(pair));
                                break;
                            },
                            _ => (), // swallow the code pair
                        }
                    }
                }
                Some(Err(e)) => return Err(e),
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
            }
        }

        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                                    EntityIter
//------------------------------------------------------------------------------
struct EntityIter<'a, I: 'a + Iterator<Item = io::Result<CodePair>>> {
    iter: &'a mut PutBack<I>,
}

impl<'a, I: 'a + Iterator<Item = io::Result<CodePair>>> Iterator for EntityIter<'a, I> {
    type Item = io::Result<Entity>;
    fn next(&mut self) -> Option<io::Result<Entity>> {
        match Entity::read(self.iter) {
            Ok(Some(e)) => Some(Ok(e)),
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
    x: f64,
    /// The Y value of the point.
    y: f64,
    /// The Z value of the point.
    z: f64,
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
    pub fn set(&mut self, pair: &CodePair) -> io::Result<()> {
        match pair.code {
            10 => self.x = pair.value.assert_f64(),
            20 => self.y = pair.value.assert_f64(),
            30 => self.z = pair.value.assert_f64(),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unexpected code for Point: {}", pair.code))),
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
    x: f64,
    /// The Y component of the vector.
    y: f64,
    /// The Z component of the vector.
    z: f64,
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
    pub fn set(&mut self, pair: &CodePair) -> io::Result<()> {
        match pair.code {
            10 => self.x = pair.value.assert_f64(),
            20 => self.y = pair.value.assert_f64(),
            30 => self.z = pair.value.assert_f64(),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unexpected code for Vector: {}", pair.code))),
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
#[derive(Clone, Debug, PartialEq)]
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
