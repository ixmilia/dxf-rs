// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use Point;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum DataTableValue {
    Boolean(bool),
    Integer(i32),
    Double(f64),
    Str(String),
    Point2D(Point),
    Point3D(Point),
    Handle(u32),
}
