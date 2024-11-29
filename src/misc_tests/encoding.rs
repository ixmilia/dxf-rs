use crate::entities::*;
use crate::enums::*;
use crate::helper_functions::tests::*;
use crate::*;

use std::io::{BufReader, Cursor, Seek, SeekFrom};
use std::str::from_utf8;

use image::{DynamicImage, GenericImageView};

#[test]
fn read_string_with_control_characters() {
    let drawing = parse_drawing(
        [
            "0",
            "SECTION",
            "2",
            "HEADER",
            "9",
            "$LASTSAVEDBY",
            "1",
            "a^G^ ^^ b",
            "0",
            "ENDSEC",
            "0",
            "EOF",
        ]
        .join("\n")
        .as_str(),
    );
    assert_eq!("a\u{7}^\u{1E} b", drawing.header.last_saved_by);
}

#[test]
fn write_string_with_control_characters() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2004;
    drawing.header.last_saved_by = String::from("a\u{7}^\u{1E} b");
    assert_contains(&drawing, String::from("a^G^ ^^ b"));
}

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
        vec![
            CodePair::new_str(1, "garbage value 1"),
            CodePair::new_str(1, "garbage value 2"),
        ],
    );
}

#[test]
fn read_lf_and_crlf() {
    let code_pairs = [
        "0", "SECTION", "2", "HEADER", "9", "$ACADVER", "1", "AC1027", "0", "ENDSEC", "0", "EOF",
    ];

    let lf_file = parse_drawing(code_pairs.join("\n").as_str());
    assert_eq!(AcadVersion::R2013, lf_file.header.version);

    let crlf_file = parse_drawing(code_pairs.join("\r\n").as_str());
    assert_eq!(AcadVersion::R2013, crlf_file.header.version);
}

#[test]
fn read_file_with_comments() {
    let drawing = drawing_from_pairs(vec![
        CodePair::new_str(999, "comment"),
        CodePair::new_str(0, "SECTION"),
        CodePair::new_str(999, ""), // empty comment
        CodePair::new_str(2, "ENTITIES"),
        CodePair::new_str(0, "LINE"),
        CodePair::new_str(999, "comment"),
        CodePair::new_f64(10, 1.1),
        CodePair::new_str(999, "comment"),
        CodePair::new_f64(20, 2.2),
        CodePair::new_str(999, "comment"),
        CodePair::new_str(0, "ENDSEC"),
        CodePair::new_str(0, "EOF"),
        CodePair::new_str(9, "comment"),
    ]);
    let entities = drawing.entities().collect::<Vec<_>>();
    assert_eq!(1, entities.len());
    match entities[0].specific {
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
        vec![CodePair::new_str(9, "$DIMZIN"), CodePair::new_i16(70, 8)],
    );
    assert_eq!(
        UnitZeroSuppression::SuppressZeroFeetAndZeroInches,
        file.header.dimension_unit_zero_suppression
    );
}

#[test]
fn round_trip() {
    // drawing with one entity and one auto-added layer
    let mut drawing = Drawing::new();
    drawing.clear();
    drawing.add_entity(Entity {
        common: Default::default(),
        specific: EntityType::Line(Default::default()),
    });
    assert_eq!(1, drawing.entities().count());
    assert_eq!(1, drawing.layers().count());

    // ensure they're still there
    let drawing = drawing_from_pairs(drawing.code_pairs().unwrap());
    assert_eq!(1, drawing.entities().count());
    assert_eq!(1, drawing.layers().count());
}

#[test]
fn parse_with_leading_bom() {
    let buf = vec![
        0xEFu8, 0xBB, 0xBF, // UTF-8 byte representation of BOM
        b'0', b'\n', b'E', b'O', b'F',
    ];
    let _drawing = Drawing::load(&mut buf.as_slice());
}

#[test]
fn parse_with_bom_like_in_the_middle() {
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
    bytes.push(0xEF); // these three bytes represent the character `ｱ` in UTF8
    bytes.push(0xBD);
    bytes.push(0xB1);
    bytes.push(b'\r');
    bytes.push(b'\n');
    for b in tail.as_bytes() {
        bytes.push(*b);
    }
    let mut bytes = bytes.as_slice();
    let drawing = unwrap_drawing(Drawing::load_with_encoding(&mut bytes, encoding_rs::UTF_8));
    assert_eq!("ｱ", drawing.header.project_name);
}

#[test]
fn parse_as_ascii_text() {
    // if version <= R2004 (AC1018) stream is ASCII
    let drawing = parse_drawing(
        r"
  0
SECTION
  2
HEADER
  9
$ACADVER
  1
AC1018
  9
$PROJECTNAME
  1
\U+00E8
  0
ENDSEC
  0
EOF
"
        .trim(),
    );
    assert_eq!("è", drawing.header.project_name);
}

#[test]
fn parse_as_utf8_text() {
    // if version >= R2007 (AC1021) stream is UTF-8
    let mut bytes = vec![];
    bytes.extend_from_slice(
        r"
  0
SECTION
  2
HEADER
  9
$ACADVER
  1
AC1018
  9
$PROJECTNAME
  1"
        .trim()
        .as_bytes(),
    );
    bytes.push(b'\n');
    bytes.push(232); // è
    bytes.push(b'\n');
    bytes.extend_from_slice(
        r"
  0
ENDSEC
  0
EOF"
        .trim()
        .as_bytes(),
    );
    let drawing = Drawing::load(&mut bytes.as_slice()).ok().unwrap();
    assert_eq!("è", drawing.header.project_name);
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
    let drawing = unwrap_drawing(Drawing::load_file("./src/misc_tests/diamond-bin.dxf"));
    let entities = drawing.entities().collect::<Vec<_>>();
    assert_eq!(12, entities.len());
    match entities[0].specific {
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
    for version in &[AcadVersion::R12, AcadVersion::R13] {
        let mut drawing = Drawing::new();
        drawing.header.version = *version;
        let line = Line {
            p1: Point::new(1.1, 2.2, 3.3),
            p2: Point::new(4.4, 5.5, 6.6),
            ..Default::default()
        };
        drawing.add_entity(Entity::new(EntityType::Line(line)));
        let mut buf = Cursor::new(vec![]);
        drawing.save_binary(&mut buf).ok().unwrap();
        buf.seek(SeekFrom::Start(0)).ok().unwrap();
        let mut reader = BufReader::new(&mut buf);
        let drawing = unwrap_drawing(Drawing::load(&mut reader));
        let entities = drawing.entities().collect::<Vec<_>>();
        assert_eq!(1, entities.len());
        match entities[0].specific {
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
        b'A', b'u', b't', b'o', b'C', b'A', b'D', b' ', b'D', b'X', b'B', b' ', b'1', b'.', b'0',
        b'\r', b'\n', 0x1A, 0x0, // color
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
    let entities = drawing.entities().collect::<Vec<_>>();
    assert_eq!(1, entities.len());
    assert_eq!(Some(1), entities[0].common.color.index());
    match entities[0].specific {
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
        b'A', b'u', b't', b'o', b'C', b'A', b'D', b' ', b'D', b'X', b'B', b' ', b'1', b'.', b'0',
        b'\r', b'\n', 0x1A, 0x0, 19, // polyline
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
    let entities = drawing.entities().collect::<Vec<_>>();
    assert_eq!(1, entities.len());
    match entities[0].specific {
        EntityType::Polyline(ref poly) => {
            let vertices = poly.vertices().collect::<Vec<_>>();
            assert_eq!(2, vertices.len());
            assert_eq!(Point::new(1.0, 2.0, 0.0), vertices[0].location);
            assert_eq!(Point::new(3.0, 4.0, 0.0), vertices[1].location);
        }
        _ => panic!("expected a polyline"),
    }
}

#[test]
fn read_dxb_after_writing() {
    let mut drawing = Drawing::new();
    let line = Line::new(Point::new(1.0, 2.0, 3.0), Point::new(4.0, 5.0, 6.0));
    drawing.add_entity(Entity::new(EntityType::Line(line)));
    let mut buf = Cursor::new(vec![]);
    drawing.save_dxb(&mut buf).ok().unwrap();
    buf.seek(SeekFrom::Start(0)).ok().unwrap();
    let mut reader = BufReader::new(&mut buf);
    let drawing = unwrap_drawing(Drawing::load(&mut reader));
    let entities = drawing.entities().collect::<Vec<_>>();
    assert_eq!(1, entities.len());
    match entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.0, 2.0, 3.0), line.p1);
            assert_eq!(Point::new(4.0, 5.0, 6.0), line.p2);
        }
        _ => panic!("expected a line"),
    }
}

#[test]
fn dont_write_utf8_bom() {
    let drawing = Drawing::new();
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
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2004;
    drawing.header.project_name = String::from("è");
    assert_contains(
        &drawing,
        ["  9", "$PROJECTNAME", "  1", "\\U+00E8"].join("\r\n"),
    );
}

#[test]
fn write_unicode_as_utf8() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2007;
    drawing.header.project_name = String::from("è");
    assert_contains(&drawing, ["  9", "$PROJECTNAME", "  1", "è"].join("\r\n"));
}

#[test]
fn write_binary_file() {
    for version in &[AcadVersion::R12, AcadVersion::R13] {
        println!("checking version {:?}", version);
        let mut drawing = Drawing::new();
        drawing.header.version = *version;
        let buf = to_binary(&drawing);

        // check binary sentinel
        let sentinel = from_utf8(&buf[0..20]).ok().unwrap();
        assert_eq!("AutoCAD Binary DXF\r\n", sentinel);

        // check "SECTION" text at expected offset
        let sec_offset = if *version <= AcadVersion::R12 { 23 } else { 24 };
        let sec_end = sec_offset + 7;
        let sec_text = from_utf8(&buf[sec_offset..sec_end]).ok().unwrap();
        assert_eq!("SECTION", sec_text);
    }
}

#[test]
fn thumbnail_round_trip_rgb8() {
    // 1x1 px image, red pixel
    let mut imgbuf = image::ImageBuffer::new(1, 1);
    imgbuf.put_pixel(0, 0, image::Rgb([255u8, 0, 0]));
    let thumbnail = DynamicImage::ImageRgb8(imgbuf);

    let round_tripped = round_trip_thumbnail(thumbnail);
    assert_eq!((1, 1), round_tripped.dimensions());
    assert_eq!(
        image::Rgba([255u8, 0, 0, 255]),
        round_tripped.get_pixel(0, 0)
    );
}

#[test]
fn thumbnail_round_trip_grayscale() {
    // 1x1 px grayscale image, white
    let mut imgbuf = image::ImageBuffer::new(1, 1);
    imgbuf.put_pixel(0, 0, image::Luma([255]));
    let thumbnail = DynamicImage::ImageLuma8(imgbuf);

    let round_tripped = round_trip_thumbnail(thumbnail);
    assert_eq!((1, 1), round_tripped.dimensions());
    assert_eq!(
        image::Rgba([255u8, 255, 255, 255]),
        round_tripped.get_pixel(0, 0)
    ); // it comes back as RGB
}

fn round_trip_thumbnail(thumbnail: image::DynamicImage) -> image::DynamicImage {
    // write drawing with thumbnail
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2000; // thumbnails are only written >= R2000
    drawing.thumbnail = Some(thumbnail);

    let drawing_pairs = drawing.code_pairs().unwrap();
    assert_vec_contains(
        &drawing_pairs,
        &[
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "THUMBNAILIMAGE"),
        ],
    );

    let drawing = drawing_from_pairs(drawing_pairs);
    drawing.thumbnail.unwrap()
}
