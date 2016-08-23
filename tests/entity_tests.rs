// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::entities::*;

mod test_helpers;
use test_helpers::helpers::*;

fn read_entity(entity_type: &str, body: String) -> Entity {
    let file = from_section("ENTITIES", vec!["0", entity_type, body.as_str()].join("\r\n").as_str());
    assert_eq!(1, file.entities.len());
    file.entities[0].to_owned()
}

#[test]
fn empty_entities_section() {
    let file = Drawing::parse(vec!["0", "SECTION", "2", "ENTITIES", "0", "ENDSEC", "0", "EOF"].join("\r\n").as_str()).ok().unwrap();
    assert_eq!(0, file.entities.len());
}

#[test]
fn unsupported_entity() {
    let file = Drawing::parse(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "UNSUPPORTED_ENTITY",
                    "1", "unsupported string",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str()).ok().unwrap();
    assert_eq!(0, file.entities.len());
}

#[test]
fn unsupported_entity_between_supported_entities() {
    let file = Drawing::parse(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "LINE",
                "0", "UNSUPPORTED_ENTITY",
                    "1", "unsupported string",
                "0", "CIRCLE",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str()).ok().unwrap();
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
    let file = Drawing::parse(vec![
        "0", "SECTION",
            "2", "ENTITIES",
                "0", "LINE",
        "0", "ENDSEC",
        "0", "EOF"].join("\r\n").as_str()).ok().unwrap();
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
fn entity_with_post_parse() {
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
fn entity_with_custom_reader_mtext() {
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
fn entity_with_flags() {
    let ent = read_entity("IMAGE", vec!["70", "5"].join("\r\n"));
    match ent.specific {
        EntityType::Image(ref image) => {
            assert!(image.get_show_image());
            assert!(image.get_use_clipping_boundary());
        },
        _ => panic!("expected an IMAGE"),
    }
}
