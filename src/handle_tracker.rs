// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use crate::Block;

pub(crate) struct HandleTracker {
    next_handle: u32,
}

impl HandleTracker {
    pub fn new(next_handle: u32) -> Self {
        HandleTracker { next_handle }
    }
    pub fn get_block_handle(&mut self, item: &Block) -> u32 {
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
