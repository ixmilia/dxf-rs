// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::tables::*;

mod test_helpers;
use test_helpers::helpers::*;

fn read_table(table_name: &str, value_pairs: Vec<&str>) -> Drawing {
    let mut pairs = vec![
        "0", "SECTION",
        "2", "TABLES",
            "0", "TABLE",
            "2", table_name,
            "100", "AcDbSymbolTable",
            "70", "0",
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
    let drawing = parse_drawing(vec![
        "0", "SECTION",
        "2", "TABLES",
            "0", "TABLE",
            "2", "UNSUPPORTED",
                "0", "UNSUPPORTED",
                "2", "unsupported-name",
            "0", "ENDTAB",
            "0", "TABLE",
            "2", "LAYER",
                "0", "LAYER",
            "0", "ENDTAB",
        "0", "ENDSEC",
        "0", "EOF",
    ].join("\r\n").as_str());
    assert_eq!(1, drawing.layers.len());
}

#[test]
fn read_single_layer() {
    let drawing = read_table("LAYER", vec![
        "0", "LAYER",
        "2", "layer-name",
    ]);
    assert_eq!(1, drawing.layers.len());
    assert_eq!("layer-name", drawing.layers[0].name);
}

#[test]
fn read_variable_table_items() {
    let drawing = parse_drawing(vec![
        "0", "SECTION",
        "2", "TABLES",
            // no app ids
            "0", "TABLE",
            "2", "APPID",
            "0", "ENDTAB",

            // 1 layer
            "0", "TABLE",
            "2", "LAYER",
                "0", "LAYER",
                "2", "layer-name",
            "0", "ENDTAB",

            // 2 styles
            "0", "TABLE",
            "2", "STYLE",
                "0", "STYLE",
                "40", "1.1",
                "0", "STYLE",
                "40", "2.2",
            "0", "ENDTAB",
        "0", "ENDSEC",
        "0", "EOF",
    ].join("\r\n").as_str());
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
    let drawing = read_table("LAYER", vec![
        "0", "LAYER",
        "62", "5",
    ]);
    let layer = &drawing.layers[0];
    assert_eq!(5, layer.color.get_raw_value());
    assert!(layer.is_layer_on);
}

#[test]
fn read_layer_color_and_layer_is_off() {
    let drawing = read_table("LAYER", vec![
        "0", "LAYER",
        "62", "-5",
    ]);
    let layer = &drawing.layers[0];
    assert_eq!(5, layer.color.get_raw_value());
    assert!(!layer.is_layer_on);
}

#[test]
fn write_layer() {
    let mut drawing = Drawing::new();
    let mut layer = Layer::new();
    layer.name = String::from("layer-name");
    layer.color = Color::from_index(3);
    drawing.layers.push(layer);
    assert_contains(&drawing, vec![
        "  0", "TABLE",
        "  2", "LAYER",
        "100", "AcDbSymbolTable",
        " 70", "     0",
            "  0", "LAYER",
            "  5", "0",
            "100", "AcDbSymbolTableRecord",
            "100", "AcDbLayerTableRecord",
            "  2", "layer-name",
            " 70", "     0",
            " 62", "     3",
            "  6", "CONTINUOUS",
    ].join("\r\n"));
}

#[test]
fn write_layer_with_invalid_values() {
    let mut drawing = Drawing::new();
    let mut layer = Layer::new();
    layer.name = String::from("layer-name");
    layer.color = Color::by_layer(); // code 62, value 256 not valid; normalized to 7
    layer.linetype_name = String::from(""); // code 6, empty string not valid; normalized to CONTINUOUS
    drawing.layers.push(layer);
    assert_contains(&drawing, vec![
        "  2", "layer-name",
        " 70", "     0",
        " 62", "     7",
        "  6", "CONTINUOUS",
    ].join("\r\n"));
}

#[test]
fn write_view_with_invalid_values() {
    let mut drawing = Drawing::new();
    let mut view = View::new();
    view.name = String::from("view-name");
    view.view_height = 0.0; // code 40, invalid; normalized to 1.0
    view.view_width = -1.0; // code 41, invalid; normalized to 1.0
    view.lens_length = 42.0; // code 42, valid
    drawing.views.push(view);
    assert_contains(&drawing, vec![
        "  2", "view-name",
        " 70", "     0",
        " 40", "1.0",
        " 10", "0.0",
        " 20", "0.0",
        " 41", "1.0",
        " 11", "0.0",
        " 21", "0.0",
        " 31", "1.0",
        " 12", "0.0",
        " 22", "0.0",
        " 32", "0.0",
        " 42", "42.0",
    ].join("\r\n"));
}
