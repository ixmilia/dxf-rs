#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Handle(pub u64);

impl Handle {
    pub fn empty() -> Self {
        Handle(0)
    }
    pub fn next_handle_value(self) -> Self {
        Handle(self.0 + 1)
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn as_string(self) -> String {
        format!("{:X}", self.0)
    }
}
