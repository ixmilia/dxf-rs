// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

extern crate encoding_rs;

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
        0xEFu8, 0xBB, 0xBF, // UTF-8 byte representation of BOM
        '0' as u8, '\n' as u8, 'E' as u8, 'O' as u8, 'F' as u8,
    ];
    let _drawing = Drawing::load(&mut buf.as_slice());
}

#[test]
fn parse_as_ascii_text() {
    // if version <= R2004 (AC1018) stream is ASCII
    let file = from_section(
        "HEADER",
        "
  9
$ACADVER
  1
AC1018
  9
$PROJECTNAME
  1
\\U+00E8"
            .trim_start(),
    );
    assert_eq!("è", file.header.project_name);
}

#[test]
fn parse_as_utf8_text() {
    // if version >= R2007 (AC1021) stream is UTF-8
    let file = from_section(
        "HEADER",
        "
  9
$ACADVER
  1
AC1021
  9
$PROJECTNAME
  1
è"
        .trim_start(),
    );
    assert_eq!("è", file.header.project_name);
}

#[test]
fn read_with_alternate_encoding() {
    let head = "
  0
SECTION
  2
HEADER
  9
$PROJECTNAME
  1"
    .trim();
    let tail = "
  0
ENDSEC
  0
EOF"
    .trim();
    let mut bytes = head.as_bytes().to_vec();
    bytes.push(b'\r');
    bytes.push(b'\n');
    bytes.push(0xB2); // these two bytes represent the character `不` in GB18030 encoding
    bytes.push(0xBB);
    bytes.push(b'\r');
    bytes.push(b'\n');
    for b in tail.as_bytes() {
        bytes.push(*b);
    }
    let mut bytes = bytes.as_slice();
    let drawing = unwrap_drawing(Drawing::load_with_encoding(
        &mut bytes,
        encoding_rs::GB18030,
    ));
    assert_eq!("不", drawing.header.project_name);
}

#[test]
fn read_binary_file() {
    // `diamond-bin.dxf` is a pre-R13 binary file
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
fn read_binary_file_post_r13() {
    // post R13 binary files have 2 byte codes and single byte booleans
    let data = vec![
        // binary header
        b'A', b'u', b't', b'o', b'C', b'A', b'D', b' ', b'B', b'i', b'n', b'a', b'r', b'y', b' ',
        b'D', b'X', b'F', b'\r', b'\n', 0x1A, 0x00, // 0/SECTION
        0x00, 0x00, b'S', b'E', b'C', b'T', b'I', b'O', b'N', 0x00, // 2/HEADER
        0x02, 0x00, b'H', b'E', b'A', b'D', b'E', b'R', 0x00, // 9/$LWDISPLAY
        0x09, 0x00, b'$', b'L', b'W', b'D', b'I', b'S', b'P', b'L', b'A', b'Y', 0x00, 0x22, 0x01,
        0x01, // 290/true
        0x00, 0x00, b'E', b'N', b'D', b'S', b'E', b'C', 0x00, // 0/ENDSEC
        0x00, 0x00, b'E', b'O', b'F', 0x00, // 0/EOF
    ];
    let drawing = Drawing::load(&mut data.as_slice()).unwrap();
    assert!(drawing.header.display_linewieght_in_model_and_layout_tab);
}

#[test]
fn read_binary_file_after_writing() {
    for version in vec![AcadVersion::R12, AcadVersion::R13] {
        let mut drawing = Drawing::default();
        drawing.header.version = version;
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
