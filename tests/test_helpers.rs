// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;
use self::dxf::dxf_file::*;

#[cfg(test)]
pub fn from_section(section: &str, body: &str) -> DxfFile {
    let text = vec!["0", "SECTION", "2", section, body.trim(), "0", "ENDSEC", "0", "EOF"].join("\n");
    DxfFile::parse(text.trim()).ok().unwrap()
}
