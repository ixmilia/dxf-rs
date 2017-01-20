// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use itertools::PutBack;
use ::{
    CodePair,
    DxfResult,
};
use ::objects::Object;

#[doc(hidden)]
pub struct ObjectIter<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> {
    pub iter: &'a mut PutBack<I>,
}

impl<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> Iterator for ObjectIter<'a, I> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        match Object::read(self.iter) {
            Ok(Some(o)) => Some(o),
            Ok(None) | Err(_) => None,
        }
    }
}
