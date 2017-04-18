// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Write;

extern crate byteorder;
use self::byteorder::{
    LittleEndian,
    WriteBytesExt,
};

use ::{
    Drawing,
    DxfResult,
};

use ::dxb_item_type::DxbItemType;
use ::entities::*;

use itertools::Itertools;

#[doc(hidden)]
pub struct DxbWriter<T: Write> {
    writer: T,
}

impl<T: Write> DxbWriter<T> {
    pub fn new(writer: T) -> Self {
        DxbWriter {
            writer: writer,
        }
    }
    pub fn write(&mut self, drawing: &Drawing) -> DxfResult<()> {
        // write sentinel
        try!(self.write_string("AutoCAD DXB 1.0\r\n"));
        try!(self.writer.write_u8(0x1A));
        try!(self.writer.write_u8(0x00));

        let writing_block = drawing.entities.len() == 0 && drawing.blocks.len() == 1;
        if writing_block {
            // write block header
            try!(self.write_item_type(&DxbItemType::BlockBase));
            let ref block = drawing.blocks[0];
            try!(self.write_n(block.base_point.x));
            try!(self.write_n(block.base_point.y));
        }

        // force all numbers to be floats
        try!(self.write_item_type(&DxbItemType::NumberMode));
        try!(self.write_w(1));

        // write color
        let mut last_color = 0i16;
        try!(self.write_item_type(&DxbItemType::NewColor));
        try!(self.write_w(last_color));

        if writing_block {
            try!(self.write_entities(&drawing.blocks[0].entities));
        }
        else {
            let groups = drawing.entities.iter().group_by(|&e| e.common.layer.clone());
            for (layer, entities) in &groups {
                try!(self.write_item_type(&DxbItemType::NewLayer));
                try!(self.write_null_terminated_string(&*layer));
                for entity in entities {
                    match entity.common.color.get_raw_value() {
                        c if c == last_color => (), // same color, do nothing
                        c => {
                            last_color = c;
                            try!(self.write_item_type(&DxbItemType::NewColor));
                            try!(self.write_w(last_color));
                        },
                    }
                    try!(self.write_entity(&entity));
                }
            }
        }

        // write null terminator
        try!(self.writer.write_u8(0));
        Ok(())
    }
    fn write_entities(&mut self, entities: &Vec<Entity>) -> DxfResult<()> {
        for ref entity in entities {
            try!(self.write_entity(&entity));
        }
        Ok(())
    }
    fn write_entity(&mut self, entity: &Entity) -> DxfResult<()> {
        match &entity.specific {
            &EntityType::Arc(ref arc) => { try!(self.write_arc(&arc)); },
            &EntityType::Circle(ref circle) => { try!(self.write_circle(&circle)); },
            &EntityType::Face3D(ref face) => { try!(self.write_face(&face)); },
            &EntityType::Line(ref line) => { try!(self.write_line(&line)); },
            &EntityType::ModelPoint(ref point) => { try!(self.write_point(&point)); },
            &EntityType::Polyline(ref poly) => { try!(self.write_polyline(&poly)); },
            &EntityType::Seqend(_) => { try!(self.write_seqend()); },
            &EntityType::Solid(ref solid) => { try!(self.write_solid(&solid)); },
            &EntityType::Trace(ref trace) => { try!(self.write_trace(&trace)); },
            &EntityType::Vertex(ref vertex) => { try!(self.write_vertex(&vertex)); },
            _ => (),
        }
        Ok(())
    }
    fn write_arc(&mut self, arc: &Arc) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Arc));
        try!(self.write_n(arc.center.x));
        try!(self.write_n(arc.center.y));
        try!(self.write_n(arc.radius));
        try!(self.write_n(arc.start_angle));
        try!(self.write_n(arc.end_angle));
        Ok(())
    }
    fn write_circle(&mut self, circle: &Circle) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Circle));
        try!(self.write_n(circle.center.x));
        try!(self.write_n(circle.center.y));
        try!(self.write_n(circle.radius));
        Ok(())
    }
    fn write_face(&mut self, face: &Face3D) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Face));
        try!(self.write_n(face.first_corner.x));
        try!(self.write_n(face.first_corner.y));
        try!(self.write_n(face.first_corner.z));
        try!(self.write_n(face.second_corner.x));
        try!(self.write_n(face.second_corner.y));
        try!(self.write_n(face.second_corner.z));
        try!(self.write_n(face.third_corner.x));
        try!(self.write_n(face.third_corner.y));
        try!(self.write_n(face.third_corner.z));
        try!(self.write_n(face.fourth_corner.x));
        try!(self.write_n(face.fourth_corner.y));
        try!(self.write_n(face.fourth_corner.z));
        Ok(())
    }
    fn write_line(&mut self, line: &Line) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Line));
        try!(self.write_n(line.p1.x));
        try!(self.write_n(line.p1.y));
        try!(self.write_n(line.p1.z));
        try!(self.write_n(line.p2.x));
        try!(self.write_n(line.p2.y));
        try!(self.write_n(line.p2.z));
        Ok(())
    }
    fn write_point(&mut self, point: &ModelPoint) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Point));
        try!(self.write_n(point.location.x));
        try!(self.write_n(point.location.y));
        Ok(())
    }
    fn write_polyline(&mut self, poly: &Polyline) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Polyline));
        try!(self.write_w(if poly.get_is_closed() { 1 } else { 0 }));
        for ref vertex in &poly.vertices {
            try!(self.write_vertex(&vertex));
        }
        try!(self.write_seqend());
        Ok(())
    }
    fn write_seqend(&mut self) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Seqend));
        Ok(())
    }
    fn write_solid(&mut self, solid: &Solid) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Solid));
        try!(self.write_n(solid.first_corner.x));
        try!(self.write_n(solid.first_corner.y));
        try!(self.write_n(solid.second_corner.x));
        try!(self.write_n(solid.second_corner.y));
        try!(self.write_n(solid.third_corner.x));
        try!(self.write_n(solid.third_corner.y));
        try!(self.write_n(solid.fourth_corner.x));
        try!(self.write_n(solid.fourth_corner.y));
        Ok(())
    }
    fn write_trace(&mut self, trace: &Trace) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Trace));
        try!(self.write_n(trace.first_corner.x));
        try!(self.write_n(trace.first_corner.y));
        try!(self.write_n(trace.second_corner.x));
        try!(self.write_n(trace.second_corner.y));
        try!(self.write_n(trace.third_corner.x));
        try!(self.write_n(trace.third_corner.y));
        try!(self.write_n(trace.fourth_corner.x));
        try!(self.write_n(trace.fourth_corner.y));
        Ok(())
    }
    fn write_vertex(&mut self, vertex: &Vertex) -> DxfResult<()> {
        try!(self.write_item_type(&DxbItemType::Vertex));
        try!(self.write_n(vertex.location.x));
        try!(self.write_n(vertex.location.y));
        Ok(())
    }
    fn write_string(&mut self, value: &str) -> DxfResult<()> {
        for c in value.chars() {
            try!(self.writer.write_u8(c as u8));
        }
        Ok(())
    }
    fn write_null_terminated_string(&mut self, value: &str) -> DxfResult<()> {
        try!(self.write_string(value));
        try!(self.writer.write_u8(0));
        Ok(())
    }
    fn write_n(&mut self, d: f64) -> DxfResult<()> {
        try!(self.writer.write_f32::<LittleEndian>(d as f32));
        Ok(())
    }
    fn write_w(&mut self, s: i16) -> DxfResult<()> {
        try!(self.writer.write_i16::<LittleEndian>(s));
        Ok(())
    }
    fn write_item_type(&mut self, item_type: &DxbItemType) -> DxfResult<()> {
        try!(self.writer.write_u8(*item_type as u8));
        Ok(())
    }
}
