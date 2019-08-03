// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

use std::io::{Cursor, Seek, SeekFrom};
use self::dxf::*;

mod test_helpers;

#[test]
fn dont_write_utf8_bom() {
    let drawing = Drawing::default();
    let mut buf = Cursor::new(vec![]);
    drawing.save(&mut buf).ok().unwrap();
    buf.seek(SeekFrom::Start(0)).ok().unwrap();
    let vec = buf.into_inner();

    // file should start directly with a code, not a UTF-8 BOM
    assert_eq!(b' ', vec[0]);
    assert_eq!(b' ', vec[1]);
    assert_eq!(b'0', vec[2]);
}
