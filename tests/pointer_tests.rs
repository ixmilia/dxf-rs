// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::entities::*;
use self::dxf::objects::*;

mod test_helpers;
use test_helpers::helpers::*;

#[test]
fn follow_entity_pointer_to_object() {
    let drawing = parse_drawing(vec![
        "  0", "SECTION",
        "  2", "OBJECTS",
        "  0", "MATERIAL",
        "  5", "ABCD",
        "  1", "material-name",
        "  0", "ENDSEC",
        "  0", "SECTION",
        "  2", "ENTITIES",
        "  0", "LINE",
        "347", "ABCD",
        "  0", "ENDSEC",
        "  0", "EOF",
    ].join("\r\n").as_str());
    let line_common = match &drawing.entities[0] {
        &Entity { ref common, specific: EntityType::Line(_) } => common,
        _ => panic!("expected a line"),
    };
    let bound_material = match line_common.get_material(&drawing).unwrap().specific {
        ObjectType::Material(ref mat) => mat,
        _ => panic!("expected a material"),
    };
    assert_eq!("material-name", bound_material.name);
}

#[test]
fn follow_object_pointer_to_entity_collection() {
    let drawing = parse_drawing(vec![
        "  0", "SECTION",
        "  2", "OBJECTS",
        "  0", "GROUP",
        "340", "ABCD",
        "  0", "ENDSEC",
        "  0", "SECTION",
        "  2", "ENTITIES",
        "  0", "TEXT",
        "  5", "ABCD",
        "  1", "text value",
        "  0", "ENDSEC",
        "  0", "EOF",
    ].join("\r\n").as_str());
    let group = match drawing.objects[0].specific {
        ObjectType::Group(ref g) => g,
        _ => panic!("expected a group"),
    };
    let entity_collection = group.get_entities(&drawing);
    assert_eq!(1, entity_collection.len());
    let bound_text = match entity_collection[0].specific {
        EntityType::Text(ref t) => t,
        _ => panic!("expected text"),
    };
    assert_eq!("text value", bound_text.value);
}

#[test]
fn no_pointer_bound() {
    let drawing = from_section("ENTITIES", vec![
        "  0", "LINE",
    ].join("\r\n").as_str());
    match drawing.entities[0].common.get_material(&drawing) {
        None => (),
        _ => panic!("expected None"),
    }
}
