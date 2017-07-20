// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use entities::*;
use objects::*;

pub enum DrawingItem<'a> {
    Entity(&'a Entity),
    Object(&'a Object),
}
