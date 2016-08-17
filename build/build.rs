// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

mod entity_generator;
mod header_generator;

include!("../src/expected_type.rs");

fn main() {
    entity_generator::generate_entities();
    header_generator::generate_header();
}
