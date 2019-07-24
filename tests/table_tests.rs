// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::enums::*;
use self::dxf::tables::*;
use self::dxf::*;

mod test_helpers;
use test_helpers::helpers::*;

fn read_table(table_name: &str, value_pairs: Vec<&str>) -> Drawing {
    let mut pairs = vec![
        "0",
        "SECTION",
        "2",
        "TABLES",
        "0",
        "TABLE",
        "2",
        table_name,
        "100",
        "AcDbSymbolTable",
        "70",
        "0",
    ];

    for pair in value_pairs {
        pairs.push(pair);
    }

    pairs.push("0");
    pairs.push("ENDTAB");
    pairs.push("0");
    pairs.push("ENDSEC");
    pairs.push("0");
    pairs.push("EOF");

    parse_drawing(pairs.join("\r\n").as_str())
}

#[test]
fn read_unsupported_table() {
    let drawing = parse_drawing(
        vec![
            "0",
            "SECTION",
            "2",
            "TABLES",
            "0",
            "TABLE",
            "2",
            "UNSUPPORTED",
            "0",
            "UNSUPPORTED",
            "2",
            "unsupported-name",
            "0",
            "ENDTAB",
            "0",
            "TABLE",
            "2",
            "LAYER",
            "0",
            "LAYER",
            "0",
            "ENDTAB",
            "0",
            "ENDSEC",
            "0",
            "EOF",
        ]
        .join("\r\n")
        .as_str(),
    );
    assert_eq!(1, drawing.layers.len());
}

#[test]
fn read_single_layer() {
    let drawing = read_table("LAYER", vec!["0", "LAYER", "2", "layer-name"]);
    assert_eq!(1, drawing.layers.len());
    assert_eq!("layer-name", drawing.layers[0].name);
}

#[test]
fn read_variable_table_items() {
    let drawing = parse_drawing(
        vec![
            "0",
            "SECTION",
            "2",
            "TABLES",
            // no app ids
            "0",
            "TABLE",
            "2",
            "APPID",
            "0",
            "ENDTAB",
            // 1 layer
            "0",
            "TABLE",
            "2",
            "LAYER",
            "0",
            "LAYER",
            "2",
            "layer-name",
            "0",
            "ENDTAB",
            // 2 styles
            "0",
            "TABLE",
            "2",
            "STYLE",
            "0",
            "STYLE",
            "40",
            "1.1",
            "0",
            "STYLE",
            "40",
            "2.2",
            "0",
            "ENDTAB",
            "0",
            "ENDSEC",
            "0",
            "EOF",
        ]
        .join("\r\n")
        .as_str(),
    );
    assert_eq!(0, drawing.block_records.len()); // not listed in file, but make sure there are still 0
    assert_eq!(0, drawing.app_ids.len());
    assert_eq!(1, drawing.layers.len());
    assert_eq!("layer-name", drawing.layers[0].name);
    assert_eq!(2, drawing.styles.len());
    assert_eq!(1.1, drawing.styles[0].text_height);
    assert_eq!(2.2, drawing.styles[1].text_height);
}

#[test]
fn read_layer_color_and_layer_is_on() {
    let drawing = read_table("LAYER", vec!["0", "LAYER", "62", "5"]);
    let layer = &drawing.layers[0];
    assert_eq!(Some(5), layer.color.index());
    assert!(layer.is_layer_on);
}

#[test]
fn read_layer_color_and_layer_is_off() {
    let drawing = read_table("LAYER", vec!["0", "LAYER", "62", "-5"]);
    let layer = &drawing.layers[0];
    assert_eq!(Some(5), layer.color.index());
    assert!(!layer.is_layer_on);
}

#[test]
fn write_layer() {
    let mut drawing = Drawing::default();
    let mut layer = Layer::default();
    layer.name = String::from("layer-name");
    layer.color = Color::from_index(3);
    drawing.layers.push(layer);
    assert_contains(
        &drawing,
        vec![
            "  0",
            "TABLE",
            "  2",
            "LAYER",
            "100",
            "AcDbSymbolTable",
            " 70",
            "     0",
            "  0",
            "LAYER",
            "  5",
            "1",
            "100",
            "AcDbSymbolTableRecord",
            "100",
            "AcDbLayerTableRecord",
            "  2",
            "layer-name",
            " 70",
            "     0",
            " 62",
            "     3",
            "  6",
            "CONTINUOUS",
        ]
        .join("\r\n"),
    );
}

#[test]
fn normalize_layer() {
    let mut layer = Layer::default();
    layer.name = String::from("layer-name");
    layer.color = Color::by_layer(); // value 256 not valid; normalized to 7
    layer.line_type_name = String::from(""); // empty string not valid; normalized to CONTINUOUS
    layer.normalize();
    assert_eq!(Some(7), layer.color.index());
    assert_eq!("CONTINUOUS", layer.line_type_name);
}

#[test]
fn normalize_view() {
    let mut view = View::default();
    view.view_height = 0.0; // invalid; normalized to 1.0
    view.view_width = -1.0; // invalid; normalized to 1.0
    view.lens_length = 42.0; // valid
    view.normalize();
    assert_eq!(1.0, view.view_height);
    assert_eq!(1.0, view.view_width);
    assert_eq!(42.0, view.lens_length);
}

#[test]
fn read_table_item_with_extended_data() {
    let drawing = read_table(
        "LAYER",
        vec![
            "  0",
            "LAYER",
            "102",
            "{IXMILIA",
            "  1",
            "some string",
            "102",
            "}",
        ],
    );
    let layer = &drawing.layers[0];
    assert_eq!(1, layer.extension_data_groups.len());
    let group = &layer.extension_data_groups[0];
    assert_eq!("IXMILIA", group.application_name);
    assert_eq!(1, group.items.len());
    match group.items[0] {
        ExtensionGroupItem::CodePair(ref p) => {
            assert_eq!(&CodePair::new_str(1, "some string"), p);
        }
        _ => panic!("expected a code pair"),
    }
}

#[test]
fn write_table_item_with_extended_data() {
    let layer = Layer {
        extension_data_groups: vec![ExtensionGroup {
            application_name: String::from("IXMILIA"),
            items: vec![ExtensionGroupItem::CodePair(CodePair::new_str(
                1,
                "some string",
            ))],
        }],
        ..Default::default()
    };
    let drawing = Drawing {
        header: Header {
            version: AcadVersion::R14,
            ..Default::default()
        },
        layers: vec![layer],
        ..Default::default()
    };
    assert_contains(
        &drawing,
        vec!["102", "{IXMILIA", "  1", "some string", "102", "}"].join("\r\n"),
    );
}

#[test]
fn read_table_item_with_x_data() {
    let drawing = read_table(
        "LAYER",
        vec!["  0", "LAYER", "1001", "IXMILIA", "1040", "1.1"],
    );
    let layer = &drawing.layers[0];
    assert_eq!(1, layer.x_data.len());
    let x = &layer.x_data[0];
    assert_eq!("IXMILIA", x.application_name);
    assert_eq!(1, x.items.len());
    match x.items[0] {
        XDataItem::Real(r) => assert_eq!(1.1, r),
        _ => panic!("expected a code pair"),
    }
}

#[test]
fn write_table_item_with_x_data() {
    let layer = Layer {
        x_data: vec![XData {
            application_name: String::from("IXMILIA"),
            items: vec![XDataItem::Real(1.1)],
        }],
        ..Default::default()
    };
    let drawing = Drawing {
        header: Header {
            version: AcadVersion::R2000,
            ..Default::default()
        },
        layers: vec![layer],
        ..Default::default()
    };
    assert_contains(
        &drawing,
        vec!["1001", "IXMILIA", "1040", "1.1"].join("\r\n"),
    );
}
