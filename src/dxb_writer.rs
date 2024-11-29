use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{Drawing, DxfResult};

use crate::dxb_item_type::DxbItemType;
use crate::entities::*;

use itertools::Itertools;

pub(crate) struct DxbWriter<T: Write> {
    writer: T,
}

impl<T: Write> DxbWriter<T> {
    pub fn new(writer: T) -> Self {
        DxbWriter { writer }
    }
    pub fn write(&mut self, drawing: &Drawing) -> DxfResult<()> {
        // write sentinel
        self.write_string("AutoCAD DXB 1.0\r\n")?;
        self.writer.write_u8(0x1A)?;
        self.writer.write_u8(0x00)?;

        let writing_block = drawing.entities().any(|_| true) && drawing.blocks().any(|_| true);
        if writing_block {
            // write block header
            self.write_item_type(DxbItemType::BlockBase)?;
            let block = drawing.blocks().next().unwrap();
            self.write_n(block.base_point.x)?;
            self.write_n(block.base_point.y)?;
        }

        // force all numbers to be floats
        self.write_item_type(DxbItemType::NumberMode)?;
        self.write_w(1)?;

        // write color
        let mut last_color = 0i16;
        self.write_item_type(DxbItemType::NewColor)?;
        self.write_w(last_color)?;

        if writing_block {
            self.write_entities(&drawing.blocks().next().unwrap().entities)?;
        } else {
            let groups = drawing.entities().chunk_by(|&e| e.common.layer.clone());
            for (layer, entities) in &groups {
                self.write_item_type(DxbItemType::NewLayer)?;
                self.write_null_terminated_string(&layer)?;
                for entity in entities {
                    match entity.common.color.raw_value() {
                        c if c == last_color => (), // same color, do nothing
                        c => {
                            last_color = c;
                            self.write_item_type(DxbItemType::NewColor)?;
                            self.write_w(last_color)?;
                        }
                    }
                    self.write_entity(entity)?;
                }
            }
        }

        // write null terminator
        self.writer.write_u8(0)?;
        Ok(())
    }
    fn write_entities(&mut self, entities: &[Entity]) -> DxfResult<()> {
        for entity in entities.iter() {
            self.write_entity(entity)?;
        }
        Ok(())
    }
    fn write_entity(&mut self, entity: &Entity) -> DxfResult<()> {
        match &entity.specific {
            EntityType::Arc(ref arc) => {
                self.write_arc(arc)?;
            }
            EntityType::Circle(ref circle) => {
                self.write_circle(circle)?;
            }
            EntityType::Face3D(ref face) => {
                self.write_face(face)?;
            }
            EntityType::Line(ref line) => {
                self.write_line(line)?;
            }
            EntityType::ModelPoint(ref point) => {
                self.write_point(point)?;
            }
            EntityType::Polyline(ref poly) => {
                self.write_polyline(poly)?;
            }
            EntityType::Seqend(_) => {
                self.write_seqend()?;
            }
            EntityType::Solid(ref solid) => {
                self.write_solid(solid)?;
            }
            EntityType::Trace(ref trace) => {
                self.write_trace(trace)?;
            }
            EntityType::Vertex(ref vertex) => {
                self.write_vertex(vertex)?;
            }
            _ => (),
        }
        Ok(())
    }
    fn write_arc(&mut self, arc: &Arc) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Arc)?;
        self.write_n(arc.center.x)?;
        self.write_n(arc.center.y)?;
        self.write_n(arc.radius)?;
        self.write_n(arc.start_angle)?;
        self.write_n(arc.end_angle)?;
        Ok(())
    }
    fn write_circle(&mut self, circle: &Circle) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Circle)?;
        self.write_n(circle.center.x)?;
        self.write_n(circle.center.y)?;
        self.write_n(circle.radius)?;
        Ok(())
    }
    fn write_face(&mut self, face: &Face3D) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Face)?;
        self.write_n(face.first_corner.x)?;
        self.write_n(face.first_corner.y)?;
        self.write_n(face.first_corner.z)?;
        self.write_n(face.second_corner.x)?;
        self.write_n(face.second_corner.y)?;
        self.write_n(face.second_corner.z)?;
        self.write_n(face.third_corner.x)?;
        self.write_n(face.third_corner.y)?;
        self.write_n(face.third_corner.z)?;
        self.write_n(face.fourth_corner.x)?;
        self.write_n(face.fourth_corner.y)?;
        self.write_n(face.fourth_corner.z)?;
        Ok(())
    }
    fn write_line(&mut self, line: &Line) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Line)?;
        self.write_n(line.p1.x)?;
        self.write_n(line.p1.y)?;
        self.write_n(line.p1.z)?;
        self.write_n(line.p2.x)?;
        self.write_n(line.p2.y)?;
        self.write_n(line.p2.z)?;
        Ok(())
    }
    fn write_point(&mut self, point: &ModelPoint) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Point)?;
        self.write_n(point.location.x)?;
        self.write_n(point.location.y)?;
        Ok(())
    }
    fn write_polyline(&mut self, poly: &Polyline) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Polyline)?;
        self.write_w(if poly.is_closed() { 1 } else { 0 })?;
        for vertex in poly.vertices() {
            self.write_vertex(vertex)?;
        }
        self.write_seqend()?;
        Ok(())
    }
    fn write_seqend(&mut self) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Seqend)?;
        Ok(())
    }
    fn write_solid(&mut self, solid: &Solid) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Solid)?;
        self.write_n(solid.first_corner.x)?;
        self.write_n(solid.first_corner.y)?;
        self.write_n(solid.second_corner.x)?;
        self.write_n(solid.second_corner.y)?;
        self.write_n(solid.third_corner.x)?;
        self.write_n(solid.third_corner.y)?;
        self.write_n(solid.fourth_corner.x)?;
        self.write_n(solid.fourth_corner.y)?;
        Ok(())
    }
    fn write_trace(&mut self, trace: &Trace) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Trace)?;
        self.write_n(trace.first_corner.x)?;
        self.write_n(trace.first_corner.y)?;
        self.write_n(trace.second_corner.x)?;
        self.write_n(trace.second_corner.y)?;
        self.write_n(trace.third_corner.x)?;
        self.write_n(trace.third_corner.y)?;
        self.write_n(trace.fourth_corner.x)?;
        self.write_n(trace.fourth_corner.y)?;
        Ok(())
    }
    fn write_vertex(&mut self, vertex: &Vertex) -> DxfResult<()> {
        self.write_item_type(DxbItemType::Vertex)?;
        self.write_n(vertex.location.x)?;
        self.write_n(vertex.location.y)?;
        Ok(())
    }
    fn write_string(&mut self, value: &str) -> DxfResult<()> {
        for c in value.chars() {
            self.writer.write_u8(c as u8)?;
        }
        Ok(())
    }
    fn write_null_terminated_string(&mut self, value: &str) -> DxfResult<()> {
        self.write_string(value)?;
        self.writer.write_u8(0)?;
        Ok(())
    }
    fn write_n(&mut self, d: f64) -> DxfResult<()> {
        self.writer.write_f32::<LittleEndian>(d as f32)?;
        Ok(())
    }
    fn write_w(&mut self, s: i16) -> DxfResult<()> {
        self.writer.write_i16::<LittleEndian>(s)?;
        Ok(())
    }
    fn write_item_type(&mut self, item_type: DxbItemType) -> DxfResult<()> {
        self.writer.write_u8(item_type as u8)?;
        Ok(())
    }
}
