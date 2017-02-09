// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use ::Color;
use tables::*;
use ::helper_functions::*;

//------------------------------------------------------------------------------
//                                                                         Layer
//------------------------------------------------------------------------------
impl Layer {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        default_if_empty(&mut self.line_type_name, "CONTINUOUS");
        match self.color.get_raw_value() {
            0 | 256 => self.color = Color::from_raw_value(7), // BYLAYER and BYBLOCK aren't valid layer colors
            _ => (),
        }
    }
}

//------------------------------------------------------------------------------
//                                                                         Style
//------------------------------------------------------------------------------
impl Style {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.text_height, 0.0);
        ensure_positive_or_default(&mut self.width_factor, 1.0);
    }
}

//------------------------------------------------------------------------------
//                                                                          View
//------------------------------------------------------------------------------
impl View {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.view_height, 1.0);
        ensure_positive_or_default(&mut self.view_width, 1.0);
        ensure_positive_or_default(&mut self.lens_length, 1.0);
    }
}

//------------------------------------------------------------------------------
//                                                                      ViewPort
//------------------------------------------------------------------------------
impl ViewPort {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.snap_spacing.x, 1.0);
        ensure_positive_or_default(&mut self.snap_spacing.y, 1.0);
        ensure_positive_or_default(&mut self.snap_spacing.z, 1.0);
        ensure_positive_or_default(&mut self.grid_spacing.x, 1.0);
        ensure_positive_or_default(&mut self.grid_spacing.y, 1.0);
        ensure_positive_or_default(&mut self.grid_spacing.z, 1.0);
        ensure_positive_or_default(&mut self.view_height, 1.0);
        ensure_positive_or_default(&mut self.view_port_aspect_ratio, 1.0);
        ensure_positive_or_default(&mut self.lens_length, 50.0);
        ensure_positive_or_default_i16(&mut self.ucs_icon, 3);
        ensure_positive_or_default_i32(&mut self.circle_sides, 1000);
    }
}
