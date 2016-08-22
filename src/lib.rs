// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

#[macro_use] extern crate enum_primitive;

pub mod enums;
pub mod header;
pub mod entities;

use self::header::*;
use self::entities::*;

use self::enums::*;
use enum_primitive::FromPrimitive;

use std::io;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::iter::Peekable;

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
    pub fn new_bool(code: i32, val: bool) -> CodePair {
        CodePair::new(code, CodePairValue::Boolean(val))
    }
}

////////////////////////////////////////////////////////////////////////////////
//                                                             CodePairAsciiIter
////////////////////////////////////////////////////////////////////////////////
struct CodePairAsciiIter<T>
    where T: BufRead
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

impl<T: BufRead> Iterator for CodePairAsciiIter<T> {
    type Item = io::Result<CodePair>;
    fn next(&mut self) -> Option<io::Result<CodePair>> {
        // Read code.  If no line is available, fail gracefully.
        let mut code_line = String::new();
        match self.reader.read_line(&mut code_line) {
            Ok(_) => (),
            Err(_) => return None,
        }
        let code_line = code_line.trim();
        if code_line.is_empty() { return None; }
        let code = try_option!(code_line.parse::<i32>());

        // Read value.  If no line is available die horribly.
        let mut value_line = String::new();
        try_option!(self.reader.read_line(&mut value_line));
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
        }

        Ok(Some(entity))
    }
    fn apply_code_pair(&mut self, pair: &CodePair) -> io::Result<()> {
        if !try!(self.specific.try_apply_code_pair(&pair)) {
            try!(self.common.apply_individual_pair(&pair));
        }
        Ok(())
    }
    fn apply_custom_reader<I>(&mut self, peekable: &mut Peekable<I>) -> io::Result<bool>
        where I: Iterator<Item = io::Result<CodePair>>
    {
        match self.specific {
            EntityType::Image { ref mut class_version, ref mut location, ref mut u_vector, ref mut v_vector, ref mut image_size, ref mut image_def_reference, ref mut display_options_flags, ref mut use_clipping, ref mut brightness, ref mut contrast, ref mut fade, ref mut image_def_reactor_reference, ref mut clipping_type, ref mut clipping_vertex_count, ref mut clipping_vertices, ref mut is_inside_clipping } => {
                loop {
                    let pair = next_pair!(peekable);
                    match pair.code {
                        90 => { *class_version = int_value(&pair.value); },
                        10 => { location.x = double_value(&pair.value); },
                        20 => { location.y = double_value(&pair.value); },
                        30 => { location.z = double_value(&pair.value); },
                        11 => { u_vector.x = double_value(&pair.value); },
                        21 => { u_vector.y = double_value(&pair.value); },
                        31 => { u_vector.z = double_value(&pair.value); },
                        12 => { v_vector.x = double_value(&pair.value); },
                        22 => { v_vector.y = double_value(&pair.value); },
                        32 => { v_vector.z = double_value(&pair.value); },
                        13 => { image_size.x = double_value(&pair.value); },
                        23 => { image_size.y = double_value(&pair.value); },
                        340 => { *image_def_reference = string_value(&pair.value); },
                        70 => { *display_options_flags = short_value(&pair.value) as i32; },
                        280 => { *use_clipping = as_bool(short_value(&pair.value)); },
                        281 => { *brightness = short_value(&pair.value); },
                        282 => { *contrast = short_value(&pair.value); },
                        283 => { *fade = short_value(&pair.value); },
                        360 => { *image_def_reactor_reference = string_value(&pair.value); },
                        71 => { *clipping_type = try_result!(ImageClippingBoundaryType::from_i16(short_value(&pair.value))); },
                        91 => { *clipping_vertex_count = int_value(&pair.value); },
                        14 => {
                            // add new clipping vertex x value
                            clipping_vertices.push(Point::new(double_value(&pair.value), 0.0, 0.0));
                        },
                        24 => {
                            // append existing clipping vertex y value
                            let last = clipping_vertices.len(); // TODO: handle index out of bounds
                            clipping_vertices[last - 1].y = double_value(&pair.value);
                        }
                        290 => { *is_inside_clipping = bool_value(&pair.value); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            EntityType::MText { ref mut insertion_point, ref mut initial_text_height, ref mut reference_rectangle_width, ref mut attachment_point, ref mut drawing_direction, ref mut extended_text, ref mut text, ref mut text_style_name, ref mut extrusion_direction, ref mut x_axis_direction, ref mut horizontal_width, ref mut vertical_height, ref mut rotation_angle, ref mut line_spacing_style, ref mut line_spacing_factor, ref mut background_fill_setting, ref mut background_color_r_g_b, ref mut background_color_name, ref mut fill_box_scale, ref mut background_fill_color, ref mut background_fill_color_transparency, ref mut column_type, ref mut column_count, ref mut is_column_flow_reversed, ref mut is_column_auto_height, ref mut column_width, ref mut column_gutter, ref mut column_heights } => {
                let mut reading_column_data = false;
                let mut read_column_count = false;
                loop {
                    let pair = next_pair!(peekable);
                    match pair.code {
                        10 => { insertion_point.x = double_value(&pair.value); },
                        20 => { insertion_point.y = double_value(&pair.value); },
                        30 => { insertion_point.z = double_value(&pair.value); },
                        40 => { *initial_text_height = double_value(&pair.value); },
                        41 => { *reference_rectangle_width = double_value(&pair.value); },
                        71 => { *attachment_point = try_result!(AttachmentPoint::from_i16(short_value(&pair.value))); },
                        72 => { *drawing_direction = try_result!(DrawingDirection::from_i16(short_value(&pair.value))); },
                        3 => { extended_text.push(string_value(&pair.value)); },
                        1 => { *text = string_value(&pair.value); },
                        7 => { *text_style_name = string_value(&pair.value); },
                        210 => { extrusion_direction.x = double_value(&pair.value); },
                        220 => { extrusion_direction.y = double_value(&pair.value); },
                        230 => { extrusion_direction.z = double_value(&pair.value); },
                        11 => { x_axis_direction.x = double_value(&pair.value); },
                        21 => { x_axis_direction.y = double_value(&pair.value); },
                        31 => { x_axis_direction.z = double_value(&pair.value); },
                        42 => { *horizontal_width = double_value(&pair.value); },
                        43 => { *vertical_height = double_value(&pair.value); },
                        50 => {
                            if reading_column_data {
                                if read_column_count {
                                    column_heights.push(double_value(&pair.value));
                                }
                                else {
                                    *column_count = double_value(&pair.value) as i32;
                                    read_column_count = true;
                                }
                            }
                            else {
                                *rotation_angle = double_value(&pair.value);
                            }
                        },
                        73 => { *line_spacing_style = try_result!(MTextLineSpacingStyle::from_i16(short_value(&pair.value))); },
                        44 => { *line_spacing_factor = double_value(&pair.value); },
                        90 => { *background_fill_setting = try_result!(BackgroundFillSetting::from_i32(int_value(&pair.value))); },
                        420 => { *background_color_r_g_b = int_value(&pair.value); },
                        430 => { *background_color_name = string_value(&pair.value); },
                        45 => { *fill_box_scale = double_value(&pair.value); },
                        63 => { *background_fill_color = Color::from_raw_value(short_value(&pair.value)); },
                        441 => { *background_fill_color_transparency = int_value(&pair.value); },
                        75 => {
                            *column_type = short_value(&pair.value);
                            reading_column_data = true;
                        },
                        76 => { *column_count = short_value(&pair.value) as i32; },
                        78 => { *is_column_flow_reversed = as_bool(short_value(&pair.value)); },
                        79 => { *is_column_auto_height = as_bool(short_value(&pair.value)); },
                        48 => { *column_width = double_value(&pair.value); },
                        49 => { *column_gutter = double_value(&pair.value); },
                        _ => { try!(self.common.apply_individual_pair(&pair)); },
                    }
                }
            },
            _ => return Ok(false), // no custom reader
        }

        Ok(true)
    }
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

impl Drawing {
    pub fn new() -> Self {
        Drawing {
            header: Header::new(),
            entities: vec![],
        }
    }
    pub fn read<T>(reader: &mut T) -> io::Result<Drawing>
        where T: Read
    {
        let buf_reader = BufReader::new(reader);
        Drawing::load(buf_reader)
    }
    pub fn load<T>(reader: T) -> io::Result<Drawing>
        where T: BufRead {
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
    pub fn parse(s: &str) -> io::Result<Drawing> {
        let data = String::from(s);
        let bytes = data.as_bytes();
        Drawing::load(bytes)
    }
    pub fn write<T>(&self, writer: &mut T) -> io::Result<()>
        where T: Write {
        let mut writer = CodePairAsciiWriter { writer: writer };
        try!(self.header.write(&mut writer));
        // TODO: write other sections
        try!(writer.write_code_pair(&CodePair::new_str(0, "EOF")));
        Ok(())
    }
    pub fn to_string(&self) -> io::Result<String> {
        use std::io::Cursor;
        let mut buf = Cursor::new(vec![]);
        try!(self.write(&mut buf));
        try!(buf.seek(SeekFrom::Start(0)));
        let reader = BufReader::new(&mut buf);
        Ok(reader.lines().map(|l| l.unwrap() + "\r\n").collect())
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
                                "ENTITIES" => {
                                    loop {
                                        match try!(Entity::read(peekable)) {
                                            Some(e) => drawing.entities.push(e),
                                            None => break,
                                        }
                                    }
                                },
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
#[derive(Clone)]
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
