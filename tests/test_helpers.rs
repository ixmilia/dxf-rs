// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

#[cfg(test)]
#[allow(dead_code)]
pub mod helpers {
    use dxf::dxf_file::*;

    pub fn from_section(section: &str, body: &str) -> DxfFile {
        let text = vec!["0", "SECTION", "2", section, body.trim(), "0", "ENDSEC", "0", "EOF"].join("\n");
        DxfFile::parse(text.trim()).ok().unwrap()
    }

    pub fn to_test_string(file: &DxfFile) -> String {
        let contents = file.to_string().ok().unwrap();
        println!("{}", contents); // will only be displayed on the console if the test fails
        contents
    }

    pub fn assert_contains(file: &DxfFile, contents: String) {
        let actual = to_test_string(&file);
        assert!(actual.contains(contents.as_str()));
    }
}
