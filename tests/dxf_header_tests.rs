// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::dxf_file::enums::*;

extern crate chrono;
use self::chrono::*;

mod test_helpers;
use test_helpers::*;

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
fn date_conversion() {
    let file = from_section("HEADER", "
  9
$TDCREATE
 40
2451544.91568287
".trim_left());
    assert_eq!(Local.ymd(1999, 12, 31).and_hms(21, 58, 35), file.header.creation_date);
    // TODO: test writing the date
}

#[test]
fn read_alternate_version() {
    let file = from_section("HEADER", "
  9
$ACADVER
  1
15.05
".trim_left());
    assert_eq!(DxfAcadVersion::R2000, file.header.version);
}
