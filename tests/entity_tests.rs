// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::entities::*;
use self::dxf::enums::*;

mod test_helpers;
use test_helpers::helpers::*;

mod generated;
use generated::all_types;

fn read_entity(entity_type: &str, body: String) -> Entity {
    let drawing = from_section("ENTITIES", vec!["0", entity_type, body.as_str()].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    drawing.entities[0].to_owned()
}

#[test]
fn read_empty_entities_section() {
    let drawing = parse_drawing(vec!["0", "SECTION", "2", "ENTITIES", "0", "ENDSEC", "0", "EOF"].join("\r\n").as_str());
    assert_eq!(0, drawing.entities.len());
}

#[test]
fn read_unsupported_entity() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "UNSUPPORTED_ENTITY",
                    "1", "unsupported string",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(0, drawing.entities.len());
}

#[test]
fn read_unsupported_entity_between_supported_entities() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "LINE",
                "0", "UNSUPPORTED_ENTITY",
                    "1", "unsupported string",
                "0", "CIRCLE",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a line"),
    }
    match drawing.entities[1].specific {
        EntityType::Circle(_) => (),
        _ => panic!("expected a circle"),
    }
}

#[test]
fn read_entity_with_no_values() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "LINE",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
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
    let mut drawing = Drawing::default();
    let mut ent = Entity {
        common: Default::default(),
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
    let mut drawing = Drawing::default();
    let line = Line {
        p1: Point::new(1.1, 2.2, 3.3),
        p2: Point::new(4.4, 5.5, 6.6),
        .. Default::default()
    };
    drawing.entities.push(Entity::new(EntityType::Line(line)));
    assert_contains(&drawing, vec![
        "100", "AcDbLine",
        " 10", "1.1",
        " 20", "2.2",
        " 30", "3.3",
        " 11", "4.4",
        " 21", "5.5",
        " 31", "6.6",
    ].join("\r\n"));
}

#[test]
fn read_multiple_entities() {
    let drawing = from_section("ENTITIES", vec![
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
    assert_eq!(2, drawing.entities.len());

    // verify circle
    match drawing.entities[0].specific {
        EntityType::Circle(ref circle) => {
            assert_eq!(Point::new(1.1, 2.2, 3.3), circle.center);
            assert_eq!(4.4, circle.radius);
        },
        _ => panic!("expected a line"),
    }

    // verify line
    match drawing.entities[1].specific {
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
    let mut drawing = Drawing::default();
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
    let mut drawing = Drawing::default();
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
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R14; // IMAGE is only supported on R14+
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Image(Image {
            clipping_vertices: vec![Point::new(1.1, 2.2, 0.0), Point::new(3.3, 4.4, 0.0), Point::new(5.5, 6.6, 0.0)],
            .. Default::default()
        }),
    });
    assert_contains(&drawing, vec![
        " 91", "        3",
        " 14", "1.1",
        " 24", "2.2",
        " 14", "3.3",
        " 24", "4.4",
        " 14", "5.5",
        " 24", "6.6",
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
fn read_entity_after_entity_with_custom_reader() {
    let drawing = from_section("ENTITIES", vec![
        "  0", "MTEXT", // has a custom reader
        "  0", "LINE", // uses the auto-generated reader
    ].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::MText(_) => {},
        _ => panic!("expected an mtext"),
    }
    match drawing.entities[1].specific {
        EntityType::Line(_) => {},
        _ => panic!("expected a line"),
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
    let mut drawing = Drawing::default();
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
        " 70", "     5", // flags
        "280", "     1", // sentinels to make sure we're not reading a header value
        "281", "    50",
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
    let mut drawing = Drawing::default();
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
    let mut drawing = Drawing::default();
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

#[test]
fn read_polyline() {
    let drawing = from_section("ENTITIES", vec![
        "  0", "POLYLINE", // polyline sentinel
        "  0", "VERTEX", // vertex 1
        " 10", "1.1",
        " 20", "2.1",
        " 30", "3.1",
        "  0", "VERTEX", // vertex 2
        " 10", "1.2",
        " 20", "2.2",
        " 30", "3.2",
        "  0", "VERTEX", // vertex 3
        " 10", "1.3",
        " 20", "2.3",
        " 30", "3.3",
        "  0", "SEQEND", // end sequence
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => {
            assert_eq!(vec![
                Vertex { location: Point::new(1.1, 2.1, 3.1), .. Default::default() },
                Vertex { location: Point::new(1.2, 2.2, 3.2), .. Default::default() },
                Vertex { location: Point::new(1.3, 2.3, 3.3), .. Default::default() },
            ], poly.vertices);
        },
        _ => panic!("expected a POLYLINE"),
    }
}

#[test]
fn read_polyline_without_seqend() {
    let drawing = from_section("ENTITIES", vec![
        "  0", "POLYLINE", // polyline sentinel
        "  0", "VERTEX", // vertex 1
        " 10", "1.1",
        " 20", "2.1",
        " 30", "3.1",
        "  0", "VERTEX", // vertex 2
        " 10", "1.2",
        " 20", "2.2",
        " 30", "3.2",
        "  0", "VERTEX", // vertex 3
        " 10", "1.3",
        " 20", "2.3",
        " 30", "3.3",
        // no end sequence
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => {
            assert_eq!(vec![
                Vertex { location: Point::new(1.1, 2.1, 3.1), .. Default::default() },
                Vertex { location: Point::new(1.2, 2.2, 3.2), .. Default::default() },
                Vertex { location: Point::new(1.3, 2.3, 3.3), .. Default::default() },
            ], poly.vertices);
        },
        _ => panic!("expected a POLYLINE"),
    }
}

#[test]
fn read_empty_polyline() {
    let drawing = from_section("ENTITIES", vec!["0", "POLYLINE", "0", "SEQEND"].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => assert_eq!(0, poly.vertices.len()),
        _ => panic!("expected a POLYLINE"),
    }
}

#[test]
fn read_empty_polyline_without_seqend() {
    let drawing = from_section("ENTITIES", vec!["0", "POLYLINE"].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => assert_eq!(0, poly.vertices.len()),
        _ => panic!("expected a POLYLINE"),
    }
}

#[test]
fn read_polyline_with_trailing_entity() {
    let drawing = from_section("ENTITIES", vec![
        "  0", "POLYLINE", // polyline sentinel
        "  0", "VERTEX", // vertex 1
        " 10", "1.1",
        " 20", "2.1",
        " 30", "3.1",
        "  0", "VERTEX", // vertex 2
        " 10", "1.2",
        " 20", "2.2",
        " 30", "3.2",
        "  0", "VERTEX", // vertex 3
        " 10", "1.3",
        " 20", "2.3",
        " 30", "3.3",
        "  0", "SEQEND", // end sequence
        "  0", "LINE", // trailing entity
    ].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => {
            assert_eq!(vec![
                Vertex { location: Point::new(1.1, 2.1, 3.1), .. Default::default() },
                Vertex { location: Point::new(1.2, 2.2, 3.2), .. Default::default() },
                Vertex { location: Point::new(1.3, 2.3, 3.3), .. Default::default() },
            ], poly.vertices);
        },
        _ => panic!("expected a POLYLINE"),
    }

    match drawing.entities[1].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a LINE"),
    }
}

#[test]
fn read_polyline_without_seqend_with_trailing_entity() {
    let drawing = from_section("ENTITIES", vec![
        "  0", "POLYLINE", // polyline sentinel
        "  0", "VERTEX", // vertex 1
        " 10", "1.1",
        " 20", "2.1",
        " 30", "3.1",
        "  0", "VERTEX", // vertex 2
        " 10", "1.2",
        " 20", "2.2",
        " 30", "3.2",
        "  0", "VERTEX", // vertex 3
        " 10", "1.3",
        " 20", "2.3",
        " 30", "3.3",
        // no end sequence
        "  0", "LINE", // trailing entity
    ].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => {
            assert_eq!(vec![
                Vertex { location: Point::new(1.1, 2.1, 3.1), .. Default::default() },
                Vertex { location: Point::new(1.2, 2.2, 3.2), .. Default::default() },
                Vertex { location: Point::new(1.3, 2.3, 3.3), .. Default::default() },
            ], poly.vertices);
        },
        _ => panic!("expected a POLYLINE"),
    }

    match drawing.entities[1].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a LINE"),
    }
}

#[test]
fn read_empty_polyline_with_trailing_entity() {
    let drawing = from_section("ENTITIES", vec!["0", "POLYLINE", "0", "SEQEND", "0", "LINE"].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => assert_eq!(0, poly.vertices.len()),
        _ => panic!("expected a POLYLINE"),
    }
    match drawing.entities[1].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a LINE"),
    }
}

#[test]
fn read_empty_polyline_without_seqend_with_trailing_entity() {
    let drawing = from_section("ENTITIES", vec!["0", "POLYLINE", "0", "LINE"].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Polyline(ref poly) => assert_eq!(0, poly.vertices.len()),
        _ => panic!("expected a POLYLINE"),
    }
    match drawing.entities[1].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a LINE"),
    }
}

#[test]
fn write_polyline() {
    let mut drawing = Drawing::default();
    let poly = Polyline {
        vertices: vec![
            Vertex { location: Point::new(1.1, 2.1, 3.1), .. Default::default() },
            Vertex { location: Point::new(1.2, 2.2, 3.2), .. Default::default() },
            Vertex { location: Point::new(1.3, 2.3, 3.3), .. Default::default() },
        ],
        .. Default::default()
    };
    drawing.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Polyline(poly),
    });
    assert_contains(&drawing, vec![
        "  0", "POLYLINE", // polyline
        "  5", "0",
        "100", "AcDbEntity",
        "  8", "0",
        "100", "AcDb2dPolyline",
        " 66", "     1",
        " 10", "0.0",
        " 20", "0.0",
        " 30", "0.0",
        "  0", "VERTEX", // vertex 1
        "  5", "0",
        "100", "AcDbEntity",
        "  8", "0",
        "100", "AcDbVertex",
        " 10", "1.1",
        " 20", "2.1",
        " 30", "3.1",
        " 70", "     0",
        " 50", "0.0",
        "  0", "VERTEX", // vertex 2
        "  5", "0",
        "100", "AcDbEntity",
        "  8", "0",
        "100", "AcDbVertex",
        " 10", "1.2",
        " 20", "2.2",
        " 30", "3.2",
        " 70", "     0",
        " 50", "0.0",
        "  0", "VERTEX", // vertex 3
        "  5", "0",
        "100", "AcDbEntity",
        "  8", "0",
        "100", "AcDbVertex",
        " 10", "1.3",
        " 20", "2.3",
        " 30", "3.3",
        " 70", "     0",
        " 50", "0.0",
        "  0", "SEQEND", // end sequence
    ].join("\r\n"));
}

#[test]
fn read_lw_polyline_with_no_vertices() {
    let drawing = from_section("ENTITIES", vec![
        "0", "LWPOLYLINE",
            "43", "43.0",
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match &drawing.entities[0].specific {
        &EntityType::LwPolyline(ref poly) => {
            assert_eq!(43.0, poly.constant_width);
            assert_eq!(0, poly.vertices.len());
        },
        _ => panic!("expected an LWPOLYLINE"),
    }
}

#[test]
fn read_lw_polyline_with_one_vertex() {
    let drawing = from_section("ENTITIES", vec![
        "0", "LWPOLYLINE",
            "43", "43.0",
            // vertex 1
            "10", "1.1",
            "20", "2.1",
            "40", "40.1",
            "41", "41.1",
            "42", "42.1",
            "91", "91",
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match &drawing.entities[0].specific {
        &EntityType::LwPolyline(ref poly) => {
            assert_eq!(43.0, poly.constant_width);
            assert_eq!(1, poly.vertices.len());
            // vertex 1
            assert_eq!(1.1, poly.vertices[0].x);
            assert_eq!(2.1, poly.vertices[0].y);
            assert_eq!(40.1, poly.vertices[0].starting_width);
            assert_eq!(41.1, poly.vertices[0].ending_width);
            assert_eq!(42.1, poly.vertices[0].bulge);
            assert_eq!(91, poly.vertices[0].id);
        },
        _ => panic!("expected an LWPOLYLINE"),
    }
}

#[test]
fn read_lw_polyline_with_multiple_vertices() {
    let drawing = from_section("ENTITIES", vec![
        "0", "LWPOLYLINE",
            "43", "43.0",
            // vertex 1
            "10", "1.1",
            "20", "2.1",
            "40", "40.1",
            "41", "41.1",
            "42", "42.1",
            "91", "91",
            // vertex 2
            "10", "1.2",
            "20", "2.2",
            "40", "40.2",
            "41", "41.2",
            "42", "42.2",
            "91", "92",
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match &drawing.entities[0].specific {
        &EntityType::LwPolyline(ref poly) => {
            assert_eq!(43.0, poly.constant_width);
            assert_eq!(2, poly.vertices.len());
            // vertex 1
            assert_eq!(1.1, poly.vertices[0].x);
            assert_eq!(2.1, poly.vertices[0].y);
            assert_eq!(40.1, poly.vertices[0].starting_width);
            assert_eq!(41.1, poly.vertices[0].ending_width);
            assert_eq!(42.1, poly.vertices[0].bulge);
            assert_eq!(91, poly.vertices[0].id);
            // vertex 2
            assert_eq!(1.2, poly.vertices[1].x);
            assert_eq!(2.2, poly.vertices[1].y);
            assert_eq!(40.2, poly.vertices[1].starting_width);
            assert_eq!(41.2, poly.vertices[1].ending_width);
            assert_eq!(42.2, poly.vertices[1].bulge);
            assert_eq!(92, poly.vertices[1].id);
        },
        _ => panic!("expected an LWPOLYLINE"),
    }
}

#[test]
fn write_lw_polyline() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2013;
    let mut poly = LwPolyline::default();
    poly.constant_width = 43.0;
    poly.vertices.push(LwPolylineVertex {
        x: 1.1,
        y: 2.1,
        .. Default::default()
    });
    poly.vertices.push(LwPolylineVertex {
        x: 1.2,
        y: 2.2,
        starting_width: 40.2,
        ending_width: 41.2,
        bulge: 42.2,
        id: 92,
    });
    drawing.entities.push(Entity::new(EntityType::LwPolyline(poly)));
    assert_contains(&drawing, vec![
        "100", "AcDbPolyline",
        " 90", "        2",
        " 70", "     0",
        " 43", "43.0",
        // vertex 1
        " 10", "1.1",
        " 20", "2.1",
        " 91", "        0",
        // vertex 2
        " 10", "1.2",
        " 20", "2.2",
        " 91", "       92",
        " 40", "40.2",
        " 41", "41.2",
        " 42", "42.2",
    ].join("\r\n"));
}

#[test]
fn read_dimension() {
    let ent = read_entity("DIMENSION", vec![
        "1", "text",
        "100", "AcDbOrdinateDimension",
        "13", "1.1", // definition_point_2
        "23", "2.2",
        "33", "3.3",
        "14", "4.4", // definition_point_3
        "24", "5.5",
        "34", "6.6"].join("\r\n"));
    match ent.specific {
        EntityType::OrdinateDimension(ref dim) => {
            assert_eq!("text", dim.dimension_base.text);
            assert_eq!(Point::new(1.1, 2.2, 3.3), dim.definition_point_2);
            assert_eq!(Point::new(4.4, 5.5, 6.6), dim.definition_point_3);
        },
        _ => panic!("expected an ordinate dimension"),
    }
}

#[test]
fn read_entity_after_unsupported_dimension() {
    let drawing = from_section("ENTITIES", vec![
        "0", "DIMENSION",
            "1", "text",
            "100", "AcDbSomeUnsupportedDimensionType",
            "10", "1.1",
            "20", "2.2",
            "30", "3.3",
        "0", "LINE",
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(_) => {},
        _ => panic!("expected a line"),
    }
}

#[test]
fn write_dimension() {
    let dim = RadialDimension {
        dimension_base: DimensionBase { text: String::from("some-text"), .. Default::default() },
        definition_point_2: Point::new(1.1, 2.2, 3.3),
        .. Default::default()
    };
    let ent = Entity::new(EntityType::RadialDimension(dim));
    let mut drawing = Drawing::default();
    drawing.entities.push(ent);
    assert_contains(&drawing, vec!["  0", "DIMENSION"].join("\r\n"));
    assert_contains(&drawing, vec!["  1", "some-text"].join("\r\n"));
    assert_contains(&drawing, vec![
        "100", "AcDbRadialDimension",
        " 15", "1.1", // definition_point_2
        " 25", "2.2",
        " 35", "3.3",
        " 40", "0.0", // leader_length
    ].join("\r\n"));
}

#[test]
fn read_extension_data() {
    let ent = read_entity("LINE", vec![
        "102", "{IXMILIA",
        "  1", "some string",
        "102", "}",
    ].join("\r\n"));
    assert_eq!(1, ent.common.extension_data_groups.len());
    let group = &ent.common.extension_data_groups[0];
    assert_eq!("IXMILIA", group.application_name);
    match group.items[0] {
        ExtensionGroupItem::CodePair(ref p) => assert_eq!(&CodePair::new_str(1, "some string"), p),
        _ => panic!("expected a code pair"),
    }
}

#[test]
fn write_extension_data() {
    let drawing = Drawing {
        header: Header { version: AcadVersion::R14, .. Default::default() },
        entities: vec![
            Entity {
                common: EntityCommon {
                    extension_data_groups: vec![
                        ExtensionGroup {
                            application_name: String::from("IXMILIA"),
                            items: vec![
                                ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string")),
                            ],
                        }
                    ],
                    .. Default::default()
                },
                specific: EntityType::Line(Line::default()),
            }
        ],
        .. Default::default()
    };
    assert_contains(&drawing, vec![
        "102", "{IXMILIA",
        "  1", "some string",
        "102", "}",
    ].join("\r\n"));
}

#[test]
fn read_x_data() {
    let ent = read_entity("LINE", vec![
        "1001", "IXMILIA",
        "1000", "some string",
    ].join("\r\n"));
    assert_eq!(1, ent.common.x_data.len());
    let x = &ent.common.x_data[0];
    assert_eq!("IXMILIA", x.application_name);
    match x.items[0] {
        XDataItem::Str(ref s) => assert_eq!("some string", s),
        _ => panic!("expected a string"),
    }
}

#[test]
fn write_x_data() {
    let drawing = Drawing {
        header: Header { version: AcadVersion::R2000, .. Default::default() },
        entities: vec![
            Entity {
                common: EntityCommon {
                    x_data: vec![
                        XData {
                            application_name: String::from("IXMILIA"),
                            items: vec![
                                XDataItem::Real(1.1),
                            ],
                        }
                    ],
                    .. Default::default()
                },
                specific: EntityType::Line(Line::default()),
            }
        ],
        .. Default::default()
    };
    assert_contains(&drawing, vec![
        "1001", "IXMILIA",
        "1040", "1.1",
        "  0", "ENDSEC", // xdata is written after all the entity's other code pairs
    ].join("\r\n"));
}

#[test]
fn read_entity_after_extension_data() {
    let drawing = parse_drawing(vec![
        "  0", "SECTION",
        "  2", "ENTITIES",
            "  0", "LINE",
            "102", "{IXMILIA",
            "102", "}",
            "  0", "CIRCLE",
        "  0", "ENDSEC",
        "  0", "EOF",
    ].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a line"),
    }
    match drawing.entities[1].specific {
        EntityType::Circle(_) => (),
        _ => panic!("expected a circle"),
    }
}

#[test]
fn read_entity_after_x_data() {
    let drawing = parse_drawing(vec![
        "  0", "SECTION",
        "  2", "ENTITIES",
            "  0", "LINE",
            "1001", "IXMILIA",
            "  0", "CIRCLE",
        "  0", "ENDSEC",
        "  0", "EOF",
    ].join("\r\n").as_str());
    assert_eq!(2, drawing.entities.len());
    match drawing.entities[0].specific {
        EntityType::Line(_) => (),
        _ => panic!("expected a line"),
    }
    match drawing.entities[1].specific {
        EntityType::Circle(_) => (),
        _ => panic!("expected a circle"),
    }
}

#[test]
fn read_all_types() {
    for (type_string, subclass, expected_type, _) in all_types::get_all_entity_types() {
        println!("parsing {}/{}", type_string, subclass);
        let ent = read_entity(type_string, vec![
            "100", subclass,
            "102", "{IXMILIA", // read extension data
            "  1", "some string",
            "102", "}",
            "1001", "IXMILIA", // read x data
            "1040", "1.1",
        ].join("\r\n"));

        // validate specific
        assert_eq!(expected_type, ent.specific);

        // validate extension data
        assert_eq!(1, ent.common.extension_data_groups.len());
        assert_eq!("IXMILIA", ent.common.extension_data_groups[0].application_name);
        assert_eq!(1, ent.common.extension_data_groups[0].items.len());
        assert_eq!(ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string")), ent.common.extension_data_groups[0].items[0]);

        // validate x data
        assert_eq!(1, ent.common.x_data.len());
        assert_eq!("IXMILIA", ent.common.x_data[0].application_name);
        assert_eq!(1, ent.common.x_data[0].items.len());
        assert_eq!(XDataItem::Real(1.1), ent.common.x_data[0].items[0]);
    }
}

#[test]
fn write_all_types() {
    for (type_string, _, expected_type, max_version) in all_types::get_all_entity_types() {
        println!("writing {}", type_string);
        let mut common = EntityCommon::default();
        common.extension_data_groups.push(ExtensionGroup {
            application_name: String::from("IXMILIA"),
            items: vec![ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string"))]
        });
        common.x_data.push(XData {
            application_name: String::from("IXMILIA"),
            items: vec![XDataItem::Real(1.1)],
        });
        let drawing = Drawing {
            entities: vec![Entity { common: common, specific: expected_type }],
            header: Header { version: max_version, .. Default::default() },
            .. Default::default()
        };
        // 3DLINE writes as a LINE
        let type_string = if type_string == "3DLINE" { "LINE" } else { type_string };
        assert_contains(&drawing, vec![
            "  0", type_string,
        ].join("\r\n"));
        if max_version >= AcadVersion::R14 {
            // only written on R14+
            assert_contains(&drawing, vec![
                "102", "{IXMILIA",
                "  1", "some string",
                "102", "}",
            ].join("\r\n"));
        }
        if max_version >= AcadVersion::R2000 {
            // only written on R2000+
            assert_contains(&drawing, vec![
                "1001", "IXMILIA",
                "1040", "1.1",
            ].join("\r\n"));
        }
    }
}
