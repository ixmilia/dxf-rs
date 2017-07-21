// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use entities::*;
use objects::*;

pub enum DrawingItem<'a> {
    Entity(&'a Entity),
    Object(&'a Object),
}

impl<'a> DrawingItem<'a> {
    pub fn get_handle(&self) -> u32 {
        match self {
            &DrawingItem::Entity(&Entity { ref common, .. }) => common.handle,
            &DrawingItem::Object(&Object { ref common, .. }) => common.handle,
        }
    }
}

pub enum DrawingItemMut<'a> {
    Entity(&'a mut Entity),
    Object(&'a mut Object),
}

impl<'a> DrawingItemMut<'a> {
    pub fn get_handle(&self) -> u32 {
        match self {
            &DrawingItemMut::Entity(&mut Entity { ref common, .. }) => common.handle,
            &DrawingItemMut::Object(&mut Object { ref common, .. }) => common.handle,
        }
    }
    pub fn set_handle(&mut self, handle: u32) {
        match self {
            &mut DrawingItemMut::Entity(&mut Entity { ref mut common, .. }) => common.handle = handle,
            &mut DrawingItemMut::Object(&mut Object { ref mut common, .. }) => common.handle = handle,
        }
    }
    pub fn to_drawing_item(&self) -> DrawingItem {
        match self {
            &DrawingItemMut::Entity(ref ent) => DrawingItem::Entity(ent),
            &DrawingItemMut::Object(ref obj) => DrawingItem::Object(obj),
        }
    }
}
