// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

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
    pub(crate) fn from_vec(&mut self, values: &Vec<f64>) {
        self.m11 = TransformationMatrix::get_value_or_default(&values, 0);
        self.m12 = TransformationMatrix::get_value_or_default(&values, 1);
        self.m13 = TransformationMatrix::get_value_or_default(&values, 2);
        self.m14 = TransformationMatrix::get_value_or_default(&values, 3);
        self.m21 = TransformationMatrix::get_value_or_default(&values, 4);
        self.m22 = TransformationMatrix::get_value_or_default(&values, 5);
        self.m23 = TransformationMatrix::get_value_or_default(&values, 6);
        self.m24 = TransformationMatrix::get_value_or_default(&values, 7);
        self.m31 = TransformationMatrix::get_value_or_default(&values, 8);
        self.m32 = TransformationMatrix::get_value_or_default(&values, 9);
        self.m33 = TransformationMatrix::get_value_or_default(&values, 10);
        self.m34 = TransformationMatrix::get_value_or_default(&values, 11);
        self.m41 = TransformationMatrix::get_value_or_default(&values, 12);
        self.m42 = TransformationMatrix::get_value_or_default(&values, 13);
        self.m43 = TransformationMatrix::get_value_or_default(&values, 14);
        self.m44 = TransformationMatrix::get_value_or_default(&values, 15);
    }
    pub(crate) fn get_values(&self) -> Vec<f64> {
        vec![
            self.m11, self.m12, self.m13, self.m14, self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34, self.m41, self.m42, self.m43, self.m44,
        ]
    }
    pub(crate) fn get_4x3_values_row_major(&self) -> Vec<f64> {
        vec![
            self.m11, self.m21, self.m31, self.m12, self.m22, self.m32, self.m13, self.m23,
            self.m33, self.m14, self.m24, self.m34,
        ]
    }
}

// private implementation
impl TransformationMatrix {
    fn get_value_or_default(values: &Vec<f64>, index: usize) -> f64 {
        if values.len() > index {
            values[index]
        } else {
            0.0
        }
    }
}
