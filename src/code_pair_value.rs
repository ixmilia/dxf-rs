// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::fmt;
use std::fmt::{
    Debug,
    Formatter,
};

#[doc(hidden)]
pub enum CodePairValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Short(i16),
    Double(f64),
    Str(String),
}

impl CodePairValue {
    pub fn assert_bool(&self) -> bool {
        match self {
            &CodePairValue::Boolean(b) => b,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_i64(&self) -> i64 {
        match self {
            &CodePairValue::Long(l) => l,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_i32(&self) -> i32 {
        match self {
            &CodePairValue::Integer(i) => i,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_f64(&self) -> f64 {
        match self {
            &CodePairValue::Double(f) => f,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_string(&self) -> String {
        match self {
            &CodePairValue::Str(ref s) => s.clone(),
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
    pub fn assert_i16(&self) -> i16 {
        match self {
            &CodePairValue::Short(s) => s,
            _ => panic!("this should never have happened, please file a bug"),
        }
    }
}

impl Clone for CodePairValue {
    fn clone(&self) -> Self {
        match self {
            &CodePairValue::Boolean(b) => CodePairValue::Boolean(b),
            &CodePairValue::Integer(i) => CodePairValue::Integer(i),
            &CodePairValue::Long(l) => CodePairValue::Long(l),
            &CodePairValue::Short(s) => CodePairValue::Short(s),
            &CodePairValue::Double(d) => CodePairValue::Double(d),
            &CodePairValue::Str(ref s) => CodePairValue::Str(String::from(s.as_str())),
        }
    }
}

impl Debug for CodePairValue {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            &CodePairValue::Boolean(b) => write!(formatter, "{}", if b { 1 } else { 0 }),
            &CodePairValue::Integer(i) => write!(formatter, "{: >9}", i),
            &CodePairValue::Long(l) => write!(formatter, "{}", l),
            &CodePairValue::Short(s) => write!(formatter, "{: >6}", s),
            &CodePairValue::Double(d) => write!(formatter, "{}", format_f64(d)),
            &CodePairValue::Str(ref s) => write!(formatter, "{}", s),
        }
    }
}

/// Formats an `f64` value with up to 12 digits of precision, ensuring at least one trailing digit after the decimal.
fn format_f64(val: f64) -> String {
    // format with 12 digits of precision
    let mut val = format!("{:.12}", val);

    // trim trailing zeros
    while val.ends_with('0') {
        val.pop();
    }

    // ensure it doesn't end with a decimal
    if val.ends_with('.') {
        val.push('0');
    }

    val
}
