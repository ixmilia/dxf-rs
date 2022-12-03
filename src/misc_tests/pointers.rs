use crate::entities::*;
use crate::enums::*;
use crate::helper_functions::tests::*;
use crate::objects::*;
use crate::*;

#[test]
fn follow_entity_pointer_to_object() {
    let drawing = drawing_from_pairs(vec![
        CodePair::new_str(0, "SECTION"),
        CodePair::new_str(2, "OBJECTS"),
        CodePair::new_str(0, "MATERIAL"),
        CodePair::new_str(5, "ABCD"),
        CodePair::new_str(1, "material-name"),
        CodePair::new_str(0, "ENDSEC"),
        CodePair::new_str(0, "SECTION"),
        CodePair::new_str(2, "ENTITIES"),
        CodePair::new_str(0, "LINE"),
        CodePair::new_str(347, "ABCD"),
        CodePair::new_str(0, "ENDSEC"),
        CodePair::new_str(0, "EOF"),
    ]);
    let entities = drawing.entities().collect::<Vec<_>>();
    let line_common = match entities[0] {
        Entity {
            ref common,
            specific: EntityType::Line(_),
        } => common,
        _ => panic!("expected a line"),
    };
    let bound_material = match line_common.material(&drawing).unwrap().specific {
        ObjectType::Material(ref mat) => mat,
        _ => panic!("expected a material"),
    };
    assert_eq!("material-name", bound_material.name);
}

#[test]
fn follow_object_pointer_to_entity_collection() {
    let drawing = drawing_from_pairs(vec![
        CodePair::new_str(0, "SECTION"),
        CodePair::new_str(2, "OBJECTS"),
        CodePair::new_str(0, "GROUP"),
        CodePair::new_str(340, "ABCD"),
        CodePair::new_str(0, "ENDSEC"),
        CodePair::new_str(0, "SECTION"),
        CodePair::new_str(2, "ENTITIES"),
        CodePair::new_str(0, "TEXT"),
        CodePair::new_str(5, "ABCD"),
        CodePair::new_str(1, "text value"),
        CodePair::new_str(0, "ENDSEC"),
        CodePair::new_str(0, "EOF"),
    ]);
    let objects = drawing.objects().collect::<Vec<_>>();
    let group = match objects[0].specific {
        ObjectType::Group(ref g) => g,
        _ => panic!("expected a group"),
    };
    let entity_collection = group.entities(&drawing);
    assert_eq!(1, entity_collection.len());
    let bound_text = match entity_collection[0].specific {
        EntityType::Text(ref t) => t,
        _ => panic!("expected text"),
    };
    assert_eq!("text value", bound_text.value);
}

#[test]
fn no_pointer_bound() {
    let drawing = from_section("ENTITIES", vec![CodePair::new_str(0, "LINE")]);
    let entities = drawing.entities().collect::<Vec<_>>();
    match entities[0].common.material(&drawing) {
        None => (),
        _ => panic!("expected None"),
    }
}

#[test]
fn set_pointer_on_entity() {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R2007;
    let material = Object {
        common: Default::default(),
        specific: ObjectType::Material(Material {
            name: String::from("material-name"),
            ..Default::default()
        }),
    };
    let mut line = Entity {
        common: Default::default(),
        specific: EntityType::Line(Default::default()),
    };
    assert_eq!(Handle(0), material.common.handle);

    let material = drawing.add_object(material);
    assert_eq!(Handle(0x10), material.common.handle);
    line.common.set_material(material).ok().unwrap();
    drawing.add_entity(line);

    assert_contains_pairs(
        &drawing,
        vec![CodePair::new_str(0, "MATERIAL"), CodePair::new_str(5, "10")],
    );

    assert_contains_pairs(
        &drawing,
        vec![
            CodePair::new_str(0, "LINE"),
            CodePair::new_str(5, "11"),
            CodePair::new_str(100, "AcDbEntity"),
            CodePair::new_str(8, "0"),
            CodePair::new_str(347, "10"), // handle of `material`
        ],
    );
}
