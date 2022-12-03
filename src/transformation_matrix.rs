/// Applies a transformation to a point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct TransformationMatrix {
    pub m11: f64,
    pub m12: f64,
    pub m13: f64,
    pub m14: f64,
    pub m21: f64,
    pub m22: f64,
    pub m23: f64,
    pub m24: f64,
    pub m31: f64,
    pub m32: f64,
    pub m33: f64,
    pub m34: f64,
    pub m41: f64,
    pub m42: f64,
    pub m43: f64,
    pub m44: f64,
}

// public implementation
impl TransformationMatrix {
    pub fn identity() -> Self {
        TransformationMatrix {
            m11: 1.0,
            m22: 1.0,
            m33: 1.0,
            m44: 1.0,
            ..Default::default()
        }
    }
}

// internal visibility only
impl TransformationMatrix {
    pub(crate) fn from_vec(values: &[f64]) -> Self {
        TransformationMatrix {
            m11: TransformationMatrix::value_or_default(values, 0),
            m12: TransformationMatrix::value_or_default(values, 1),
            m13: TransformationMatrix::value_or_default(values, 2),
            m14: TransformationMatrix::value_or_default(values, 3),
            m21: TransformationMatrix::value_or_default(values, 4),
            m22: TransformationMatrix::value_or_default(values, 5),
            m23: TransformationMatrix::value_or_default(values, 6),
            m24: TransformationMatrix::value_or_default(values, 7),
            m31: TransformationMatrix::value_or_default(values, 8),
            m32: TransformationMatrix::value_or_default(values, 9),
            m33: TransformationMatrix::value_or_default(values, 10),
            m34: TransformationMatrix::value_or_default(values, 11),
            m41: TransformationMatrix::value_or_default(values, 12),
            m42: TransformationMatrix::value_or_default(values, 13),
            m43: TransformationMatrix::value_or_default(values, 14),
            m44: TransformationMatrix::value_or_default(values, 15),
        }
    }
    pub(crate) fn values(&self) -> Vec<f64> {
        vec![
            self.m11, self.m12, self.m13, self.m14, self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34, self.m41, self.m42, self.m43, self.m44,
        ]
    }
    pub(crate) fn values_row_major_4x3(&self) -> Vec<f64> {
        vec![
            self.m11, self.m21, self.m31, self.m12, self.m22, self.m32, self.m13, self.m23,
            self.m33, self.m14, self.m24, self.m34,
        ]
    }
}

// private implementation
impl TransformationMatrix {
    fn value_or_default(values: &[f64], index: usize) -> f64 {
        if values.len() > index {
            values[index]
        } else {
            0.0
        }
    }
}
