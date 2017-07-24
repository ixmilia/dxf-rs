// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use entities::*;
use objects::*;
use tables::*;

pub enum DrawingItem<'a> {
    AppId(&'a AppId),
    BlockRecord(&'a BlockRecord),
    DimStyle(&'a DimStyle),
    Entity(&'a Entity),
    Layer(&'a Layer),
    LineType(&'a LineType),
    Object(&'a Object),
    Style(&'a Style),
    Ucs(&'a Ucs),
    View(&'a View),
    ViewPort(&'a ViewPort),
}

impl<'a> DrawingItem<'a> {
    pub fn get_handle(&self) -> u32 {
        match self {
            &DrawingItem::AppId(ref app_id) => app_id.handle,
            &DrawingItem::BlockRecord(ref br) => br.handle,
            &DrawingItem::DimStyle(ref ds) => ds.handle,
            &DrawingItem::Entity(&Entity { ref common, .. }) => common.handle,
            &DrawingItem::Layer(ref l) => l.handle,
            &DrawingItem::LineType(ref l) => l.handle,
            &DrawingItem::Object(&Object { ref common, .. }) => common.handle,
            &DrawingItem::Style(ref s) => s.handle,
            &DrawingItem::Ucs(ref u) => u.handle,
            &DrawingItem::View(ref v) => v.handle,
            &DrawingItem::ViewPort(ref v) => v.handle,
        }
    }
}

pub enum DrawingItemMut<'a> {
    AppId(&'a mut AppId),
    BlockRecord(&'a mut BlockRecord),
    DimStyle(&'a mut DimStyle),
    Entity(&'a mut Entity),
    Layer(&'a mut Layer),
    LineType(&'a mut LineType),
    Object(&'a mut Object),
    Style(&'a mut Style),
    Ucs(&'a mut Ucs),
    View(&'a mut View),
    ViewPort(&'a mut ViewPort),
}

impl<'a> DrawingItemMut<'a> {
    pub fn get_handle(&self) -> u32 {
        match self {
            &DrawingItemMut::AppId(ref app_id) => app_id.handle,
            &DrawingItemMut::BlockRecord(ref br) => br.handle,
            &DrawingItemMut::DimStyle(ref ds) => ds.handle,
            &DrawingItemMut::Entity(&mut Entity { ref common, .. }) => common.handle,
            &DrawingItemMut::Layer(ref l) => l.handle,
            &DrawingItemMut::LineType(ref l) => l.handle,
            &DrawingItemMut::Object(&mut Object { ref common, .. }) => common.handle,
            &DrawingItemMut::Style(ref s) => s.handle,
            &DrawingItemMut::Ucs(ref u) => u.handle,
            &DrawingItemMut::View(ref v) => v.handle,
            &DrawingItemMut::ViewPort(ref v) => v.handle,
        }
    }
    pub fn set_handle(&mut self, handle: u32) {
        match self {
            &mut DrawingItemMut::AppId(ref mut app_id) => app_id.handle = handle,
            &mut DrawingItemMut::BlockRecord(ref mut br) => br.handle = handle,
            &mut DrawingItemMut::DimStyle(ref mut ds) => ds.handle = handle,
            &mut DrawingItemMut::Entity(&mut Entity { ref mut common, .. }) => common.handle = handle,
            &mut DrawingItemMut::Layer(ref mut l) => l.handle = handle,
            &mut DrawingItemMut::LineType(ref mut l) => l.handle = handle,
            &mut DrawingItemMut::Object(&mut Object { ref mut common, .. }) => common.handle = handle,
            &mut DrawingItemMut::Style(ref mut s) => s.handle = handle,
            &mut DrawingItemMut::Ucs(ref mut u) => u.handle = handle,
            &mut DrawingItemMut::View(ref mut v) => v.handle = handle,
            &mut DrawingItemMut::ViewPort(ref mut v) => v.handle = handle,
        }
    }
    pub fn to_drawing_item(&self) -> DrawingItem {
        match self {
            &DrawingItemMut::AppId(ref app_id) => DrawingItem::AppId(app_id),
            &DrawingItemMut::BlockRecord(ref br) => DrawingItem::BlockRecord(br),
            &DrawingItemMut::DimStyle(ref ds) => DrawingItem::DimStyle(ds),
            &DrawingItemMut::Entity(ref ent) => DrawingItem::Entity(ent),
            &DrawingItemMut::Layer(ref l) => DrawingItem::Layer(l),
            &DrawingItemMut::LineType(ref l) => DrawingItem::LineType(l),
            &DrawingItemMut::Object(ref obj) => DrawingItem::Object(obj),
            &DrawingItemMut::Style(ref s) => DrawingItem::Style(s),
            &DrawingItemMut::Ucs(ref u) => DrawingItem::Ucs(u),
            &DrawingItemMut::View(ref v) => DrawingItem::View(v),
            &DrawingItemMut::ViewPort(ref v) => DrawingItem::ViewPort(v),
        }
    }
}
