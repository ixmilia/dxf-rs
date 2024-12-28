use std::io::Read;

use crate::{Block, Color, Drawing, DxfError, DxfResult, Point};

use crate::dxb_item_type::DxbItemType;
use crate::entities::*;
use crate::entity_iter::collect_entities;
use crate::helper_functions::*;

use enum_primitive::FromPrimitive;

pub(crate) struct DxbReader<T: Read> {
    reader: T,
    is_integer_mode: bool,
    layer_name: String,
    scale_factor: f64,
    current_color: Color,
    last_line_point: Point,
    last_trace_p3: Point,
    last_trace_p4: Point,
    offset: usize,
}

impl<T: Read> DxbReader<T> {
    pub fn new(reader: T) -> Self {
        DxbReader {
            reader,
            is_integer_mode: true,
            layer_name: String::from("0"),
            scale_factor: 1.0,
            current_color: Color::by_layer(),
            last_line_point: Point::origin(),
            last_trace_p3: Point::origin(),
            last_trace_p4: Point::origin(),
            offset: 0,
        }
    }
    pub fn load(&mut self) -> DxfResult<Drawing> {
        // swallow the next two bytes
        assert_or_err!(
            try_option_io_result_into_err!(read_u8(&mut self.reader)),
            0x1A,
            16
        );
        self.advance_offset(1);
        assert_or_err!(
            try_option_io_result_into_err!(read_u8(&mut self.reader)),
            0x00,
            17
        );
        self.advance_offset(1);

        let mut block_base = None;
        let mut entities = vec![];
        loop {
            let item_type = match DxbItemType::from_u8(try_option_io_result_into_err!(read_u8(
                &mut self.reader
            ))) {
                Some(item_type) => item_type,
                None => return Err(DxfError::UnexpectedEnumValue(self.offset)),
            };
            self.advance_offset(1);
            match item_type {
                // entities
                DxbItemType::Arc => {
                    entities.push(self.read_arc()?);
                }
                DxbItemType::Circle => {
                    entities.push(self.read_circle()?);
                }
                DxbItemType::Face => {
                    entities.push(self.read_face()?);
                }
                DxbItemType::Line | DxbItemType::Line3D => {
                    entities.push(self.read_line()?);
                }
                DxbItemType::LineExtension => {
                    entities.push(self.read_line_extension()?);
                }
                DxbItemType::LineExtension3D => {
                    entities.push(self.read_line_extension_3d()?);
                }
                DxbItemType::Point => {
                    entities.push(self.read_point()?);
                }
                DxbItemType::Polyline => {
                    entities.push(self.read_polyline()?);
                }
                DxbItemType::Seqend => {
                    entities.push(self.read_seqend()?);
                }
                DxbItemType::Solid => {
                    entities.push(self.read_solid()?);
                }
                DxbItemType::Trace => {
                    entities.push(self.read_trace()?);
                }
                DxbItemType::TraceExtension => {
                    entities.push(self.read_trace_extension()?);
                }
                DxbItemType::Vertex => {
                    entities.push(self.read_vertex()?);
                }
                // global values
                DxbItemType::NewColor => {
                    self.current_color = Color::from_raw_value(self.read_w()? as i16);
                }
                DxbItemType::NewLayer => {
                    self.layer_name = self.read_null_terminated_string()?;
                }
                DxbItemType::ScaleFactor => {
                    self.scale_factor = self.read_f()?;
                }
                // other
                DxbItemType::BlockBase => {
                    let loc = Point::new(self.read_n()?, self.read_n()?, 0.0);
                    if block_base.is_none() && entities.is_empty() {
                        // only if this is the first item encountered
                        block_base = Some(loc);
                    } else {
                        return Err(DxfError::InvalidBinaryFile);
                    }
                }
                DxbItemType::Bulge => {
                    let bulge = self.read_u()?;
                    match vec_last!(entities) {
                        Entity {
                            specific: EntityType::Vertex(ref mut v),
                            ..
                        } => {
                            v.bulge = bulge;
                        }
                        _ => return Err(DxfError::UnexpectedEnumValue(self.offset)),
                    }
                }
                DxbItemType::NumberMode => {
                    self.is_integer_mode = self.read_w()? == 0;
                }
                DxbItemType::Width => {
                    let starting_width = self.read_n()?;
                    let ending_width = self.read_n()?;
                    match vec_last!(entities) {
                        Entity {
                            specific: EntityType::Vertex(ref mut v),
                            ..
                        } => {
                            v.starting_width = starting_width;
                            v.ending_width = ending_width;
                        }
                        _ => return Err(DxfError::UnexpectedEnumValue(self.offset)),
                    }
                }
                // done
                DxbItemType::EOF => break,
            }
        }

        let mut gathered_entities = vec![];
        collect_entities(&mut entities.into_iter(), &mut gathered_entities)?;
        let mut drawing = Drawing::new();
        drawing.clear();
        match block_base {
            Some(location) => {
                let mut block = Block {
                    base_point: location,
                    ..Default::default()
                };
                block.entities = gathered_entities;
                drawing.add_block(block);
            }
            None => {
                for e in gathered_entities {
                    drawing.add_entity(e);
                }
            }
        }

        Ok(drawing)
    }
    fn read_arc(&mut self) -> DxfResult<Entity> {
        let center = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let radius = self.read_n()?;
        let start = self.read_a()?;
        let end = self.read_a()?;
        let arc = Arc::new(center, radius, start, end);
        Ok(self.wrap_common_values(EntityType::Arc(arc)))
    }
    fn read_circle(&mut self) -> DxfResult<Entity> {
        let center = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let radius = self.read_n()?;
        let circle = Circle::new(center, radius);
        Ok(self.wrap_common_values(EntityType::Circle(circle)))
    }
    fn read_face(&mut self) -> DxfResult<Entity> {
        let p1 = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        let p2 = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        let p3 = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        let p4 = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        let face = Face3D::new(p1, p2, p3, p4);
        Ok(self.wrap_common_values(EntityType::Face3D(face)))
    }
    fn read_line(&mut self) -> DxfResult<Entity> {
        let from = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        let to = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        self.last_line_point = to.clone();
        let line = Line::new(from, to);
        Ok(self.wrap_common_values(EntityType::Line(line)))
    }
    fn read_line_extension(&mut self) -> DxfResult<Entity> {
        let to = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let line = Line::new(self.last_line_point.clone(), to);
        self.last_line_point = line.p2.clone();
        Ok(self.wrap_common_values(EntityType::Line(line)))
    }
    fn read_line_extension_3d(&mut self) -> DxfResult<Entity> {
        let to = Point::new(self.read_n()?, self.read_n()?, self.read_n()?);
        let line = Line::new(self.last_line_point.clone(), to);
        self.last_line_point = line.p2.clone();
        Ok(self.wrap_common_values(EntityType::Line(line)))
    }
    fn read_point(&mut self) -> DxfResult<Entity> {
        let point = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let point = ModelPoint::new(point);
        Ok(self.wrap_common_values(EntityType::ModelPoint(point)))
    }
    fn read_polyline(&mut self) -> DxfResult<Entity> {
        let is_closed = self.read_w()? != 0;
        let mut poly = Polyline::default();
        poly.set_is_closed(is_closed);
        Ok(self.wrap_common_values(EntityType::Polyline(poly)))
    }
    fn read_seqend(&mut self) -> DxfResult<Entity> {
        Ok(self.wrap_common_values(EntityType::Seqend(Seqend::default())))
    }
    fn read_solid(&mut self) -> DxfResult<Entity> {
        let p1 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p2 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p3 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p4 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let solid = Solid::new(p1, p2, p3, p4);
        Ok(self.wrap_common_values(EntityType::Solid(solid)))
    }
    fn read_trace(&mut self) -> DxfResult<Entity> {
        let p1 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p2 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p3 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p4 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let trace = Trace::new(p1, p2, p3, p4);
        self.last_trace_p3 = trace.third_corner.clone();
        self.last_trace_p4 = trace.fourth_corner.clone();
        Ok(self.wrap_common_values(EntityType::Trace(trace)))
    }
    fn read_trace_extension(&mut self) -> DxfResult<Entity> {
        let p3 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let p4 = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let trace = Trace::new(
            self.last_trace_p3.clone(),
            self.last_trace_p4.clone(),
            p3,
            p4,
        );
        self.last_trace_p3 = trace.third_corner.clone();
        self.last_trace_p4 = trace.fourth_corner.clone();
        Ok(self.wrap_common_values(EntityType::Trace(trace)))
    }
    fn read_vertex(&mut self) -> DxfResult<Entity> {
        let location = Point::new(self.read_n()?, self.read_n()?, 0.0);
        let vertex = Vertex::new(location);
        Ok(self.wrap_common_values(EntityType::Vertex(vertex)))
    }
    fn wrap_common_values(&self, specific: EntityType) -> Entity {
        let mut entity = Entity::new(specific);
        entity.common.color = self.current_color.clone();
        entity.common.layer.clone_from(&self.layer_name);
        entity
    }
    fn read_null_terminated_string(&mut self) -> DxfResult<String> {
        let mut value = String::new();
        loop {
            let b = try_option_io_result_into_err!(read_u8(&mut self.reader));
            self.advance_offset(1);
            if b == 0 {
                return Ok(value);
            } else {
                value.push(b as char);
            }
        }
    }
    fn read_a(&mut self) -> DxfResult<f64> {
        let value = if self.is_integer_mode {
            f64::from(read_i32(&mut self.reader)?) * self.scale_factor / 1_000_000.0
        } else {
            f64::from(read_f32(&mut self.reader)?)
        };
        self.advance_offset(4);
        Ok(value)
    }
    fn read_f(&mut self) -> DxfResult<f64> {
        let value = read_f64(&mut self.reader);
        self.advance_offset(8);
        Ok(value.or::<f64>(Ok(0.0)).unwrap())
    }
    fn read_n(&mut self) -> DxfResult<f64> {
        if self.is_integer_mode {
            let value = f64::from(read_i16(&mut self.reader)?) * self.scale_factor;
            self.advance_offset(2);
            Ok(value)
        } else {
            let value = f64::from(read_f32(&mut self.reader)?);
            self.advance_offset(4);
            Ok(value)
        }
    }
    fn read_u(&mut self) -> DxfResult<f64> {
        let value = if self.is_integer_mode {
            f64::from(read_i32(&mut self.reader)?) * 65536.0 * self.scale_factor
        } else {
            f64::from(read_f32(&mut self.reader)?)
        };
        self.advance_offset(4);
        Ok(value)
    }
    fn read_w(&mut self) -> DxfResult<i32> {
        let value = (f64::from(read_i16(&mut self.reader)?) * self.scale_factor) as i32;
        self.advance_offset(2);
        Ok(value)
    }
    fn advance_offset(&mut self, offset: usize) {
        self.offset += offset;
    }
}
