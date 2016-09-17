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

mod drawing;
pub use drawing::Drawing;

pub mod enums;

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

use entities::*;
use header::*;
use tables::*;

use self::enums::*;
use enum_primitive::FromPrimitive;

use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::{Read, Write};
use std::num;

use itertools::PutBack;

include!("expected_type.rs");

mod helper_functions;
use helper_functions::*;

//------------------------------------------------------------------------------
//                                                                 CodePairValue
//------------------------------------------------------------------------------
#[doc(hidden)]
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

impl Clone for CodePairValue {
    fn clone(&self) -> Self {
        match self {
            &CodePairValue::Boolean(b) => CodePairValue::Boolean(b),
            &CodePairValue::Integer(i) => CodePairValue::Integer(i),
            &CodePairValue::Long(l) => CodePairValue::Long(l),
            &CodePairValue::Short(s) => CodePairValue::Short(s),
            &CodePairValue::Double(d) => CodePairValue::Double(d),
            &CodePairValue::Str(ref s) => CodePairValue::Str(String::from(s.as_str())),
        }
    }
}

impl Debug for CodePairValue {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            &CodePairValue::Boolean(b) => write!(formatter, "{}", if b { 1 } else { 0 }),
            &CodePairValue::Integer(i) => write!(formatter, "{}", i),
            &CodePairValue::Long(l) => write!(formatter, "{}", l),
            &CodePairValue::Short(s) => write!(formatter, "{}", s),
            &CodePairValue::Double(d) => write!(formatter, "{:.12}", d),
            &CodePairValue::Str(ref s) => write!(formatter, "{}", s),
        }
    }
}

//------------------------------------------------------------------------------
//                                                                      CodePair
//------------------------------------------------------------------------------
#[doc(hidden)]
#[derive(Clone)]
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

impl Debug for CodePair {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}/{:?}", self.code, &self.value)
    }
}

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
    UnexpectedEndOfInput,
    UnexpectedEnumValue,
    UnexpectedEmptySet,
    ExpectedTableType,
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
            &DxfError::UnexpectedEndOfInput => write!(formatter, "the input unexpectedly ended before the drawing was completely loaded"),
            &DxfError::UnexpectedEnumValue => write!(formatter, "the specified enum value does not fall into the expected range"),
            &DxfError::UnexpectedEmptySet => write!(formatter, "the set was not expected to be empty"),
            &DxfError::ExpectedTableType => write!(formatter, "a 2/<table-type> code pair was expected"),
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
            &DxfError::UnexpectedEndOfInput => "the input unexpectedly ended before the drawing was completely loaded",
            &DxfError::UnexpectedEnumValue => "the specified enum value does not fall into the expected range",
            &DxfError::UnexpectedEmptySet => "the set was not expected to be empty",
            &DxfError::ExpectedTableType => "a 2/<table-type> code pair was expected",
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
//                                                             CodePairAsciiIter
//------------------------------------------------------------------------------
struct CodePairAsciiIter<T>
    where T: Read
{
    reader: T,
}

// Used to turn Result<T> into Option<Result<T>>.
macro_rules! try_into_result {
    ($expr : expr) => (
        match $expr {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        }
    )
}

// because I don't want to depend on BufRead here
fn read_line<T>(reader: &mut T, result: &mut String) -> DxfResult<()>
    where T: Read {
    for c in reader.bytes() { // .bytes() is OK since DXF files must be ASCII encoded
        let c = try!(c) as char;
        result.push(c);
        if c == '\n' { break; }
    }

    Ok(())
}

impl<T: Read> Iterator for CodePairAsciiIter<T> {
    type Item = DxfResult<CodePair>;
    fn next(&mut self) -> Option<DxfResult<CodePair>> {
        loop {
            // Read code.  If no line is available, fail gracefully.
            let mut code_line = String::new();
            match read_line(&mut self.reader, &mut code_line) {
                Ok(_) => (),
                Err(_) => return None,
            }
            let code_line = code_line.trim();
            if code_line.is_empty() { return None; }
            let code = try_into_result!(parse_i32(String::from(code_line)));

            // Read value.  If no line is available die horribly.
            let mut value_line = String::new();
            try_into_result!(read_line(&mut self.reader, &mut value_line));
            trim_trailing_newline(&mut value_line);

            // construct the value pair
            let expected_type = match get_expected_type(code) {
                Some(t) => t,
                None => return Some(Err(DxfError::UnexpectedEnumValue)),
            };
            let value = match expected_type {
                ExpectedType::Boolean => CodePairValue::Boolean(try_into_result!(parse_bool(value_line))),
                ExpectedType::Integer => CodePairValue::Integer(try_into_result!(parse_i32(value_line))),
                ExpectedType::Long => CodePairValue::Long(try_into_result!(parse_i64(value_line))),
                ExpectedType::Short => CodePairValue::Short(try_into_result!(parse_i16(value_line))),
                ExpectedType::Double => CodePairValue::Double(try_into_result!(parse_f64(value_line))),
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
    pub fn write_code_pair(&mut self, pair: &CodePair) -> DxfResult<()> {
        try!(self.writer.write_fmt(format_args!("{: >3}\r\n", pair.code)));
        try!(self.writer.write_fmt(format_args!("{:?}\r\n", &pair.value)));
        Ok(())
    }
}

//------------------------------------------------------------------------------
//                                                                        Header
//------------------------------------------------------------------------------
// implementation is in `header.rs`
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
    pub fn write<T>(&self, writer: &mut CodePairAsciiWriter<T>) -> DxfResult<()>
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
    pub fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Option<Entity>>
        where I: Iterator<Item = DxfResult<CodePair>> {
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
                                        Some(Err(e)) => return Err(e),
                                        None => return Err(DxfError::UnexpectedEndOfInput),
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
                                    Some(Err(e)) => return Err(e),
                                    None => return Err(DxfError::UnexpectedEndOfInput),
                                }
                            }
                        }
                    }
                },
                Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/entity-type or 0/ENDSEC"))),
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
    fn apply_custom_reader<I>(&mut self, iter: &mut PutBack<I>) -> DxfResult<bool>
        where I: Iterator<Item = DxfResult<CodePair>>
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
                                    _ => return Err(DxfError::UnexpectedCodePair(pair, String::new())),
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
                                    _ => return Err(DxfError::UnexpectedCodePair(pair, String::new())),
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
    pub fn write<T>(&self, version: &AcadVersion, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> DxfResult<()>
        where T: Write {
        if self.specific.is_supported_on_version(version) {
            try!(writer.write_code_pair(&CodePair::new_str(0, self.specific.to_type_string())));
            try!(self.common.write(version, write_handles, writer));
            try!(self.specific.write(&self.common, version, writer));
            try!(self.post_write(&version, write_handles, writer));
        }

        Ok(())
    }
    fn post_write<T>(&self, version: &AcadVersion, write_handles: bool, writer: &mut CodePairAsciiWriter<T>) -> DxfResult<()>
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
    for (x, y) in v1.drain(..).zip(v2.drain(..)) {
        result.push(comb(x, y, 0.0));
    }
    v1.clear();
    v2.clear();
}

fn combine_points_3<F, T>(v1: &mut Vec<f64>, v2: &mut Vec<f64>, v3: &mut Vec<f64>, result: &mut Vec<T>, comb: F)
    where F: Fn(f64, f64, f64) -> T {
    for (x, (y, z)) in v1.drain(..).zip(v2.drain(..).zip(v3.drain(..))) {
        result.push(comb(x, y, z))
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
    pub fn set(&mut self, pair: &CodePair) -> DxfResult<()> {
        match pair.code {
            10 => self.x = pair.value.assert_f64(),
            20 => self.y = pair.value.assert_f64(),
            30 => self.z = pair.value.assert_f64(),
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
    pub fn set(&mut self, pair: &CodePair) -> DxfResult<()> {
        match pair.code {
            10 => self.x = pair.value.assert_f64(),
            20 => self.y = pair.value.assert_f64(),
            30 => self.z = pair.value.assert_f64(),
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
