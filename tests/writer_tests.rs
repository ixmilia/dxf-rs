// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

use self::dxf::enums::*;
use self::dxf::*;
use std::io::{Cursor, Seek, SeekFrom};
use std::str::from_utf8;

mod test_helpers;
use crate::test_helpers::helpers::*;

#[test]
fn dont_write_utf8_bom() {
    let drawing = Drawing::default();
    let mut buf = Cursor::new(vec![]);
    drawing.save(&mut buf).ok().unwrap();
    buf.seek(SeekFrom::Start(0)).ok().unwrap();
    let vec = buf.into_inner();

    // file should start directly with a code, not a UTF-8 BOM
    assert_eq!(b' ', vec[0]);
    assert_eq!(b' ', vec[1]);
    assert_eq!(b'0', vec[2]);
}

#[test]
fn write_unicode_as_ascii() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2004;
    drawing.header.project_name = String::from("è");
    assert_contains(
        &drawing,
        vec!["  9", "$PROJECTNAME", "  1", "\\U+00E8"].join("\r\n"),
    );
}

#[test]
fn write_unicode_as_utf8() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2007;
    drawing.header.project_name = String::from("è");
    assert_contains(
        &drawing,
        vec!["  9", "$PROJECTNAME", "  1", "è"].join("\r\n"),
    );
}

#[test]
fn write_binary_file() {
    for version in vec![AcadVersion::R12, AcadVersion::R13] {
        println!("checking version {:?}", version);
        let mut drawing = Drawing::default();
        drawing.header.version = version;
        let buf = to_binary(&drawing);

        // check binary sentinel
        let sentinel = from_utf8(&buf[0..20]).ok().unwrap();
        assert_eq!("AutoCAD Binary DXF\r\n", sentinel);

        // check "SECTION" text at expected offset
        let sec_offset = if version <= AcadVersion::R12 { 23 } else { 24 };
        let sec_end = sec_offset + 7;
        let sec_text = from_utf8(&buf[sec_offset..sec_end]).ok().unwrap();
        assert_eq!("SECTION", sec_text);
    }
}
