// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

mod other_helpers;
mod xml_helpers;
mod entity_generator;
mod header_generator;
mod object_generator;
mod table_generator;
mod test_helper_generator;

use std::fs::File;
use std::io::Write;

include!("../src/expected_type.rs");

fn main() {
    let _ = std::fs::create_dir("src/generated/"); // might fail if it's already there

    let mut file = File::create("src/generated/mod.rs").ok().unwrap();
    file.write_all("// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

pub mod entities;
pub mod header;
pub mod objects;
pub mod tables;
".as_bytes()).ok().unwrap();

    entity_generator::generate_entities();
    header_generator::generate_header();
    object_generator::generate_objects();
    table_generator::generate_tables();

    test_helper_generator::generate_test_helpers();
}
