/// Represents a line weight.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct LineWeight {
    raw_value: i16,
}

impl LineWeight {
    pub(crate) fn from_raw_value(v: i16) -> LineWeight {
        LineWeight { raw_value: v }
    }
    /// Creates a new `LineWeight` that defaults back to the containing block's line weight.
    pub fn by_block() -> LineWeight {
        LineWeight::from_raw_value(-1)
    }
    /// Creates a new `LineWeight` that defaults back to the item's layer's line weight.
    pub fn by_layer() -> LineWeight {
        LineWeight::from_raw_value(-2)
    }
    /// Gets the raw value of the `LineWeight`.
    pub fn raw_value(&self) -> i16 {
        self.raw_value
    }
    /// Returns `true` if the `LineWeight` is BYBLOCK.
    pub fn is_by_block(&self) -> bool {
        self.raw_value == -1
    }
    /// Returns `true` if the `LineWeight` is BYLAYER.
    pub fn is_by_layer(&self) -> bool {
        self.raw_value == -2
    }
}
