// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

use self::dxf::enums::*;
use self::dxf::*;
use std::io::{Cursor, Seek, SeekFrom};

mod test_helpers;
use test_helpers::helpers::*;

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
