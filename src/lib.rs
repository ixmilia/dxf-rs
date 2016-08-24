// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

#[macro_use] extern crate enum_primitive;

pub mod enums;
pub mod header;
pub mod entities;

use self::header::*;
use self::entities::*;

use self::enums::*;
use enum_primitive::FromPrimitive;

use std::cmp::min;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::iter::Peekable;
use std::path::Path;

include!("expected_type.rs");

mod helper_functions;
use helper_functions::*;

////////////////////////////////////////////////////////////////////////////////
//                                                                 CodePairValue
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub enum CodePairValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

////////////////////////////////////////////////////////////////////////////////
//                                                                      CodePair
////////////////////////////////////////////////////////////////////////////////
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
    pub fn new_short(code: i32, val: i16) -> CodePair {
        CodePair::new(code, CodePairValue::Short(val))
    }
    pub fn new_double(code: i32, val: f64) -> CodePair {
        CodePair::new(code, CodePairValue::Double(val))
    }
    pub fn new_long(code: i32, val: i64) -> CodePair {
        CodePair::new(code, CodePairValue::Long(val))
    }
    pub fn new_int(code: i32, val: i32) -> CodePair {
        CodePair::new(code, CodePairValue::Integer(val))
    }
    pub fn new_bool(code: i32, val: bool) -> CodePair {
        CodePair::new(code, CodePairValue::Boolean(val))
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                             CodePairAsciiIter
////////////////////////////////////////////////////////////////////////////////
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

        Some(Ok(CodePair {
            code: code,
            value: value,
        }))
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                           CodePairAsciiWriter
////////////////////////////////////////////////////////////////////////////////
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
        try!(self.writer.write_fmt(format_args!("{}\r\n", str_val.as_str())));
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                        Header
////////////////////////////////////////////////////////////////////////////////
// implementation is in `header.rs`
impl Header {
    pub fn read<I>(peekable: &mut Peekable<I>) -> io::Result<Header>
        where I: Iterator<Item = io::Result<CodePair>>
    {
        let mut header = Header::new();
        loop {
            match peekable.peek() {
                Some(&Ok(CodePair { code: 9, value: _ })) => {
                    let pair = peekable.next().unwrap().ok().unwrap(); // unwrap() and ok() calls are valid due to the match above
                    let last_header_variable = string_value(&pair.value);
                    loop {
                        match peekable.peek() {
                            Some(&Ok(CodePair { code: c, value: _ })) if c == 0 || c == 9 => break, // 0/ENDSEC or a new header variable
                            Some(&Ok(_)) => {
                                let pair = peekable.next().unwrap().ok().unwrap(); // unwrap() and ok() calls are valid due to the match above
                                try!(header.set_header_value(last_header_variable.as_str(), &pair));
                            },
                            Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "unable to read header variable value")),
                            None => break,
                        }
                    }
                },
                Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "unable to read header")),
                _ => break,
            }
        }

        Ok(header)
    }
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

////////////////////////////////////////////////////////////////////////////////
//                                                                        Entity
////////////////////////////////////////////////////////////////////////////////
// returns the next CodePair that's not 0, or bails out early
macro_rules! next_pair {
    ($expr : expr) => (
        match $expr.peek() {
            Some(&Ok(CodePair { code: 0, .. })) | None => return Ok(true),
            _ => $expr.next().unwrap().ok().unwrap(), // unwrap() and ok() calls are valid due to the match above
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
// implementation is in `entity.rs`
impl Entity {
    pub fn new(specific: EntityType) -> Self {
        Entity {
            common: EntityCommon::new(),
            specific: specific,
        }
    }
    pub fn read<I>(peekable: &mut Peekable<I>) -> io::Result<Option<Entity>>
        where I: Iterator<Item = io::Result<CodePair>>
    {
        let entity_type;
        loop {
            match peekable.peek() {
                // first code pair must be 0/entity-type
                Some(&Ok(CodePair { code: 0, .. })) => {
                    let pair = peekable.next().unwrap().ok().unwrap(); // unwrap() and ok() calls are valid due to the match above
                    let type_string = string_value(&pair.value);
                    if type_string == "ENDSEC" {
                        return Ok(None);
                    }

                    match EntityType::from_type_string(type_string.as_str()) {
                        Some(e) => {
                            entity_type = e;
                            break;
                        },
                        None => {
                            // swallow unsupported entity
                            loop {
                               match peekable.peek() {
                                    Some(&Ok(CodePair { code: 0, .. })) => break, // found another entity or 0/ENDSEC
                                    Some(&Ok(_)) => { peekable.next(); }, // part of the unsupported entity
                                    Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected error")),
                                    None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
                                }
                            }
                        }
                    }
                },
                Some(&Ok(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "expected 0/entity-type or 0/ENDSEC")),
                Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "")),
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
            }
        }

        let mut entity = Entity::new(entity_type);
        if !try!(entity.apply_custom_reader(peekable)) {
            // no custom reader, use the auto-generated one
            loop {
                match peekable.peek() {
                    Some(&Ok(CodePair { code: 0, .. })) => break, // new entity or 0/ENDSEC
                    Some(&Ok(_)) => {
                        let pair = peekable.next().unwrap().ok().unwrap(); // unwrap() and ok() calls are valid due to the match above
                        try!(entity.apply_code_pair(&pair));
                    },
                    Some(&Err(_)) => return Err(io::Error::new(io::ErrorKind::InvalidData, "error reading drawing")),
                    None => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected end of input")),
                }
            }

            try!(entity.post_parse());
        }

        Ok(Some(entity))
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
            EntityType::Underlay(ref mut underlay) => {
                combine_points_2(&mut underlay._point_x, &mut underlay._point_y, &mut underlay.points, Point::new);
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
    fn apply_custom_reader<I>(&mut self, peekable: &mut Peekable<I>) -> io::Result<bool>
        where I: Iterator<Item = io::Result<CodePair>>
    {
        match self.specific {
            EntityType::MText(ref mut mtext) => {
                let mut reading_column_data = false;
                let mut read_column_count = false;
                loop {
                    let pair = next_pair!(peekable);
                    match pair.code {
                        10 => { mtext.insertion_point.x = double_value(&pair.value); },
                        20 => { mtext.insertion_point.y = double_value(&pair.value); },
                        30 => { mtext.insertion_point.z = double_value(&pair.value); },
                        40 => { mtext.initial_text_height = double_value(&pair.value); },
                        41 => { mtext.reference_rectangle_width = double_value(&pair.value); },
                        71 => { mtext.attachment_point = try_result!(AttachmentPoint::from_i16(short_value(&pair.value))); },
                        72 => { mtext.drawing_direction = try_result!(DrawingDirection::from_i16(short_value(&pair.value))); },
                        3 => { mtext.extended_text.push(string_value(&pair.value)); },
                        1 => { mtext.text = string_value(&pair.value); },
                        7 => { mtext.text_style_name = string_value(&pair.value); },
                        210 => { mtext.extrusion_direction.x = double_value(&pair.value); },
                        220 => { mtext.extrusion_direction.y = double_value(&pair.value); },
                        230 => { mtext.extrusion_direction.z = double_value(&pair.value); },
                        11 => { mtext.x_axis_direction.x = double_value(&pair.value); },
                        21 => { mtext.x_axis_direction.y = double_value(&pair.value); },
                        31 => { mtext.x_axis_direction.z = double_value(&pair.value); },
                        42 => { mtext.horizontal_width = double_value(&pair.value); },
                        43 => { mtext.vertical_height = double_value(&pair.value); },
                        50 => {
                            if reading_column_data {
                                if read_column_count {
                                    mtext.column_heights.push(double_value(&pair.value));
                                }
                                else {
                                    mtext.column_count = double_value(&pair.value) as i32;
                                    read_column_count = true;
                                }
                            }
                            else {
                                mtext.rotation_angle = double_value(&pair.value);
                            }
                        },
                        73 => { mtext.line_spacing_style = try_result!(MTextLineSpacingStyle::from_i16(short_value(&pair.value))); },
                        44 => { mtext.line_spacing_factor = double_value(&pair.value); },
                        90 => { mtext.background_fill_setting = try_result!(BackgroundFillSetting::from_i32(int_value(&pair.value))); },
                        420 => { mtext.background_color_r_g_b = int_value(&pair.value); },
                        430 => { mtext.background_color_name = string_value(&pair.value); },
                        45 => { mtext.fill_box_scale = double_value(&pair.value); },
                        63 => { mtext.background_fill_color = Color::from_raw_value(short_value(&pair.value)); },
                        441 => { mtext.background_fill_color_transparency = int_value(&pair.value); },
                        75 => {
                            mtext.column_type = short_value(&pair.value);
                            reading_column_data = true;
                        },
                        76 => { mtext.column_count = short_value(&pair.value) as i32; },
                        78 => { mtext.is_column_flow_reversed = as_bool(short_value(&pair.value)); },
                        79 => { mtext.is_column_auto_height = as_bool(short_value(&pair.value)); },
                        48 => { mtext.column_width = double_value(&pair.value); },
                        49 => { mtext.column_gutter = double_value(&pair.value); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            _ => return Ok(false), // no custom reader
        }

        Ok(true)
    }
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

////////////////////////////////////////////////////////////////////////////////
//                                                                       Drawing
////////////////////////////////////////////////////////////////////////////////
pub struct Drawing {
    pub header: Header,
    pub entities: Vec<Entity>,
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
    pub fn new() -> Self {
        Drawing {
            header: Header::new(),
            entities: vec![],
        }
    }
    pub fn load<T>(reader: T) -> io::Result<Drawing>
        where T: Read {
        let reader = CodePairAsciiIter { reader: reader };
        let mut peekable = reader.peekable();
        let mut drawing = Drawing::new();
        match Drawing::read_sections(&mut drawing, &mut peekable) {
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            _ => (),
        }
        match peekable.next() {
            Some(Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "EOF" => Ok(drawing),
            Some(Ok(CodePair { code: c, value: v })) => Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/EOF but got {}/{:?}", c, v))),
            Some(Err(e)) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            None => Ok(drawing), //Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/EOF but got nothing"))), // n.b., this is probably fine
        }
    }
    pub fn load_file(file_name: &str) -> io::Result<Drawing> {
        let path = Path::new(file_name);
        let file = try!(File::open(&path));
        let buf_reader = BufReader::new(file);
        Drawing::load(buf_reader)
    }
    pub fn save<T>(&self, writer: &mut T) -> io::Result<()>
        where T: Write {
        let mut writer = CodePairAsciiWriter { writer: writer };
        try!(self.header.write(&mut writer));
        let write_handles = self.header.version >= AcadVersion::R13 || self.header.handles_enabled;
        try!(self.write_entities(write_handles, &mut writer));
        // TODO: write other sections
        try!(writer.write_code_pair(&CodePair::new_str(0, "EOF")));
        Ok(())
    }
    pub fn save_file(&self, file_name: &str) -> io::Result<()> {
        let path = Path::new(file_name);
        let file = try!(File::create(&path));
        let mut buf_writer = BufWriter::new(file);
        self.save(&mut buf_writer)
    }
}

// private implementation
impl Drawing {
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
    fn read_sections<I>(drawing: &mut Drawing, peekable: &mut Peekable<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        loop {
            match peekable.peek() {
                Some(&Ok(CodePair { code: 0, value: CodePairValue::Str(_) })) => {
                    let pair = peekable.next().unwrap().ok().unwrap(); // consume 0/SECTION.  unwrap() and ok() calls are valid due to the match above
                    if string_value(&pair.value).as_str() == "EOF" { break; }
                    if string_value(&pair.value).as_str() != "SECTION" { return Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected 0/SECTION, got 0/{}", string_value(&pair.value).as_str()))); }
                    match peekable.peek() {
                        Some(&Ok(CodePair { code: 2, value: CodePairValue::Str(_) })) => {
                            let pair = peekable.next().unwrap().ok().unwrap(); // consume 2/<section-name>.  unwrap() and ok() calls are valid due to the match above
                            match string_value(&pair.value).as_str() {
                                "HEADER" => drawing.header = try!(header::Header::read(peekable)),
                                "ENTITIES" => try!(drawing.read_entities(peekable)),
                                // TODO: read other sections
                                _ => Drawing::swallow_section(peekable),
                            }

                            let mut swallow_endsec = false;
                            match peekable.peek() {
                                Some(&Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "ENDSEC" => swallow_endsec = true,
                                _ => (), // expected 0/ENDSEC
                            }

                            if swallow_endsec {
                                peekable.next();
                            }
                        },
                        _ => (), // expected 2/<section-name>
                    }
                },
                _ => break,
            }
        }

        Ok(())
    }
    fn swallow_section<I>(peekable: &mut Peekable<I>)
        where I: Iterator<Item = io::Result<CodePair>> {
        loop {
            let mut quit = false;
            match peekable.peek() {
                Some(&Ok(CodePair { code: 0, value: CodePairValue::Str(ref s) })) if s == "ENDSEC" => quit = true,
                _ => (),
            }

            if quit {
                return;
            }
            else {
                peekable.next();
            }
        }
    }
    fn read_entities<I>(&mut self, peekable: &mut Peekable<I>) -> io::Result<()>
        where I: Iterator<Item = io::Result<CodePair>> {
        let mut peekable = EntityIter { peekable: peekable }.peekable();
        loop {
            match peekable.peek() {
                Some(&Ok(Entity { specific: EntityType::Polyline(_), .. })) => {
                    let entity = peekable.next().unwrap().ok().unwrap(); // these unwrap() and ok() calls are valid due to the match above
                    match entity.specific {
                        EntityType::Polyline(ref poly) => {
                            // found a polyline, gather the following VERTEX entities
                            let mut poly = poly.clone(); // 13 fields
                            loop {
                                match peekable.peek() {
                                    Some(&Ok(Entity { specific: EntityType::Vertex(_), .. })) => {
                                        let vertex = peekable.next().unwrap().ok().unwrap(); // these unwrap() and ok() calls are valid due to the match above
                                        match vertex.specific {
                                            EntityType::Vertex(v) => poly.vertices.push(v),
                                            _ => panic!("this will never happen"),
                                        }
                                    },
                                    _ => break, // stop gathering on any non-VERTEX
                                }
                            }

                            // swallow the following SEQEND if it's present
                            match peekable.peek() {
                                Some(&Ok(Entity { specific: EntityType::Seqend(_), .. })) => drop(peekable.next()),
                                _ => (),
                            }

                            // and finally keep the POLYLINE
                            self.entities.push(Entity {
                                common: entity.common.clone(), // 18 fields
                                specific: EntityType::Polyline(poly)
                            });
                        },
                        _ => panic!("this will never happen"),
                    }
                },
                Some(&Ok(_)) => {
                    let entity = peekable.next().unwrap().ok().unwrap(); // these unwrap() and ok() calls are valid due to the match above
                    self.entities.push(entity);
                },
                Some(&Err(_)) => {
                    let err = peekable.next().unwrap().err().unwrap(); // these unwrap() and err() calls are valid due to the match above
                    return Err(err);
                },
                None => break,
            }
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                    EntityIter
////////////////////////////////////////////////////////////////////////////////
struct EntityIter<'a, I: 'a + Iterator<Item = io::Result<CodePair>>> {
    peekable: &'a mut Peekable<I>,
}

impl<'a, I: 'a + Iterator<Item = io::Result<CodePair>>> Iterator for EntityIter<'a, I> {
    type Item = io::Result<Entity>;
    fn next(&mut self) -> Option<io::Result<Entity>> {
        match Entity::read(self.peekable) {
            Ok(Some(e)) => Some(Ok(e)),
            Ok(None) | Err(_) => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                         Point
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point {
        Point{
            x: x,
            y: y,
            z: z,
        }
    }
    pub fn origin() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }
    pub fn set(&mut self, pair: &CodePair) -> io::Result<()> {
        match pair.code {
            10 => self.x = double_value(&pair.value),
            20 => self.y = double_value(&pair.value),
            30 => self.z = double_value(&pair.value),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unexpected code for Point: {}", pair.code))),
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                        Vector
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, PartialEq)]
pub struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector {
            x: x,
            y: y,
            z: z,
        }
    }
    pub fn zero() -> Vector {
        Vector::new(0.0, 0.0, 0.0)
    }
    pub fn x_axis() -> Vector {
        Vector::new(1.0, 0.0, 0.0)
    }
    pub fn y_axis() -> Vector {
        Vector::new(0.0, 1.0, 0.0)
    }
    pub fn z_axis() -> Vector {
        Vector::new(0.0, 0.0, 1.0)
    }
    pub fn set(&mut self, pair: &CodePair) -> io::Result<()> {
        match pair.code {
            10 => self.x = double_value(&pair.value),
            20 => self.y = double_value(&pair.value),
            30 => self.z = double_value(&pair.value),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unexpected code for Vector: {}", pair.code))),
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                         Color
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    raw_value: i16,
}

impl Color {
    pub fn is_by_layer(&self) -> bool {
        self.raw_value == 256
    }
    pub fn is_by_entity(&self) -> bool {
        self.raw_value == 257
    }
    pub fn is_by_block(&self) -> bool {
        self.raw_value == 0
    }
    pub fn is_turned_off(&self) -> bool {
        self.raw_value < 0
    }
    pub fn set_by_layer(&mut self) {
        self.raw_value = 256
    }
    pub fn set_by_block(&mut self) {
        self.raw_value = 0
    }
    pub fn set_by_entity(&mut self) {
        self.raw_value = 257
    }
    pub fn turn_off(&mut self) {
        self.raw_value = -1
    }
    pub fn is_index(&self) -> bool {
        self.raw_value >= 1 && self.raw_value <= 255
    }
    pub fn index(&self) -> Option<u8> {
        if self.is_index() {
            Some(self.raw_value as u8)
        }
        else {
            None
        }
    }
    pub fn get_raw_value(&self) -> i16 {
        self.raw_value
    }
    pub fn from_raw_value(val: i16) -> Color {
        Color { raw_value: val }
    }
    pub fn by_layer() -> Color {
        Color { raw_value: 256 }
    }
    pub fn by_block() -> Color {
        Color { raw_value: 0 }
    }
    pub fn by_entity() -> Color {
        Color { raw_value: 257 }
    }
    pub fn from_index(i: u8) -> Color {
        Color { raw_value: i as i16 }
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                                    LineWeight
////////////////////////////////////////////////////////////////////////////////
pub struct LineWeight {
    raw_value: i16,
}

impl LineWeight {
    pub fn from_raw_value(v: i16) -> LineWeight {
        LineWeight { raw_value: v }
    }
    pub fn by_block() -> LineWeight {
        LineWeight::from_raw_value(-1)
    }
    pub fn by_layer() -> LineWeight {
        LineWeight::from_raw_value(-2)
    }
    pub fn get_raw_value(&self) -> i16 {
        self.raw_value
    }
}
