// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use crate::objects::ObjectCommon;
use crate::tables::*;
use crate::Block;

pub(crate) struct HandleTracker {
    next_handle: u32,
}

impl HandleTracker {
    pub fn new(next_handle: u32) -> Self {
        HandleTracker { next_handle }
    }
    pub fn get_app_id_handle(&mut self, item: &AppId) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_block_handle(&mut self, item: &Block) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_block_record_handle(&mut self, item: &BlockRecord) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_dim_style_handle(&mut self, item: &DimStyle) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_layer_handle(&mut self, item: &Layer) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_line_type_handle(&mut self, item: &LineType) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_object_handle(&mut self, item: &ObjectCommon) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_style_handle(&mut self, item: &Style) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_ucs_handle(&mut self, item: &Ucs) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_view_handle(&mut self, item: &View) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_view_port_handle(&mut self, item: &ViewPort) -> u32 {
        self.get_next_handle(item.handle)
    }
    pub fn get_current_next_handle(&self) -> u32 {
        self.next_handle
    }
    fn get_next_handle(&mut self, existing_handle: u32) -> u32 {
        match existing_handle {
            0 => {
                let handle = self.next_handle;
                self.next_handle += 1;
                handle
            }
            _ => existing_handle,
        }
    }
}
