use crate::entities::*;
use crate::objects::*;
use crate::tables::*;
use crate::{Block, Handle};

#[derive(Debug)]
pub enum DrawingItem<'a> {
    AppId(&'a AppId),
    Block(&'a Block),
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
    pub fn handle(&self) -> Handle {
        match self {
            DrawingItem::AppId(ref app_id) => app_id.handle,
            DrawingItem::Block(ref b) => b.handle,
            DrawingItem::BlockRecord(ref br) => br.handle,
            DrawingItem::DimStyle(ref ds) => ds.handle,
            DrawingItem::Entity(&Entity { ref common, .. }) => common.handle,
            DrawingItem::Layer(ref l) => l.handle,
            DrawingItem::LineType(ref l) => l.handle,
            DrawingItem::Object(&Object { ref common, .. }) => common.handle,
            DrawingItem::Style(ref s) => s.handle,
            DrawingItem::Ucs(ref u) => u.handle,
            DrawingItem::View(ref v) => v.handle,
            DrawingItem::ViewPort(ref v) => v.handle,
        }
    }
}

pub enum DrawingItemMut<'a> {
    AppId(&'a mut AppId),
    Block(&'a mut Block),
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
    pub fn handle(&self) -> Handle {
        match self {
            DrawingItemMut::AppId(ref app_id) => app_id.handle,
            DrawingItemMut::Block(ref b) => b.handle,
            DrawingItemMut::BlockRecord(ref br) => br.handle,
            DrawingItemMut::DimStyle(ref ds) => ds.handle,
            DrawingItemMut::Entity(&mut Entity { ref common, .. }) => common.handle,
            DrawingItemMut::Layer(ref l) => l.handle,
            DrawingItemMut::LineType(ref l) => l.handle,
            DrawingItemMut::Object(&mut Object { ref common, .. }) => common.handle,
            DrawingItemMut::Style(ref s) => s.handle,
            DrawingItemMut::Ucs(ref u) => u.handle,
            DrawingItemMut::View(ref v) => v.handle,
            DrawingItemMut::ViewPort(ref v) => v.handle,
        }
    }
    pub fn set_handle(&mut self, handle: Handle) {
        match self {
            DrawingItemMut::AppId(ref mut app_id) => app_id.handle = handle,
            DrawingItemMut::Block(ref mut b) => b.handle = handle,
            DrawingItemMut::BlockRecord(ref mut br) => br.handle = handle,
            DrawingItemMut::DimStyle(ref mut ds) => ds.handle = handle,
            DrawingItemMut::Entity(&mut Entity { ref mut common, .. }) => common.handle = handle,
            DrawingItemMut::Layer(ref mut l) => l.handle = handle,
            DrawingItemMut::LineType(ref mut l) => l.handle = handle,
            DrawingItemMut::Object(&mut Object { ref mut common, .. }) => common.handle = handle,
            DrawingItemMut::Style(ref mut s) => s.handle = handle,
            DrawingItemMut::Ucs(ref mut u) => u.handle = handle,
            DrawingItemMut::View(ref mut v) => v.handle = handle,
            DrawingItemMut::ViewPort(ref mut v) => v.handle = handle,
        }
    }
    pub fn to_drawing_item(&self) -> DrawingItem {
        match self {
            DrawingItemMut::AppId(ref app_id) => DrawingItem::AppId(app_id),
            DrawingItemMut::Block(ref b) => DrawingItem::Block(b),
            DrawingItemMut::BlockRecord(ref br) => DrawingItem::BlockRecord(br),
            DrawingItemMut::DimStyle(ref ds) => DrawingItem::DimStyle(ds),
            DrawingItemMut::Entity(ref ent) => DrawingItem::Entity(ent),
            DrawingItemMut::Layer(ref l) => DrawingItem::Layer(l),
            DrawingItemMut::LineType(ref l) => DrawingItem::LineType(l),
            DrawingItemMut::Object(ref obj) => DrawingItem::Object(obj),
            DrawingItemMut::Style(ref s) => DrawingItem::Style(s),
            DrawingItemMut::Ucs(ref u) => DrawingItem::Ucs(u),
            DrawingItemMut::View(ref v) => DrawingItem::View(v),
            DrawingItemMut::ViewPort(ref v) => DrawingItem::ViewPort(v),
        }
    }
}
