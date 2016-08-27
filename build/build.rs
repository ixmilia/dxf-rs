// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

mod xml_helpers;
mod entity_generator;
mod header_generator;
mod table_generator;

include!("../src/expected_type.rs");

fn main() {
    entity_generator::generate_entities();
    header_generator::generate_header();
    table_generator::generate_tables();
}
