// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Write;
use itertools::PutBack;

use ::{
    CodePair,
    DxfError,
    DxfResult,
    Point,
    Vector,
};

use ::enums::AcadVersion;
use ::helper_functions::*;
use ::code_pair_writer::CodePairWriter;

#[doc(hidden)] pub const XDATA_APPLICATIONNAME: i32 = 1001;
const XDATA_STRING: i32 = 1000;
const XDATA_CONTROLGROUP: i32 = 1002;
const XDATA_LAYER: i32 = 1003;
const XDATA_BINARYDATA: i32 = 1004;
const XDATA_HANDLE: i32 = 1005;
const XDATA_THREEREALS: i32 = 1010;
const XDATA_WORLDSPACEPOSITION: i32 = 1011;
const XDATA_WORLDSPACEDISPLACEMENT: i32 = 1012;
const XDATA_WORLDDIRECTION: i32 = 1013;
const XDATA_REAL: i32 = 1040;
const XDATA_DISTANCE: i32 = 1041;
const XDATA_SCALEFACTOR: i32 = 1042;
const XDATA_INTEGER: i32 = 1070;
const XDATA_LONG: i32 = 1071;

/// Represents an application name and a collection of extended data.
#[derive(Clone, Debug)]
pub struct XData {
    pub application_name: String,
    pub items: Vec<XDataItem>,
}

/// Represents a piece of extended data.
#[derive(Clone, Debug, PartialEq)]
pub enum XDataItem {
    Str(String),
    ControlGroup(Vec<XDataItem>),
    LayerName(String),
    BinaryData(Vec<u8>),
    Handle(u32),
    ThreeReals(f64, f64, f64),
    WorldSpacePosition(Point),
    WorldSpaceDisplacement(Point),
    WorldDirection(Vector),
    Real(f64),
    Distance(f64),
    ScaleFactor(f64),
    Integer(i16),
    Long(i32),
}

impl XData {
    #[doc(hidden)]
    pub fn read_item<I>(application_name: String, iter: &mut PutBack<I>) -> DxfResult<XData>
        where I: Iterator<Item = DxfResult<CodePair>> {

        let mut xdata = XData { application_name: application_name, items: vec![] };
        loop {
            let pair = match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    iter.put_back(Ok(pair));
                    return Ok(xdata);
                },
                Some(Ok(pair)) => pair,
                Some(Err(e)) => return Err(e),
                None => return Ok(xdata),
            };
            if pair.code == XDATA_APPLICATIONNAME || pair.code < XDATA_STRING {
                // new xdata or non xdata
                break;
            }
            xdata.items.push(try!(XDataItem::read_item(&pair, iter)));
        }
        Ok(xdata)
    }
    #[doc(hidden)]
    pub fn write<T>(&self, version: &AcadVersion, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        // not supported on < R2000
        if version >= &AcadVersion::R2000 {
            try!(writer.write_code_pair(&CodePair::new_string(XDATA_APPLICATIONNAME, &self.application_name)));
            for item in &self.items {
                try!(item.write(writer));
            }
        }
        Ok(())
    }
}

impl XDataItem {
    #[doc(hidden)]
    fn read_item<I>(pair: &CodePair, iter: &mut PutBack<I>) -> DxfResult<XDataItem>
        where I: Iterator<Item = DxfResult<CodePair>> {

        loop {
            match pair.code {
                XDATA_STRING => return Ok(XDataItem::Str(try!(pair.value.assert_string()))),
                XDATA_CONTROLGROUP => {
                    let mut items = vec![];
                    loop {
                        let pair = match iter.next() {
                            Some(Ok(pair)) => {
                                if pair.code < XDATA_STRING {
                                    return Err(DxfError::UnexpectedCodePair(pair, String::from("expected XDATA item")));
                                }
                                pair
                            }
                            Some(Err(e)) => return Err(e),
                            None => return Err(DxfError::UnexpectedEndOfInput),
                        };
                        if pair.code == XDATA_CONTROLGROUP && try!(pair.value.assert_string()) == "}" {
                            // end of group
                            break;
                        }

                        items.push(try!(XDataItem::read_item(&pair, iter)));
                    }
                    return Ok(XDataItem::ControlGroup(items));
                },
                XDATA_LAYER => return Ok(XDataItem::LayerName(try!(pair.value.assert_string()))),
                XDATA_BINARYDATA => {
                    let mut data = vec![];
                    try!(parse_hex_string(&try!(pair.value.assert_string()), &mut data));
                    return Ok(XDataItem::BinaryData(data));
                },
                XDATA_HANDLE => return Ok(XDataItem::Handle(try!(as_u32(try!(pair.value.assert_string()))))),
                XDATA_THREEREALS => return Ok(XDataItem::ThreeReals(try!(pair.value.assert_f64()), try!(XDataItem::read_double(iter, pair.code)), try!(XDataItem::read_double(iter, pair.code)))),
                XDATA_WORLDSPACEDISPLACEMENT => return Ok(XDataItem::WorldSpaceDisplacement(try!(XDataItem::read_point(iter, try!(pair.value.assert_f64()), pair.code)))),
                XDATA_WORLDSPACEPOSITION => return Ok(XDataItem::WorldSpacePosition(try!(XDataItem::read_point(iter, try!(pair.value.assert_f64()), pair.code)))),
                XDATA_WORLDDIRECTION => return Ok(XDataItem::WorldDirection(try!(XDataItem::read_vector(iter, try!(pair.value.assert_f64()), pair.code)))),
                XDATA_REAL => return Ok(XDataItem::Real(try!(pair.value.assert_f64()))),
                XDATA_DISTANCE => return Ok(XDataItem::Distance(try!(pair.value.assert_f64()))),
                XDATA_SCALEFACTOR => return Ok(XDataItem::ScaleFactor(try!(pair.value.assert_f64()))),
                XDATA_INTEGER => return Ok(XDataItem::Integer(try!(pair.value.assert_i16()))),
                XDATA_LONG => return Ok(XDataItem::Long(try!(pair.value.assert_i32()))),
                _ => return Err(DxfError::UnexpectedCode(pair.code)),
            }
        }
    }
    fn read_double<T>(iter: &mut PutBack<T>, expected_code: i32) -> DxfResult<f64>
        where T: Iterator<Item = DxfResult<CodePair>> {

        match iter.next() {
            Some(Ok(ref pair)) if pair.code == expected_code => return Ok(try!(pair.value.assert_f64())),
            Some(Ok(pair)) => return Err(DxfError::UnexpectedCode(pair.code)),
            Some(Err(e)) => return Err(e),
            None => return Err(DxfError::UnexpectedEndOfInput),
        }
    }
    fn read_point<I>(iter: &mut PutBack<I>, first: f64, expected_code: i32) -> DxfResult<Point>
        where I: Iterator<Item = DxfResult<CodePair>> {

        Ok(Point::new(first, try!(XDataItem::read_double(iter, expected_code)), try!(XDataItem::read_double(iter, expected_code))))
    }
    fn read_vector<I>(iter: &mut PutBack<I>, first: f64, expected_code: i32) -> DxfResult<Vector>
        where I: Iterator<Item = DxfResult<CodePair>> {

        Ok(Vector::new(first, try!(XDataItem::read_double(iter, expected_code)), try!(XDataItem::read_double(iter, expected_code))))
    }
    #[doc(hidden)]
    pub fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        match self {
            &XDataItem::Str(ref s) => { try!(writer.write_code_pair(&CodePair::new_string(XDATA_STRING, s))); },
            &XDataItem::ControlGroup(ref items) => {
                try!(writer.write_code_pair(&CodePair::new_str(XDATA_CONTROLGROUP, "{")));
                for item in &items[..] {
                    try!(item.write(writer));
                }
                try!(writer.write_code_pair(&CodePair::new_str(XDATA_CONTROLGROUP, "}")));
            },
            &XDataItem::LayerName(ref l) => { try!(writer.write_code_pair(&CodePair::new_string(XDATA_LAYER, l))); },
            &XDataItem::BinaryData(ref data) => {
                let mut line = String::new();
                for b in data {
                    line.push_str(&format!("{:X}", b));
                }
                try!(writer.write_code_pair(&CodePair::new_string(XDATA_BINARYDATA, &line)));
            },
            &XDataItem::Handle(h) => { try!(writer.write_code_pair(&CodePair::new_string(XDATA_HANDLE, &as_handle(h)))); },
            &XDataItem::ThreeReals(x, y, z) => {
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_THREEREALS, x)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_THREEREALS, y)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_THREEREALS, z)));
            },
            &XDataItem::WorldSpacePosition(ref p) => {
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEPOSITION, p.x)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEPOSITION, p.y)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEPOSITION, p.z)));
            },
            &XDataItem::WorldSpaceDisplacement(ref p) => {
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT, p.x)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT, p.y)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT, p.z)));
            },
            &XDataItem::WorldDirection(ref v) => {
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDDIRECTION, v.x)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDDIRECTION, v.y)));
                try!(writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDDIRECTION, v.z)));
            },
            &XDataItem::Real(f) => { try!(writer.write_code_pair(&CodePair::new_f64(XDATA_REAL, f))); },
            &XDataItem::Distance(f) => { try!(writer.write_code_pair(&CodePair::new_f64(XDATA_DISTANCE, f))); },
            &XDataItem::ScaleFactor(f) => { try!(writer.write_code_pair(&CodePair::new_f64(XDATA_SCALEFACTOR, f))); },
            &XDataItem::Integer(i) => { try!(writer.write_code_pair(&CodePair::new_i16(XDATA_INTEGER, i))); },
            &XDataItem::Long(i) => { try!(writer.write_code_pair(&CodePair::new_i32(XDATA_LONG, i))); },
        }
        Ok(())
    }
}
