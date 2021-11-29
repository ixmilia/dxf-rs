use crate::{CodePair, DxfError, DxfResult};

use crate::code_pair_put_back::CodePairPutBack;

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
    pub(crate) fn read_group(
        application_name: String,
        iter: &mut CodePairPutBack,
        offset: usize,
    ) -> DxfResult<ExtensionGroup> {
        if !application_name.starts_with('{') {
            return Err(DxfError::ParseError(offset));
        }
        let mut application_name = application_name;
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
                } else if name.starts_with('{') {
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
            application_name,
            items,
        })
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>) {
        if !self.items.is_empty() {
            let mut full_group_name = String::new();
            full_group_name.push('{');
            full_group_name.push_str(&self.application_name);
            pairs.push(CodePair::new_string(EXTENSION_DATA_GROUP, &full_group_name));
            for item in &self.items {
                match item {
                    ExtensionGroupItem::CodePair(pair) => pairs.push(pair.clone()),
                    ExtensionGroupItem::Group(ref group) => group.add_code_pairs(pairs),
                }
            }
            pairs.push(CodePair::new_str(EXTENSION_DATA_GROUP, "}"));
        }
    }
}
