// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

mod entity_generator;
mod header_generator;
mod object_generator;
mod other_helpers;
mod table_generator;
mod test_helper_generator;
mod xml_helpers;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

include!("../src/expected_type.rs");

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_dir = Path::new(&out_dir).join("generated");
    let _ = std::fs::create_dir(&generated_dir); // might fail if it's already there

    let mut file = File::create(generated_dir.join("mod.rs")).ok().unwrap();
    file.write_all("// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

pub mod entities;
pub mod header;
pub mod objects;
pub mod tables;
".as_bytes()).ok().unwrap();

    entity_generator::generate_entities(&generated_dir);
    header_generator::generate_header(&generated_dir);
    object_generator::generate_objects(&generated_dir);
    table_generator::generate_tables(&generated_dir);

    test_helper_generator::generate_test_helpers(&generated_dir);

    // only watch the `build/` and `spec/` directories
    println!("cargo:rerun-if-changed=build/");
    println!("cargo:rerun-if-changed=spec/");
}
