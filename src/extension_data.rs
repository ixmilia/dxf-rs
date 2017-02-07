// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Write;
use itertools::PutBack;

use ::{
    CodePair,
    DxfError,
    DxfResult,
};

use ::code_pair_writer::CodePairWriter;

#[doc(hidden)] pub const EXTENSION_DATA_GROUP: i32 = 102;

/// Represents an application name and a collection of extension group data in the form of `CodePair`s.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtensionGroup {
    pub application_name: String,
    pub items: Vec<ExtensionGroupItem>,
}

/// Represents a single piece of extension data or a named group.
#[derive(Clone, Debug, PartialEq)]
pub enum ExtensionGroupItem {
    CodePair(CodePair),
    Group(ExtensionGroup),
}

impl ExtensionGroup {
    #[doc(hidden)]
    pub fn read_group<I>(application_name: String, iter: &mut PutBack<I>) -> DxfResult<ExtensionGroup>
        where I: Iterator<Item = DxfResult<CodePair>> {

        if !application_name.starts_with("{") {
            return Err(DxfError::ParseError);
        }
        let mut application_name = application_name.clone();
        application_name.remove(0);

        let mut items = vec![];
        loop {
            let pair = match iter.next() {
                Some(Ok(pair)) => pair,
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            };
            if pair.code == EXTENSION_DATA_GROUP {
                let name = try!(pair.value.assert_string());
                if name == "}" {
                    // end of group
                    break;
                }
                else if name.starts_with("{") {
                    // nested group
                    let sub_group = try!(ExtensionGroup::read_group(name, iter));
                    items.push(ExtensionGroupItem::Group(sub_group));
                }
                else {
                    return Err(DxfError::UnexpectedCodePair(pair, String::from("expected an extension start or end pair")));
                }
            }
            else {
                items.push(ExtensionGroupItem::CodePair(pair));
            }
        }
        Ok(ExtensionGroup { application_name: application_name, items: items })
    }
    #[doc(hidden)]
    pub fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        if self.items.len() > 0 {
            let mut full_group_name = String::new();
            full_group_name.push('{');
            full_group_name.push_str(&self.application_name);
            try!(writer.write_code_pair(&CodePair::new_string(EXTENSION_DATA_GROUP, &full_group_name)));
            for item in &self.items {
                match item {
                    &ExtensionGroupItem::CodePair(ref pair) => try!(writer.write_code_pair(pair)),
                    &ExtensionGroupItem::Group(ref group) => try!(group.write(writer)),
                }
            }
            try!(writer.write_code_pair(&CodePair::new_str(EXTENSION_DATA_GROUP, "}")));
        }
        Ok(())
    }
}
