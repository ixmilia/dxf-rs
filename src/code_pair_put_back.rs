use crate::code_pair_iter::CodePairIter;
use crate::dxf_result::DxfResult;
use crate::CodePair;

pub(crate) struct CodePairPutBack {
    top: Vec<DxfResult<CodePair>>,
    iter: Box<dyn CodePairIter>,
}

impl CodePairPutBack {
    pub fn from_code_pair_iter(iter: Box<dyn CodePairIter>) -> Self {
        CodePairPutBack { top: vec![], iter }
    }
    pub fn put_back(&mut self, item: DxfResult<CodePair>) {
        self.top.push(item);
    }
    pub fn read_as_utf8(&mut self) {
        self.iter.read_as_utf8()
    }
}

impl Iterator for CodePairPutBack {
    type Item = DxfResult<CodePair>;

    fn next(&mut self) -> Option<DxfResult<CodePair>> {
        if self.top.is_empty() {
            loop {
                let pair = self.iter.next();
                match pair {
                    Some(Ok(CodePair { code: 999, .. })) => (), // a 999 comment code, try again
                    _ => return pair,
                }
            }
        } else {
            Some(self.top.pop().unwrap())
        }
    }
}
