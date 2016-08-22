// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::*;
use self::dxf::enums::*;

mod test_helpers;
use test_helpers::helpers::*;

#[test]
fn totally_empty_file() {
    let _file = Drawing::parse("").ok().unwrap();
}

#[test]
fn empty_file_trailing_newline() {
    let _file = Drawing::parse("0\nEOF\n").ok().unwrap();
}

#[test]
fn empty_file_no_trailing_newline() {
    let _file = Drawing::parse("0\nEOF").ok().unwrap();
}

#[test]
fn unsupported_section() {
    let _file = from_section("UNSUPPORTED_SECTION", vec!["1", "garbage value 1", "2", "garbage value 2"].join("\n").as_str());
}

#[test]
fn read_lf_and_crlf() {
    let code_pairs = vec!["0", "SECTION", "2", "HEADER", "9", "$ACADVER", "1", "AC1027", "0", "ENDSEC", "0", "EOF"];

    let lf_file = Drawing::parse(code_pairs.join("\n").as_str()).ok().unwrap();
    assert_eq!(AcadVersion::R2013, lf_file.header.version);

    let crlf_file = Drawing::parse(code_pairs.join("\r\n").as_str()).ok().unwrap();
    assert_eq!(AcadVersion::R2013, crlf_file.header.version);
}
