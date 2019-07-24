// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

use std::io::{BufReader, Cursor, Seek, SeekFrom};

use self::dxf::entities::*;
use self::dxf::enums::*;
use self::dxf::*;

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
    let _file = from_section(
        "UNSUPPORTED_SECTION",
        vec!["1", "garbage value 1", "2", "garbage value 2"]
            .join("\n")
            .as_str(),
    );
}

#[test]
fn read_lf_and_crlf() {
    let code_pairs = vec![
        "0", "SECTION", "2", "HEADER", "9", "$ACADVER", "1", "AC1027", "0", "ENDSEC", "0", "EOF",
    ];

    let lf_file = parse_drawing(code_pairs.join("\n").as_str());
    assert_eq!(AcadVersion::R2013, lf_file.header.version);

    let crlf_file = parse_drawing(code_pairs.join("\r\n").as_str());
    assert_eq!(AcadVersion::R2013, crlf_file.header.version);
}

#[test]
fn read_file_with_comments() {
    let file = parse_drawing(
        vec![
            "999", "comment", "0", "SECTION", "999", "", // empty comment
            "2", "ENTITIES", "0", "LINE", "999", "comment", "10", "1.1", "999", "comment", "20",
            "2.2", "999", "comment", "0", "ENDSEC", "0", "EOF", "999", "comment",
        ]
        .join("\r\n")
        .trim(),
    );
    assert_eq!(1, file.entities.len());
    match file.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.1, 2.2, 0.0), line.p1);
        }
        _ => panic!("expected a LINE"),
    }
}

#[test]
fn enum_out_of_bounds() {
    let file = from_section(
        "HEADER",
        vec!["  9", "$DIMZIN", " 70", "     8"].join("\r\n").trim(),
    );
    assert_eq!(
        UnitZeroSuppression::SuppressZeroFeetAndZeroInches,
        file.header.dimension_unit_zero_suppression
    );
}

#[test]
fn round_trip() {
    // drawing with one entity and one layer
    let mut drawing = Drawing::default();
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
        '0' as u8, '\n' as u8, 'E' as u8, 'O' as u8, 'F' as u8,
    ];
    let _drawing = Drawing::load(&mut buf.as_slice());
}

#[test]
fn read_binary_file() {
    let drawing = unwrap_drawing(Drawing::load_file("./tests/diamond-bin.dxf"));
    assert_eq!(12, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(45.0, 45.0, 0.0), line.p1);
            assert_eq!(Point::new(45.0, -45.0, 0.0), line.p2);
        }
        _ => panic!("expected a line"),
    }
}

#[test]
fn read_binary_file_after_writing() {
    let mut drawing = Drawing::default();
    let line = Line {
        p1: Point::new(1.1, 2.2, 3.3),
        p2: Point::new(4.4, 5.5, 6.6),
        ..Default::default()
    };
    drawing.entities.push(Entity::new(EntityType::Line(line)));
    let mut buf = Cursor::new(vec![]);
    drawing.save_binary(&mut buf).ok().unwrap();
    buf.seek(SeekFrom::Start(0)).ok().unwrap();
    let mut reader = BufReader::new(&mut buf);
    let drawing = unwrap_drawing(Drawing::load(&mut reader));
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.1, 2.2, 3.3), line.p1);
            assert_eq!(Point::new(4.4, 5.5, 6.6), line.p2);
        }
        _ => panic!("expected a line"),
    }
}

#[test]
fn read_dxb_file() {
    let data = vec![
        // DXB sentinel "AutoCAD DXB 1.0\r\n"
        'A' as u8, 'u' as u8, 't' as u8, 'o' as u8, 'C' as u8, 'A' as u8, 'D' as u8, ' ' as u8,
        'D' as u8, 'X' as u8, 'B' as u8, ' ' as u8, '1' as u8, '.' as u8, '0' as u8, '\r' as u8,
        '\n' as u8, 0x1A, 0x0, // color
        136, // type specifier for new color
        0x01, 0x00, // color index 1
        // line
        0x01, // type specifier
        0x01, 0x00, // p1.x = 0x0001
        0x02, 0x00, // p1.y = 0x0002
        0x03, 0x00, // p1.z = 0x0003
        0x04, 0x00, // p2.x = 0x0004
        0x05, 0x00, // p2.y = 0x0005
        0x06, 0x00, // p2.z = 0x0006
        0x0,  // null terminator
    ];
    let drawing = Drawing::load(&mut data.as_slice()).unwrap();
    assert_eq!(1, drawing.entities.len());
    assert_eq!(Some(1), drawing.entities[0].common.color.index());
    match drawing.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.0, 2.0, 3.0), line.p1);
            assert_eq!(Point::new(4.0, 5.0, 6.0), line.p2);
        }
        _ => panic!("expected a line"),
    }
}

#[test]
fn read_dxb_file_with_polyline() {
    let data = vec![
        // DXB sentinel "AutoCAD DXB 1.0\r\n"
        'A' as u8, 'u' as u8, 't' as u8, 'o' as u8, 'C' as u8, 'A' as u8, 'D' as u8, ' ' as u8,
        'D' as u8, 'X' as u8, 'B' as u8, ' ' as u8, '1' as u8, '.' as u8, '0' as u8, '\r' as u8,
        '\n' as u8, 0x1A, 0x0, 19, // polyline
        0x00, 0x00, // is closed = false
        20,   // vertex
        0x01, 0x00, // x
        0x02, 0x00, // y
        20,   // vertex
        0x03, 0x00, // x
        0x04, 0x00, // y
        17,   // seqend
        0x0,  // null terminator
    ];
    let drawing = Drawing::load(&mut data.as_slice()).unwrap();
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => {
            assert_eq!(2, poly.vertices.len());
            assert_eq!(Point::new(1.0, 2.0, 0.0), poly.vertices[0].location);
            assert_eq!(Point::new(3.0, 4.0, 0.0), poly.vertices[1].location);
        }
        _ => panic!("expected a polyline"),
    }
}

#[test]
fn read_dxb_after_writing() {
    let mut drawing = Drawing::default();
    let line = Line::new(Point::new(1.0, 2.0, 3.0), Point::new(4.0, 5.0, 6.0));
    drawing.entities.push(Entity::new(EntityType::Line(line)));
    let mut buf = Cursor::new(vec![]);
    drawing.save_dxb(&mut buf).ok().unwrap();
    buf.seek(SeekFrom::Start(0)).ok().unwrap();
    let mut reader = BufReader::new(&mut buf);
    let drawing = unwrap_drawing(Drawing::load(&mut reader));
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.0, 2.0, 3.0), line.p1);
            assert_eq!(Point::new(4.0, 5.0, 6.0), line.p2);
        }
        _ => panic!("expected a line"),
    }
}
