// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::enums::*;

mod test_helpers;
use test_helpers::helpers::*;

fn read_single_class(version_str: &str, body: Vec<&str>) -> Class {
    let mut lines = vec![
        "0", "SECTION",
        "2", "HEADER",
        "9", "$ACADVER",
        "1", version_str,
        "0", "ENDSEC",
        "0", "SECTION",
        "2", "CLASSES",
    ];
    for line in body {
        lines.push(line);
    }
    lines.push("0");
    lines.push("ENDSEC");
    lines.push("0");
    lines.push("EOF");
    let drawing = parse_drawing(lines.join("\n").as_str());
    assert_eq!(1, drawing.classes.len());
    drawing.classes[0].to_owned()
}

#[test]
fn read_empty_classes_section() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
        "2", "CLASSES",
        "0", "ENDSEC",
        "0", "EOF",
    ].join("\n").as_str());
    assert_eq!(0, drawing.classes.len());
}

#[test]
fn read_single_class_r13() {
    let class = read_single_class("AC1012", vec![
        "0", "record-name",
        "1", "class-name",
        "2", "application-name",
        "90", "42",
        "91", "43",
        "280", "1",
        "281", "1",
    ]);
    assert_eq!("record-name", class.record_name);
    assert_eq!("class-name", class.class_name);
    assert_eq!("application-name", class.application_name);
    assert_eq!(42, class.version_number);
    assert_eq!(43, class.instance_count);
    assert_eq!(0, class.proxy_capability_flags);
    assert!(!class.was_class_loaded_with_file);
    assert!(class.is_entity);
}

#[test]
fn read_single_class_r14() {
    let class = read_single_class("AC1015", vec![
        "0", "CLASS",
        "1", "record-name",
        "2", "class-name",
        "3", "application-name",
        "90", "42",
        "91", "43",
        "280", "1",
        "281", "1",
    ]);
    assert_eq!("record-name", class.record_name);
    assert_eq!("class-name", class.class_name);
    assert_eq!("application-name", class.application_name);
    assert_eq!(42, class.proxy_capability_flags);
    assert_eq!(43, class.instance_count);
    assert_eq!(0, class.version_number);
    assert!(!class.was_class_loaded_with_file);
    assert!(class.is_entity);
}

#[test]
fn read_multiple_classes_r13() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
        "2", "HEADER",
        "9", "$ACADVER",
        "1", "AC1012",
        "0", "ENDSEC",
        "0", "SECTION",
        "2", "CLASSES",
        "0", "some class 1",
        "0", "some class 2",
        "0", "ENDSEC",
        "0", "EOF",
    ].join("\n").as_str());
    assert_eq!(2, drawing.classes.len());
}

#[test]
fn read_multiple_classes_r14() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
        "2", "HEADER",
        "9", "$ACADVER",
        "1", "AC1014",
        "0", "ENDSEC",
        "0", "SECTION",
        "2", "CLASSES",
        "0", "CLASS",
        "0", "CLASS",
        "0", "ENDSEC",
        "0", "EOF",
    ].join("\n").as_str());
    assert_eq!(2, drawing.classes.len());
}

#[test]
fn dont_write_classes_section_if_no_classes() {
    let drawing = Drawing::default();
    let contents = to_test_string(&drawing);
    assert!(!contents.contains("CLASSES"));
}

#[test]
fn write_class_r13() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R13;
    let class = Class {
        record_name: "record-name".to_string(),
        class_name: "class-name".to_string(),
        application_name: "application-name".to_string(),
        version_number: 42,
        proxy_capability_flags: 43,
        instance_count: 44,
        was_class_loaded_with_file: false,
        is_entity: true,
    };
    drawing.classes.push(class);
    assert_contains(&drawing, vec![
        "  0", "SECTION",
        "  2", "CLASSES",
        "  0", "record-name",
        "  1", "class-name",
        "  2", "application-name",
        " 90", "       42",
        "280", "     1",
        "281", "     1",
        "  0", "ENDSEC",
    ].join("\r\n"));
}

#[test]
fn write_class_r14() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R14;
    let class = Class {
        record_name: "record-name".to_string(),
        class_name: "class-name".to_string(),
        application_name: "application-name".to_string(),
        version_number: 42,
        proxy_capability_flags: 43,
        instance_count: 44,
        was_class_loaded_with_file: false,
        is_entity: true,
    };
    drawing.classes.push(class);
    assert_contains(&drawing, vec![
        "  0", "SECTION",
        "  2", "CLASSES",
        "  0", "CLASS",
        "  1", "record-name",
        "  2", "class-name",
        "  3", "application-name",
        " 90", "       43",
        "280", "     1",
        "281", "     1",
        "  0", "ENDSEC",
    ].join("\r\n"));
}
