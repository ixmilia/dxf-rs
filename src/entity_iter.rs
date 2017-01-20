// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use itertools::PutBack;
use ::{
    CodePair,
    DxfResult,
};
use ::entities::Entity;

#[doc(hidden)]
pub struct EntityIter<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> {
    pub iter: &'a mut PutBack<I>,
}

impl<'a, I: 'a + Iterator<Item = DxfResult<CodePair>>> Iterator for EntityIter<'a, I> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        match Entity::read(self.iter) {
            Ok(Some(e)) => Some(e),
            Ok(None) | Err(_) => None,
        }
    }
}
