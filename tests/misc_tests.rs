// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::entities::*;
use self::dxf::enums::*;
use self::dxf::objects::*;
use self::dxf::*;

extern crate image;
use self::image::{DynamicImage, GenericImageView};

mod test_helpers;
use test_helpers::helpers::*;

#[test]
fn read_string_with_control_characters() {
    let drawing = parse_drawing(
        vec![
            "0",
            "SECTION",
            "2",
            "HEADER",
            "9",
            "$LASTSAVEDBY",
            "1",
            "a^G^ ^^ b",
            "0",
            "ENDSEC",
            "0",
            "EOF",
        ]
        .join("\n")
        .as_str(),
    );
    assert_eq!("a\u{7}^\u{1E} b", drawing.header.last_saved_by);
}

#[test]
fn write_string_with_control_characters() {
    let mut drawing = Drawing::default();
    drawing.header.version = AcadVersion::R2004;
    drawing.header.last_saved_by = String::from("a\u{7}^\u{1E} b");
    assert_contains(&drawing, String::from("a^G^ ^^ b"));
}

#[test]
fn normalize_mline_styles() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.objects.len());
    let mut mline = MLine::default();
    mline.style_name = String::from("style name");
    file.entities.push(Entity::new(EntityType::MLine(mline)));
    file.normalize();
    assert_eq!(1, file.objects.len());
    match &file.objects[0].specific {
        &ObjectType::MLineStyle(ref ml) => assert_eq!("style name", ml.style_name),
        _ => panic!("expected an mline style"),
    }
}

#[test]
fn normalize_dimension_styles() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.dim_styles.len());
    file.entities
        .push(Entity::new(EntityType::RadialDimension(RadialDimension {
            dimension_base: DimensionBase {
                dimension_style_name: String::from("style name"),
                ..Default::default()
            },
            ..Default::default()
        })));
    file.normalize();
    assert_eq!(3, file.dim_styles.len());
    assert_eq!("ANNOTATIVE", file.dim_styles[0].name);
    assert_eq!("STANDARD", file.dim_styles[1].name);
    assert_eq!("style name", file.dim_styles[2].name);
}

#[test]
fn normalize_layers() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.layers.len());
    file.header.current_layer = String::from("current layer");
    file.normalize();
    assert_eq!(2, file.layers.len());
    assert_eq!("0", file.layers[0].name);
    assert_eq!("current layer", file.layers[1].name);
}

#[test]
fn normalize_line_types() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.line_types.len());
    file.entities.push(Entity {
        common: EntityCommon {
            line_type_name: String::from("line type"),
            ..Default::default()
        },
        specific: EntityType::Line(Default::default()),
    });
    file.normalize();
    assert_eq!(4, file.line_types.len());
    assert_eq!("BYBLOCK", file.line_types[0].name);
    assert_eq!("BYLAYER", file.line_types[1].name);
    assert_eq!("CONTINUOUS", file.line_types[2].name);
    assert_eq!("line type", file.line_types[3].name);
}

#[test]
fn normalize_text_styles() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.styles.len());
    file.entities
        .push(Entity::new(EntityType::Attribute(Attribute {
            text_style_name: String::from("text style"),
            ..Default::default()
        })));
    file.normalize();
    assert_eq!(3, file.styles.len());
    assert_eq!("ANNOTATIVE", file.styles[0].name);
    assert_eq!("STANDARD", file.styles[1].name);
    assert_eq!("text style", file.styles[2].name);
}

#[test]
fn normalize_view_ports() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.view_ports.len());
    file.normalize();
    assert_eq!(1, file.view_ports.len());
    assert_eq!("*ACTIVE", file.view_ports[0].name);
}

#[test]
fn normalize_views() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.views.len());
    file.objects
        .push(Object::new(ObjectType::PlotSettings(PlotSettings {
            plot_view_name: String::from("some view"),
            ..Default::default()
        })));
    file.normalize();
    assert_eq!(1, file.views.len());
    assert_eq!("some view", file.views[0].name);
}

#[test]
fn normalize_ucs() {
    let mut file = Drawing::default();
    file.clear();
    assert_eq!(0, file.ucss.len());
    file.header.ucs_name = String::from("primary ucs");
    file.normalize();
    assert_eq!(1, file.ucss.len());
    assert_eq!("primary ucs", file.ucss[0].name);
}

#[test]
fn thumbnail_round_trip() {
    // prepare 1x1 px image, red pixel
    let mut imgbuf = image::ImageBuffer::new(1, 1);
    imgbuf.put_pixel(0, 0, image::Rgb([255u8, 0, 0]));
    let thumbnail = DynamicImage::ImageRgb8(imgbuf);

    // write drawing with thumbnail
    let drawing = Drawing {
        header: Header {
            version: AcadVersion::R2000, // thumbnails are only written >= R2000
            ..Default::default()
        },
        thumbnail: Some(thumbnail),
        ..Default::default()
    };
    let drawing_text = to_test_string(&drawing);
    assert!(drawing_text.contains(&vec!["  0", "SECTION", "  2", "THUMBNAILIMAGE",].join("\r\n")));

    // re-read the drawing
    let drawing = parse_drawing(&*drawing_text);
    let thumbnail = drawing.thumbnail.unwrap();
    assert_eq!((1, 1), thumbnail.dimensions());
    assert_eq!(image::Rgba([255u8, 0, 0, 255]), thumbnail.get_pixel(0, 0));
}
