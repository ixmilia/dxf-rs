// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

use self::dxf::*;
use self::dxf::enums::*;
use self::dxf::entities::*;

mod test_helpers;
use test_helpers::helpers::*;

#[test]
fn totally_empty_file() {
    let _file = parse_drawing("");
}

#[test]
fn empty_file_trailing_newline() {
    let _file = parse_drawing("0\nEOF\n");
}

#[test]
fn empty_file_no_trailing_newline() {
    let _file = parse_drawing("0\nEOF");
}

#[test]
fn unsupported_section() {
    let _file = from_section("UNSUPPORTED_SECTION", vec!["1", "garbage value 1", "2", "garbage value 2"].join("\n").as_str());
}

#[test]
fn read_lf_and_crlf() {
    let code_pairs = vec!["0", "SECTION", "2", "HEADER", "9", "$ACADVER", "1", "AC1027", "0", "ENDSEC", "0", "EOF"];

    let lf_file = parse_drawing(code_pairs.join("\n").as_str());
    assert_eq!(AcadVersion::R2013, lf_file.header.version);

    let crlf_file = parse_drawing(code_pairs.join("\r\n").as_str());
    assert_eq!(AcadVersion::R2013, crlf_file.header.version);
}

#[test]
fn read_file_with_comments() {
    let file = parse_drawing(vec![
        "999", "comment",
        "0", "SECTION",
            "999", "", // empty comment
            "2", "ENTITIES",
                "0", "LINE",
                "999", "comment",
                "10", "1.1",
                "999", "comment",
                "20", "2.2",
                "999", "comment",
            "0", "ENDSEC",
        "0", "EOF",
        "999", "comment",
    ].join("\r\n").trim());
    assert_eq!(1, file.entities.len());
    match file.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.1, 2.2, 0.0), line.p1);
        },
        _ => panic!("expected a LINE"),
    }
}

#[test]
fn round_trip() {
    // drawing with one entity and one layer
    let mut drawing = Drawing::new();
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Line(Default::default()),
    });
    drawing.layers.push(Default::default());

    // ensure they're still there
    let drawing = parse_drawing(&to_test_string(&drawing));
    assert_eq!(1, drawing.entities.len());
    assert_eq!(1, drawing.layers.len());
}

#[test]
fn parse_with_leading_bom() {
    let buf = vec![
        0xFEu8, 0xFF, // UTF-8 representation of BOM
        '0' as u8,
        '\n' as u8,
        'E' as u8, 'O' as u8, 'F' as u8,
    ];
    let _drawing = Drawing::load(buf.as_slice());
}
