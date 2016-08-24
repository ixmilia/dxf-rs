// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

#[cfg(test)]
#[allow(dead_code)]
pub mod helpers {
    use dxf::*;
    use ::std::io::{BufRead, BufReader, Cursor, Seek, SeekFrom};

    pub fn parse_drawing(s: &str) -> Drawing {
        Drawing::load(s.as_bytes()).ok().unwrap()
    }

    pub fn from_section(section: &str, body: &str) -> Drawing {
        let text = vec!["0", "SECTION", "2", section, body.trim(), "0", "ENDSEC", "0", "EOF"].join("\n");
        parse_drawing(text.trim())
    }

    pub fn to_test_string(drawing: &Drawing) -> String {
        let mut buf = Cursor::new(vec![]);
        drawing.save(&mut buf).ok().unwrap();
        buf.seek(SeekFrom::Start(0)).ok().unwrap();
        let reader = BufReader::new(&mut buf);
        let contents = reader.lines().map(|l| l.unwrap()).fold(String::new(), |a, l| a + l.as_str() + "\r\n");
        println!("{}", contents); // will only be displayed on the console if the test fails
        contents
    }

    pub fn assert_contains(drawing: &Drawing, contents: String) {
        let actual = to_test_string(&drawing);
        assert!(actual.contains(contents.as_str()));
    }
}
