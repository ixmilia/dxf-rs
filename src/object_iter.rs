use crate::code_pair_put_back::CodePairPutBack;
use crate::objects::Object;

pub(crate) struct ObjectIter<'a> {
    pub iter: &'a mut CodePairPutBack,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        match Object::read(self.iter) {
            Ok(Some(o)) => Some(o),
            Ok(None) | Err(_) => None,
        }
    }
}
