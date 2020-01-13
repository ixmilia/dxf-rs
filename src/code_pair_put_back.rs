// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Read;

use crate::code_pair_iter::CodePairIter;
use crate::dxf_result::DxfResult;
use crate::CodePair;

pub(crate) struct CodePairPutBack<T: Read> {
    top: Vec<DxfResult<CodePair>>,
    iter: CodePairIter<T>,
}

impl<T: Read> CodePairPutBack<T> {
    pub fn from_code_pair_iter(iter: CodePairIter<T>) -> Self
    where
        T: Read,
    {
        CodePairPutBack { top: vec![], iter }
    }
    pub fn put_back(&mut self, item: DxfResult<CodePair>) {
        self.top.push(item);
    }
    pub fn read_as_utf8(&mut self) {
        self.iter.read_as_utf8()
    }
}

impl<T: Read> Iterator for CodePairPutBack<T> {
    type Item = DxfResult<CodePair>;

    fn next(&mut self) -> Option<DxfResult<CodePair>> {
        if self.top.is_empty() {
            self.iter.next()
        } else {
            Some(self.top.pop().unwrap())
        }
    }
}
