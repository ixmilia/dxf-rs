// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;

mod test_helpers;
use test_helpers::helpers::*;

fn read_table(table_name: &str, value_pairs: Vec<&str>) -> Drawing {
    let mut pairs = vec![
        "0", "SECTION",
        "2", "TABLES",
            "0", "TABLE",
            "2", table_name,
    ];

    for pair in value_pairs {
        pairs.push(pair);
    }

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
            "0", "TABLE",
            "2", "LAYER",
                "0", "LAYER",
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

            // 1 layer
            "0", "TABLE",
            "2", "LAYER",
                "0", "LAYER",
                "2", "layer-name",

            // 2 styles
            "0", "TABLE",
            "2", "STYLE",
                "0", "STYLE",
                "40", "1.1",
                "0", "STYLE",
                "40", "2.2",
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
