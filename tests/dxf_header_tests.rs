// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::enums::*;

extern crate chrono;
use self::chrono::*;

mod test_helpers;
use test_helpers::helpers::*;

#[test]
fn specific_header_values() {
    let file = from_section("HEADER", "
  9
$ACADMAINTVER
 70
16
  9
$ACADVER
  1
AC1012
  9
$ANGBASE
 50
5.5E1
  9
$ANGDIR
 70
1
  9
$ATTMODE
 70
1
  9
$AUNITS
 70
3
  9
$AUPREC
 70
7
  9
$CLAYER
  8
<current layer>
  9
$LUNITS
 70
6
  9
$LUPREC
 70
7".trim_left());
    assert_eq!(16, file.header.maintenance_version);
    assert_eq!(DxfAcadVersion::R13, file.header.version);
    assert_eq!(55.0, file.header.angle_zero_direction);
    assert_eq!(DxfAngleDirection::Clockwise, file.header.angle_direction);
    assert_eq!(DxfAttributeVisibility::Normal, file.header.attribute_visibility);
    assert_eq!(DxfAngleFormat::Radians, file.header.angle_unit_format);
    assert_eq!(7, file.header.angle_unit_precision);
    assert_eq!("<current layer>", file.header.current_layer);
    assert_eq!(DxfUnitFormat::Architectural, file.header.unit_format);
    assert_eq!(7, file.header.unit_precision);
}

#[test]
fn date_conversion_read() {
    // from AutoDesk spec: 2451544.91568287 = 31 December 1999, 9:58:35PM
    let file = from_section("HEADER", vec!["  9", "$TDCREATE", " 40", "2451544.91568287"].join("\r\n").as_str());
    assert_eq!(Local.ymd(1999, 12, 31).and_hms(21, 58, 35), file.header.creation_date);
}

#[test]
fn date_conversion_write() {
    // from AutoDesk spec: 2451544.91568287[0429] = 31 December 1999, 9:58:35PM
    let mut file = DxfFile::new();
    file.header.creation_date = Local.ymd(1999, 12, 31).and_hms(21, 58, 35);
    assert!(to_test_string(&file).contains(vec!["  9", "$TDCREATE", " 40", "2451544.915682870429"].join("\r\n").as_str()));
}

#[test]
fn read_alternate_version() {
    let file = from_section("HEADER", vec!["  9", "$ACADVER", "  1", "15.05"].join("\r\n").as_str());
    assert_eq!(DxfAcadVersion::R2000, file.header.version);
}

#[test]
fn read_multi_value_variable() {
    let file = from_section("HEADER", vec!["9", "$EXTMIN", "10", "1.1", "20", "2.2", "30", "3.3"].join("\r\n").as_str());
    assert_eq!(DxfPoint::new(1.1, 2.2, 3.3), file.header.minimum_drawing_extents)
}

#[test]
fn write_multiple_value_variable() {
    let mut file = DxfFile::new();
    file.header.minimum_drawing_extents = DxfPoint::new(1.1, 2.2, 3.3);
    assert!(to_test_string(&file).contains(vec!["9", "$EXTMIN", " 10", "1.100000000000", " 20", "2.200000000000", " 30", "3.300000000000"].join("\r\n").as_str()));
}

#[test]
fn write_header_with_invalid_values() {
    let mut file = DxfFile::new();
    file.header.default_text_height = -1.0; // $TEXTSIZE; normalized to 0.2
    file.header.trace_width = 0.0; // $TRACEWID; normalized to 0.05
    file.header.text_style = String::new(); // $TEXTSTYLE; normalized to "STANDARD"
    file.header.current_layer = String::new(); // $CLAYER; normalized to "0"
    file.header.current_entity_linetype = String::new(); // $CELTYPE; normalized to "BYLAYER"
    file.header.dimension_style_name = String::new(); // $DIMSTYLE; normalized to "STANDARD"
    file.header.file_name = String::new(); // $MENU; normalized to "."
    assert_contains(&file, vec!["  9", "$TEXTSIZE", " 40", "0.200000000000"].join("\r\n"));
    assert_contains(&file, vec!["  9", "$TRACEWID", " 40", "0.050000000000"].join("\r\n"));
    assert_contains(&file, vec!["  9", "$TEXTSTYLE", "  7", "STANDARD"].join("\r\n"));
    assert_contains(&file, vec!["  9", "$CLAYER", "  8", "0"].join("\r\n"));
    assert_contains(&file, vec!["  9", "$CELTYPE", "  6", "BYLAYER"].join("\r\n"));
    assert_contains(&file, vec!["  9", "$DIMSTYLE", "  2", "STANDARD"].join("\r\n"));
    assert_contains(&file, vec!["  9", "$MENU", "  1", "."].join("\r\n"));
}

#[test]
fn read_header_flags() {
    let file = from_section("HEADER", vec!["9", "$OSMODE", "70", "12"].join("\r\n").as_str());
    assert!(!file.header.get_end_point_snap());
    assert!(!file.header.get_mid_point_snap());
    assert!(file.header.get_center_snap());
    assert!(file.header.get_node_snap());
    assert!(!file.header.get_quadrant_snap());
    assert!(!file.header.get_intersection_snap());
    assert!(!file.header.get_insertion_snap());
    assert!(!file.header.get_perpendicular_snap());
    assert!(!file.header.get_tangent_snap());
    assert!(!file.header.get_nearest_snap());
    assert!(!file.header.get_apparent_intersection_snap());
    assert!(!file.header.get_extension_snap());
    assert!(!file.header.get_parallel_snap());
}

#[test]
fn write_header_flags() {
    let mut file = DxfFile::new();
    file.header.set_end_point_snap(false);
    file.header.set_mid_point_snap(false);
    file.header.set_center_snap(true);
    file.header.set_node_snap(true);
    file.header.set_quadrant_snap(false);
    file.header.set_intersection_snap(false);
    file.header.set_insertion_snap(false);
    file.header.set_perpendicular_snap(false);
    file.header.set_tangent_snap(false);
    file.header.set_nearest_snap(false);
    file.header.set_apparent_intersection_snap(false);
    file.header.set_extension_snap(false);
    file.header.set_parallel_snap(false);
    assert_contains(&file, vec!["  9", "$OSMODE", " 70", "12"].join("\r\n"));
}
