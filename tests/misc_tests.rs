// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::enums::*;

mod test_helpers;
use test_helpers::helpers::*;

#[test]
fn read_string_with_control_characters() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
        "2", "HEADER",
        "9", "$LASTSAVEDBY",
        "1", "a^G^ ^^ b",
        "0", "ENDSEC",
        "0", "EOF",
    ].join("\n").as_str());
    assert_eq!("a\u{7}^\u{1E} b", drawing.header.last_saved_by);
}

#[test]
fn write_string_with_control_characters() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2004;
    drawing.header.last_saved_by = String::from("a\u{7}^\u{1E} b");
    assert_contains(&drawing, String::from("a^G^ ^^ b"));
}
