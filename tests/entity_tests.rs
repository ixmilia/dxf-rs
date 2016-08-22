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
        EntityType::Line{..} => (),
        _ => panic!("expected a line"),
    }
    match file.entities[1].specific {
        EntityType::Circle{..} => (),
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
        EntityType::Line{..} => (),
        _ => panic!("expected a line"),
    }
}

#[test]
fn read_common_entity_fields() {
    let ent = read_entity("LINE", vec!["8", "layer"].join("\r\n"));
    assert_eq!("layer", ent.layer);
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
        EntityType::Line{ ref p1, ref p2, .. } => {
            assert_eq!(Point::new(1.1, 2.2, 3.3), *p1);
            assert_eq!(Point::new(4.4, 5.5, 6.6), *p2);
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
        EntityType::Circle{ ref center, ref radius, .. } => {
            assert_eq!(Point::new(1.1, 2.2, 3.3), *center);
            assert_eq!(4.4, *radius);
        },
        _ => panic!("expected a line"),
    }

    // verify line
    match file.entities[1].specific {
        EntityType::Line{ ref p1, ref p2, .. } => {
            assert_eq!(Point::new(5.5, 6.6, 7.7), *p1);
            assert_eq!(Point::new(8.8, 9.9, 10.1), *p2);
        },
        _ => panic!("expected a line"),
    }
}
