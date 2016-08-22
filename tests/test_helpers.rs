// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

#[cfg(test)]
#[allow(dead_code)]
pub mod helpers {
    use dxf::*;

    pub fn from_section(section: &str, body: &str) -> Drawing {
        let text = vec!["0", "SECTION", "2", section, body.trim(), "0", "ENDSEC", "0", "EOF"].join("\n");
        Drawing::parse(text.trim()).ok().unwrap()
    }

    pub fn to_test_string(drawing: &Drawing) -> String {
        let contents = drawing.to_string().ok().unwrap();
        println!("{}", contents); // will only be displayed on the console if the test fails
        contents
    }

    pub fn assert_contains(drawing: &Drawing, contents: String) {
        let actual = to_test_string(&drawing);
        assert!(actual.contains(contents.as_str()));
    }
}
