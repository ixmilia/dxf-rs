// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

extern crate dxf;

mod test_helpers;
use test_helpers::*;

#[test]
fn unsupported_section() {
    let _file = from_section("UNSUPPORTED_SECTION", vec!["1", "garbage value 1", "2", "garbage value 2"].join("\n").as_str());
}
