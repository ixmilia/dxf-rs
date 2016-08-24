// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::entities::*;
use self::dxf::enums::*;

mod test_helpers;
use test_helpers::helpers::*;

fn read_entity(entity_type: &str, body: String) -> Entity {
    let file = from_section("ENTITIES", vec!["0", entity_type, body.as_str()].join("\r\n").as_str());
    assert_eq!(1, file.entities.len());
    file.entities[0].to_owned()
}

#[test]
fn read_empty_entities_section() {
    let file = parse_drawing(vec!["0", "SECTION", "2", "ENTITIES", "0", "ENDSEC", "0", "EOF"].join("\r\n").as_str());
    assert_eq!(0, file.entities.len());
}

#[test]
fn read_unsupported_entity() {
    let file = parse_drawing(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "UNSUPPORTED_ENTITY",
                    "1", "unsupported string",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(0, file.entities.len());
}

#[test]
fn read_unsupported_entity_between_supported_entities() {
    let file = parse_drawing(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "LINE",
                "0", "UNSUPPORTED_ENTITY",
                    "1", "unsupported string",
                "0", "CIRCLE",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(2, file.entities.len());
    match file.entities[0].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a line"),
    }
    match file.entities[1].specific {
        EntityType::Circle(_) => (),
        _ => panic!("expected a circle"),
    }
}

#[test]
fn read_entity_with_no_values() {
    let file = parse_drawing(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "LINE",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(1, file.entities.len());
    match file.entities[0].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a line"),
    }
}

#[test]
fn read_common_entity_fields() {
    let ent = read_entity("LINE", vec!["8", "layer"].join("\r\n"));
    assert_eq!("layer", ent.common.layer);
}

#[test]
fn read_line() {
    let ent = read_entity("LINE", vec![
        "10", "1.1", // p1
        "20", "2.2",
        "30", "3.3",
        "11", "4.4", // p2
        "21", "5.5",
        "31", "6.6"].join("\r\n"));
    match ent.specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.1, 2.2, 3.3), line.p1);
            assert_eq!(Point::new(4.4, 5.5, 6.6), line.p2);
        },
        _ => panic!("expected a line"),
    }
}

#[test]
fn write_common_entity_fields() {
    let mut drawing = Drawing::new();
    let mut ent = Entity {
        common: EntityCommon::new(),
        specific: EntityType::Line(Default::default())
    };
    ent.common.layer = "some-layer".to_owned();
    drawing.entities.push(ent);
    assert_contains(&drawing, vec![
        "  0", "LINE",
        "  5", "0",
        "100", "AcDbEntity",
        "  8", "some-layer",
    ].join("\r\n"));
}

#[test]
fn write_specific_entity_fields() {
    let mut drawing = Drawing::new();
    let line = Line {
        p1: Point::new(1.1, 2.2, 3.3),
        p2: Point::new(4.4, 5.5, 6.6),
        .. Default::default()
    };
    drawing.entities.push(Entity::new(EntityType::Line(line)));
    assert_contains(&drawing, vec![
        "100", "AcDbLine",
        " 10", "1.100000000000",
        " 20", "2.200000000000",
        " 30", "3.300000000000",
        " 11", "4.400000000000",
        " 21", "5.500000000000",
        " 31", "6.600000000000",
    ].join("\r\n"));
}

#[test]
fn read_multiple_entities() {
    let file = from_section("ENTITIES", vec![
        "0", "CIRCLE",
            "10", "1.1", // center
            "20", "2.2",
            "30", "3.3",
            "40", "4.4", // radius
        "0", "LINE",
            "10", "5.5", // p1
            "20", "6.6",
            "30", "7.7",
            "11", "8.8", // p2
            "21", "9.9",
            "31", "10.1"].join("\r\n").as_str());
    assert_eq!(2, file.entities.len());

    // verify circle
    match file.entities[0].specific {
        EntityType::Circle(ref circle) => {
            assert_eq!(Point::new(1.1, 2.2, 3.3), circle.center);
            assert_eq!(4.4, circle.radius);
        },
        _ => panic!("expected a line"),
    }

    // verify line
    match file.entities[1].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(5.5, 6.6, 7.7), line.p1);
            assert_eq!(Point::new(8.8, 9.9, 10.1), line.p2);
        },
        _ => panic!("expected a line"),
    }
}

#[test]
fn read_field_with_multiples_common() {
    let ent = read_entity("LINE", vec!["310", "one", "310", "two"].join("\r\n"));
    assert_eq!(vec!["one", "two"], ent.common.preview_image_data);
}

#[test]
fn write_field_with_multiples_common() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2000;
    drawing.entities.push(Entity {
        common: EntityCommon { preview_image_data: vec![String::from("one"), String::from("two")], .. Default::default() },
        specific: EntityType::Line(Default::default()),
    });
    assert_contains(&drawing, vec!["310", "one", "310", "two"].join("\r\n"));
}

#[test]
fn read_field_with_multiples_specific() {
    let ent = read_entity("3DSOLID", vec!["1", "one-1", "1", "one-2", "3", "three-1", "3", "three-2"].join("\r\n"));
    match ent.specific {
        EntityType::Solid3D(ref solid3d) => {
            assert_eq!(vec!["one-1", "one-2"], solid3d.custom_data);
            assert_eq!(vec!["three-1", "three-2"], solid3d.custom_data2);
        },
        _ => panic!("expected a 3DSOLID"),
    }
}

#[test]
fn write_field_with_multiples_specific() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R13; // 3DSOLID is only supported on R13+
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Solid3D(Solid3D {
            custom_data: vec![String::from("one-1"), String::from("one-2")],
            custom_data2: vec![String::from("three-1"), String::from("three-2")],
            .. Default::default()
        }),
    });
    assert_contains(&drawing, vec!["  1", "one-1", "  1", "one-2", "  3", "three-1", "  3", "three-2"].join("\r\n"));
}

#[test]
fn read_entity_with_post_parse() {
    let ent = read_entity("IMAGE", vec![
        "14", "1.1", // clipping_vertices[0]
        "24", "2.2",
        "14", "3.3", // clipping_vertices[1]
        "24", "4.4",
        "14", "5.5", // clipping_vertices[2]
        "24", "6.6",
    ].join("\r\n"));
    match ent.specific {
        EntityType::Image(ref image) => {
            assert_eq!(3, image.clipping_vertices.len());
            assert_eq!(Point::new(1.1, 2.2, 0.0), image.clipping_vertices[0]);
            assert_eq!(Point::new(3.3, 4.4, 0.0), image.clipping_vertices[1]);
            assert_eq!(Point::new(5.5, 6.6, 0.0), image.clipping_vertices[2]);
        },
        _ => panic!("expected an IMAGE"),
    }
}

#[test]
fn write_entity_with_write_order() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // IMAGE is only supported on R14+
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Image(Image {
            clipping_vertices: vec![Point::new(1.1, 2.2, 0.0), Point::new(3.3, 4.4, 0.0), Point::new(5.5, 6.6, 0.0)],
            .. Default::default()
        }),
    });
    assert_contains(&drawing, vec![
        " 91", "3",
        " 14", "1.100000000000",
        " 24", "2.200000000000",
        " 14", "3.300000000000",
        " 24", "4.400000000000",
        " 14", "5.500000000000",
        " 24", "6.600000000000",
    ].join("\r\n"));
}

#[test]
fn read_entity_with_custom_reader_mtext() {
    let ent = read_entity("MTEXT", vec![
        "50", "1.1", // rotation angle
        "75", "7", // column type
        "50", "3", // column count
        "50", "10", // column values
        "50", "20",
        "50", "30",
    ].join("\r\n"));
    match ent.specific {
        EntityType::MText(ref mtext) => {
            assert_eq!(1.1, mtext.rotation_angle);
            assert_eq!(7, mtext.column_type);
            assert_eq!(3, mtext.column_count);
            assert_eq!(3, mtext.column_heights.len());
            assert_eq!(10.0, mtext.column_heights[0]);
            assert_eq!(20.0, mtext.column_heights[1]);
            assert_eq!(30.0, mtext.column_heights[2]);
        },
        _ => panic!("expected an MTEXT"),
    }
}

#[test]
fn read_entity_with_flags() {
    let ent = read_entity("IMAGE", vec!["70", "5"].join("\r\n"));
    match ent.specific {
        EntityType::Image(ref image) => {
            assert!(image.get_show_image());
            assert!(image.get_use_clipping_boundary());
        },
        _ => panic!("expected an IMAGE"),
    }
}

#[test]
fn write_entity_with_flags() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // IMAGE is only supported on R14+
    let mut image = Image::default();
    assert_eq!(0, image.display_options_flags);
    image.set_show_image(true);
    image.set_use_clipping_boundary(true);
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Image(image),
    });
    assert_contains(&drawing, vec![
        " 70", "5", // flags
        "280", "1", // sentinels to make sure we're not reading a header value
        "281", "50",
    ].join("\r\n"));
}

#[test]
fn read_entity_with_handle_and_pointer() {
    let ent = read_entity("3DSOLID", vec![
        "5", "A1", // handle
        "330", "A2", // owner handle
        "350", "A3", // history_object pointer
    ].join("\r\n"));
    assert_eq!(0xa1, ent.common.handle);
    assert_eq!(0xa2, ent.common.owner_handle);
    match ent.specific {
        EntityType::Solid3D(ref solid) => assert_eq!(0xa3, solid.history_object),
        _ => panic!("expected a 3DSOLID entity"),
    }
}

#[test]
fn write_entity_with_handle_and_pointer() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2000;
    drawing.entities.push(Entity {
        common: EntityCommon {
            handle: 0xa1,
            owner_handle: 0xa2,
            .. Default::default()
        },
        specific: EntityType::Line(Default::default()),
    });
    assert_contains(&drawing, vec![
        "  5", "A1",
        "330", "A2",
    ].join("\r\n"));
}

#[test]
fn write_version_specific_entity() {
    let mut drawing = Drawing::new();
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Solid3D(Default::default()),
    });

    // 3DSOLID not supported in R12 and below
    drawing.header.version = AcadVersion::R12;
    assert_contains(&drawing, vec![
        "  0", "SECTION",
        "  2", "ENTITIES",
        "  0", "ENDSEC",
    ].join("\r\n"));

    // but it is in R13 and above
    drawing.header.version = AcadVersion::R13;
    assert_contains(&drawing, vec![
        "  0", "SECTION",
        "  2", "ENTITIES",
        "  0", "3DSOLID",
    ].join("\r\n"));
}
