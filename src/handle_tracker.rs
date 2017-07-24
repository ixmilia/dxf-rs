// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use ::entities::EntityCommon;

pub(crate) struct HandleTracker {
    next_handle: u32,
}

impl HandleTracker {
    pub fn new(next_handle: u32) -> Self {
        HandleTracker {
            next_handle: next_handle,
        }
    }
    pub fn get_entity_handle(&mut self, item: &EntityCommon) -> u32 {
        match item.handle {
            0 => {
                let handle = self.next_handle;
                self.next_handle += 1;
                handle
            },
            _ => item.handle,
        }
    }
}
