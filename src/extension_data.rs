// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use itertools::PutBack;
use std::io::Write;

use {CodePair, DxfError, DxfResult};

use code_pair_writer::CodePairWriter;

pub(crate) const EXTENSION_DATA_GROUP: i32 = 102;

/// Represents an application name and a collection of extension group data in the form of `CodePair`s.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ExtensionGroup {
    pub application_name: String,
    pub items: Vec<ExtensionGroupItem>,
}

/// Represents a single piece of extension data or a named group.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum ExtensionGroupItem {
    CodePair(CodePair),
    Group(ExtensionGroup),
}

impl ExtensionGroup {
    pub(crate) fn read_group<I>(
        application_name: String,
        iter: &mut PutBack<I>,
        offset: usize,
    ) -> DxfResult<ExtensionGroup>
    where
        I: Iterator<Item = DxfResult<CodePair>>,
    {
        if !application_name.starts_with("{") {
            return Err(DxfError::ParseError(offset));
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
                let name = pair.assert_string()?;
                if name == "}" {
                    // end of group
                    break;
                } else if name.starts_with("{") {
                    // nested group
                    let sub_group = ExtensionGroup::read_group(name, iter, pair.offset)?;
                    items.push(ExtensionGroupItem::Group(sub_group));
                } else {
                    return Err(DxfError::UnexpectedCodePair(
                        pair,
                        String::from("expected an extension start or end pair"),
                    ));
                }
            } else {
                items.push(ExtensionGroupItem::CodePair(pair));
            }
        }
        Ok(ExtensionGroup {
            application_name: application_name,
            items: items,
        })
    }
    pub(crate) fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write,
    {
        if self.items.len() > 0 {
            let mut full_group_name = String::new();
            full_group_name.push('{');
            full_group_name.push_str(&self.application_name);
            writer.write_code_pair(&CodePair::new_string(
                EXTENSION_DATA_GROUP,
                &full_group_name,
            ))?;
            for item in &self.items {
                match item {
                    &ExtensionGroupItem::CodePair(ref pair) => writer.write_code_pair(pair)?,
                    &ExtensionGroupItem::Group(ref group) => group.write(writer)?,
                }
            }
            writer.write_code_pair(&CodePair::new_str(EXTENSION_DATA_GROUP, "}"))?;
        }
        Ok(())
    }
}
