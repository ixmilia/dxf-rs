// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

// other implementation is in `generated/objects.rs`

use std::io::Write;
use std::ops::Add;
use enum_primitive::FromPrimitive;
use itertools::{
    Itertools,
    PutBack,
};

extern crate chrono;
use self::chrono::Duration;

use ::{
    CodePair,
    Color,
    DataTableValue,
    DxfError,
    DxfResult,
    Point,
    SectionTypeSettings,
    TableCellStyle,
    TransformationMatrix,
};

use code_pair_writer::CodePairWriter;
use enums::*;
use objects::*;
use helper_functions::*;


//------------------------------------------------------------------------------
//                                                                  GeoMeshPoint
//------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct GeoMeshPoint {
    pub source: Point,
    pub destination: Point,
}

impl GeoMeshPoint {
    pub fn new(source: Point, destination: Point) -> Self {
        GeoMeshPoint {
            source: source,
            destination: destination,
        }
    }
}

//------------------------------------------------------------------------------
//                                                             MLineStyleElement
//------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct MLineStyleElement {
    pub offset: f64,
    pub color: Color,
    pub line_type: String,
}

impl MLineStyleElement {
    pub fn new(offset: f64, color: Color, line_type: String) -> Self {
        MLineStyleElement {
            offset: offset,
            color: color,
            line_type: line_type,
        }
    }
}

//------------------------------------------------------------------------------
//                                                                     DataTable
//------------------------------------------------------------------------------
impl DataTable {
    #[doc(hidden)]
    pub fn set_value(&mut self, row: usize, col: usize, val: DataTableValue) {
        if row <= self.row_count && col <= self.column_count {
            self.values[row][col] = Some(val);
        }
    }
}

//------------------------------------------------------------------------------
//                                                                    VbaProject
//------------------------------------------------------------------------------
impl VbaProject {
    #[doc(hidden)]
    pub fn get_hex_strings(&self) -> DxfResult<Vec<String>> {
        let mut result = vec![];
        for s in self.data.chunks(128) {
            let mut line = String::new();
            for b in s {
                line.push_str(&format!("{:X}", b));
            }
            result.push(line);
        }

        Ok(result)
    }
}

//------------------------------------------------------------------------------
//                                                                  ObjectCommon
//------------------------------------------------------------------------------
impl ObjectCommon {
    /// Ensures all values are valid.
    pub fn normalize(&mut self) {
        // nothing to do, but this method should still exist.
    }
}

//------------------------------------------------------------------------------
//                                                                        Object
//------------------------------------------------------------------------------
impl Object {
    /// Creates a new `Object` with the default common values.
    pub fn new(specific: ObjectType) -> Self {
        Object {
            common: Default::default(),
            specific: specific,
        }
    }
    /// Ensures all object values are valid.
    pub fn normalize(&mut self) {
        self.common.normalize();
        // no object-specific values to set
    }
    #[doc(hidden)]
    pub fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Option<Object>>
        where I: Iterator<Item = DxfResult<CodePair>> {

        loop {
            match iter.next() {
                // first code pair must be 0/object-type
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    let type_string = try!(pair.value.assert_string());
                    if type_string == "ENDSEC" || type_string == "ENDBLK" {
                        iter.put_back(Ok(pair));
                        return Ok(None);
                    }

                    match ObjectType::from_type_string(&type_string) {
                        Some(e) => {
                            let mut obj = Object::new(e);
                            if !try!(obj.apply_custom_reader(iter)) {
                                // no custom reader, use the auto-generated one
                                loop {
                                    match iter.next() {
                                        Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                            // new object or ENDSEC
                                            iter.put_back(Ok(pair));
                                            break;
                                        },
                                        Some(Ok(pair)) => try!(obj.apply_code_pair(&pair, iter)),
                                        Some(Err(e)) => return Err(e),
                                        None => return Err(DxfError::UnexpectedEndOfInput),
                                    }
                                }

                                try!(obj.post_parse());
                            }

                            return Ok(Some(obj));
                        },
                        None => {
                            // swallow unsupported object
                            loop {
                               match iter.next() {
                                    Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                        // found another object or ENDSEC
                                        iter.put_back(Ok(pair));
                                        break;
                                    },
                                    Some(Ok(_)) => (), // part of the unsupported object
                                    Some(Err(e)) => return Err(e),
                                    None => return Err(DxfError::UnexpectedEndOfInput),
                                }
                            }
                        }
                    }
                },
                Some(Ok(pair)) => return Err(DxfError::UnexpectedCodePair(pair, String::from("expected 0/object-type or 0/ENDSEC"))),
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }
    }
    fn apply_code_pair<I>(&mut self, pair: &CodePair, iter: &mut PutBack<I>) -> DxfResult<()>
        where I: Iterator<Item = DxfResult<CodePair>> {

        if !try!(self.specific.try_apply_code_pair(&pair)) {
            try!(self.common.apply_individual_pair(&pair, iter));
        }
        Ok(())
    }
    fn post_parse(&mut self) -> DxfResult<()> {
        match self.specific {
            ObjectType::AcadProxyObject(ref mut proxy) => {
                for item in &proxy._object_ids_a {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy._object_ids_b {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy._object_ids_c {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy._object_ids_d {
                    proxy.object_ids.push(item.clone());
                }
                proxy._object_ids_a.clear();
                proxy._object_ids_b.clear();
                proxy._object_ids_c.clear();
                proxy._object_ids_d.clear();
            },
            ObjectType::GeoData(ref mut geo) => {
                let mut source_points = vec![];
                let mut destination_points = vec![];
                combine_points_2(&mut geo._source_mesh_x_points, &mut geo._source_mesh_y_points, &mut source_points, Point::new);
                combine_points_2(&mut geo._destination_mesh_x_points, &mut geo._destination_mesh_y_points, &mut destination_points, Point::new);
                for (s, d) in source_points.drain(..).zip(destination_points.drain(..)) {
                    geo.geo_mesh_points.push(GeoMeshPoint::new(s, d));
                }

                combine_points_3(&mut geo._face_point_index_x, &mut geo._face_point_index_y, &mut geo._face_point_index_z, &mut geo.face_indices, Point::new);
            },
            ObjectType::Material(ref mut material) => {
                material.diffuse_map_transformation_matrix.from_vec(&material._diffuse_map_transformation_matrix_values);
                material.specular_map_transformation_matrix.from_vec(&material._specular_map_transformation_matrix_values);
                material.reflection_map_transformation_matrix.from_vec(&material._reflection_map_transformation_matrix_values);
                material.opacity_map_transformation_matrix.from_vec(&material._opacity_map_transformation_matrix_values);
                material.bump_map_transformation_matrix.from_vec(&material._bump_map_transformation_matrix_values);
                material.refraction_map_transformation_matrix.from_vec(&material._refraction_map_transformation_matrix_values);
                material.normal_map_transformation_matrix.from_vec(&material._normal_map_transformation_matrix_values);
                material._diffuse_map_transformation_matrix_values.clear();
                material._specular_map_transformation_matrix_values.clear();
                material._reflection_map_transformation_matrix_values.clear();
                material._opacity_map_transformation_matrix_values.clear();
                material._bump_map_transformation_matrix_values.clear();
                material._refraction_map_transformation_matrix_values.clear();
                material._normal_map_transformation_matrix_values.clear();
            },
            ObjectType::MLineStyle(ref mut mline) => {
                for (o, (c, l)) in mline._element_offsets.drain(..).zip(mline._element_colors.drain(..).zip(mline._element_line_types.drain(..))) {
                    mline.elements.push(MLineStyleElement::new(o, c, l));
                }
            },
            ObjectType::VbaProject(ref mut vba) => {
                // each char in each _hex_data should be added to `data` byte array
                let mut result = vec![];
                for s in &vba._hex_data {
                    try!(parse_hex_string(s, &mut result));
                }

                vba.data = result;
                vba._hex_data.clear();
            },
            _ => (),
        }

        Ok(())
    }
    fn apply_custom_reader<I>(&mut self, iter: &mut PutBack<I>) -> DxfResult<bool>
        where I: Iterator<Item = DxfResult<CodePair>> {

        match self.specific {
            ObjectType::DataTable(ref mut data) => {
                let mut read_column_count = false;
                let mut read_row_count = false;
                let mut _current_column_code = 0;
                let mut current_column = 0;
                let mut current_row = 0;
                let mut created_table = false;
                let mut current_2d_point = Point::origin();
                let mut current_3d_point = Point::origin();

                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { data.name = try!(pair.value.assert_string()); },
                        70 => { data.field = try!(pair.value.assert_i16()); },
                        90 => {
                            data.column_count = try!(pair.value.assert_i32()) as usize;
                            read_column_count = true;
                        },
                        91 => {
                            data.row_count = try!(pair.value.assert_i32()) as usize;
                            read_row_count = true;
                        },

                        // column headers
                        2 => { data.column_names.push(try!(pair.value.assert_string())); },
                        92 => {
                            _current_column_code = try!(pair.value.assert_i32());
                            current_column += 1;
                            current_row = 0;
                        },

                        // column values
                        3 => { data.set_value(current_row, current_column, DataTableValue::Str(try!(pair.value.assert_string()))); },
                        40 => { data.set_value(current_row, current_column, DataTableValue::Double(try!(pair.value.assert_f64()))); },
                        71 => { data.set_value(current_row, current_column, DataTableValue::Boolean(as_bool(try!(pair.value.assert_i16())))); },
                        93 => { data.set_value(current_row, current_column, DataTableValue::Integer(try!(pair.value.assert_i32()))); },
                        10 => { current_2d_point.x = try!(pair.value.assert_f64()); },
                        20 => { current_2d_point.y = try!(pair.value.assert_f64()); },
                        30 => {
                            current_2d_point.z = try!(pair.value.assert_f64());
                            data.set_value(current_row, current_column, DataTableValue::Point2D(current_2d_point.clone()));
                            current_2d_point = Point::origin();
                        },
                        11 => { current_3d_point.x = try!(pair.value.assert_f64()); },
                        21 => { current_3d_point.y = try!(pair.value.assert_f64()); },
                        31 => {
                            current_3d_point.z = try!(pair.value.assert_f64());
                            data.set_value(current_row, current_column, DataTableValue::Point3D(current_3d_point.clone()));
                            current_3d_point = Point::origin();
                        },
                        330 | 331 | 340 | 350 | 360 => {
                            if read_row_count || read_column_count {
                                data.set_value(current_row, current_column, DataTableValue::Handle(try!(as_u32(try!(pair.value.assert_string())))));
                            }
                            else {
                                try!(self.common.apply_individual_pair(&pair, iter));
                            }
                        }

                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }

                    if read_row_count && read_column_count && !created_table {
                        for row in 0..data.row_count {
                            data.values.push(vec![]);
                            for _ in 0..data.column_count {
                                data.values[row].push(None);
                            }
                        }
                        created_table = true;
                    }
                }
            },
            ObjectType::Dictionary(ref mut dict) => {
                let mut last_entry_name = String::new();
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        3 => { last_entry_name = try!(pair.value.assert_string()); },
                        280 => { dict.is_hard_owner = as_bool(try!(pair.value.assert_i16())); },
                        281 => { dict.duplicate_record_handling = try_result!(DictionaryDuplicateRecordHandling::from_i16(try!(pair.value.assert_i16()))); },
                        350 | 360 => {
                            let handle = try!(as_u32(try!(pair.value.assert_string())));
                            dict.value_handles.insert(last_entry_name.clone(), handle);
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::DictionaryWithDefault(ref mut dict) => {
                let mut last_entry_name = String::new();
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        3 => { last_entry_name = try!(pair.value.assert_string()); },
                        281 => { dict.duplicate_record_handling = try_result!(DictionaryDuplicateRecordHandling::from_i16(try!(pair.value.assert_i16()))); },
                        340 => { dict.default_handle = try!(as_u32(try!(pair.value.assert_string()))); },
                        350 | 360 => {
                            let handle = try!(as_u32(try!(pair.value.assert_string())));
                            dict.value_handles.insert(last_entry_name.clone(), handle);
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::Layout(ref mut layout) => {
                let mut is_reading_plot_settings = true;
                loop {
                    let pair = next_pair!(iter);
                    if is_reading_plot_settings {
                        if pair.code == 100 && try!(pair.value.assert_string()) == "AcDbLayout" {
                            is_reading_plot_settings = false;
                        }
                        else {
                            try!(self.common.apply_individual_pair(&pair, iter));
                        }
                    }
                    else {
                        match pair.code {
                            1 => { layout.layout_name = try!(pair.value.assert_string()); },
                            10 => { layout.minimum_limits.x = try!(pair.value.assert_f64()); },
                            20 => { layout.minimum_limits.y = try!(pair.value.assert_f64()); },
                            11 => { layout.maximum_limits.x = try!(pair.value.assert_f64()); },
                            21 => { layout.maximum_limits.y = try!(pair.value.assert_f64()); },
                            12 => { layout.insertion_base_point.x = try!(pair.value.assert_f64()); },
                            22 => { layout.insertion_base_point.y = try!(pair.value.assert_f64()); },
                            32 => { layout.insertion_base_point.z = try!(pair.value.assert_f64()); },
                            13 => { layout.ucs_origin.x = try!(pair.value.assert_f64()); },
                            23 => { layout.ucs_origin.y = try!(pair.value.assert_f64()); },
                            33 => { layout.ucs_origin.z = try!(pair.value.assert_f64()); },
                            14 => { layout.minimum_extents.x = try!(pair.value.assert_f64()); },
                            24 => { layout.minimum_extents.y = try!(pair.value.assert_f64()); },
                            34 => { layout.minimum_extents.z = try!(pair.value.assert_f64()); },
                            15 => { layout.maximum_extents.x = try!(pair.value.assert_f64()); },
                            25 => { layout.maximum_extents.y = try!(pair.value.assert_f64()); },
                            35 => { layout.maximum_extents.z = try!(pair.value.assert_f64()); },
                            16 => { layout.ucs_x_axis.x = try!(pair.value.assert_f64()); },
                            26 => { layout.ucs_x_axis.y = try!(pair.value.assert_f64()); },
                            36 => { layout.ucs_x_axis.z = try!(pair.value.assert_f64()); },
                            17 => { layout.ucs_y_axis.x = try!(pair.value.assert_f64()); },
                            27 => { layout.ucs_y_axis.y = try!(pair.value.assert_f64()); },
                            37 => { layout.ucs_y_axis.z = try!(pair.value.assert_f64()); },
                            70 => { layout.layout_flags = try!(pair.value.assert_i16()) as i32; },
                            71 => { layout.tab_order = try!(pair.value.assert_i16()) as i32; },
                            76 => { layout.ucs_orthographic_type = try_result!(UcsOrthographicType::from_i16(try!(pair.value.assert_i16()))); },
                            146 => { layout.elevation = try!(pair.value.assert_f64()); },
                            330 => { layout.viewport = try!(as_u32(try!(pair.value.assert_string()))); },
                            345 => { layout.table_record = try!(as_u32(try!(pair.value.assert_string()))); },
                            346 => { layout.table_record_base = try!(as_u32(try!(pair.value.assert_string()))); },
                            _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                        }
                    }
                }
            },
            ObjectType::LightList(ref mut ll) => {
                let mut read_version_number = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { }, // don't worry about the light's name; it'll be read from the light entity directly
                        5 => {
                            if read_version_number {
                                // pointer to a new light
                                ll.lights.push(try!(as_u32(try!(pair.value.assert_string()))));
                            }
                            else {
                                // might still be the handle
                                try!(self.common.apply_individual_pair(&pair, iter));;
                            }
                        },
                        90 => {
                            if read_version_number {
                                // count of lights is ignored since it's implicitly set by reading the values
                            }
                            else {
                                ll.version = try!(pair.value.assert_i32());
                                read_version_number = false;
                            }
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::Material(ref mut mat) => {
                let mut read_diffuse_map_file_name = false;
                let mut is_reading_normal = false;
                let mut read_diffuse_map_blend_factor = false;
                let mut read_image_file_diffuse_map = false;
                let mut read_diffuse_map_projection_method = false;
                let mut read_diffuse_map_tiling_method = false;
                let mut read_diffuse_map_auto_transform_method = false;
                let mut read_ambient_color_value = false;
                let mut read_bump_map_projection_method = false;
                let mut read_luminance_mode = false;
                let mut read_bump_map_tiling_method = false;
                let mut read_normal_map_method = false;
                let mut read_bump_map_auto_transform_method = false;
                let mut read_use_image_file_for_refraction_map = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { mat.name = try!(pair.value.assert_string()); },
                        2 => { mat.description = try!(pair.value.assert_string()); },
                        3 => {
                            if !read_diffuse_map_file_name {
                                mat.diffuse_map_file_name = try!(pair.value.assert_string());
                                read_diffuse_map_file_name = true;
                            }
                            else {
                                mat.normal_map_file_name = try!(pair.value.assert_string());
                                is_reading_normal = true;
                            }
                        },
                        4 => { mat.normal_map_file_name = try!(pair.value.assert_string()); },
                        6 => { mat.reflection_map_file_name = try!(pair.value.assert_string()); },
                        7 => { mat.opacity_map_file_name = try!(pair.value.assert_string()); },
                        8 => { mat.bump_map_file_name = try!(pair.value.assert_string()); },
                        9 => { mat.refraction_map_file_name = try!(pair.value.assert_string()); },
                        40 => { mat.ambient_color_factor = try!(pair.value.assert_f64()); },
                        41 => { mat.diffuse_color_factor = try!(pair.value.assert_f64()); },
                        42 => {
                            if !read_diffuse_map_blend_factor {
                                mat.diffuse_map_blend_factor = try!(pair.value.assert_f64());
                                read_diffuse_map_blend_factor = true;
                            }
                            else {
                                mat.normal_map_blend_factor = try!(pair.value.assert_f64());
                                is_reading_normal = true;
                            }
                        },
                        43 => {
                            if is_reading_normal {
                                mat._normal_map_transformation_matrix_values.push(try!(pair.value.assert_f64()));
                            }
                            else {
                                mat._diffuse_map_transformation_matrix_values.push(try!(pair.value.assert_f64()));
                            }
                        },
                        44 => { mat.specular_gloss_factor = try!(pair.value.assert_f64()); },
                        45 => { mat.specular_color_factor = try!(pair.value.assert_f64()); },
                        46 => { mat.specular_map_blend_factor = try!(pair.value.assert_f64()); },
                        47 => { mat._specular_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        48 => { mat.reflection_map_blend_factor = try!(pair.value.assert_f64()); },
                        49 => { mat._reflection_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        62 => { mat.gen_proc_color_index_value = Color::from_raw_value(try!(pair.value.assert_i16())); },
                        70 => { mat.override_ambient_color = as_bool(try!(pair.value.assert_i16())); },
                        71 => { mat.override_diffuse_color = as_bool(try!(pair.value.assert_i16())); },
                        72 => {
                            if !read_image_file_diffuse_map {
                                mat.use_image_file_for_diffuse_map = as_bool(try!(pair.value.assert_i16()));
                                read_image_file_diffuse_map = true;
                            }
                            else {
                                mat.use_image_file_for_normal_map = as_bool(try!(pair.value.assert_i16()));
                            }
                        },
                        73 => {
                            if !read_diffuse_map_projection_method {
                                mat.diffuse_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16())));
                                read_diffuse_map_projection_method = true;
                            }
                            else {
                                mat.normal_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16())));
                                is_reading_normal = true;
                            }
                        },
                        74 => {
                            if !read_diffuse_map_tiling_method {
                                mat.diffuse_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16())));
                                read_diffuse_map_tiling_method = true;
                            }
                            else {
                                mat.normal_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16())));
                                is_reading_normal = true;
                            }
                        },
                        75 => {
                            if !read_diffuse_map_auto_transform_method {
                                mat.diffuse_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16())));
                                read_diffuse_map_auto_transform_method = true;
                            }
                            else {
                                mat.normal_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16())));
                                is_reading_normal = true;
                            }
                        },
                        76 => { mat.override_specular_color = as_bool(try!(pair.value.assert_i16())); },
                        77 => { mat.use_image_file_for_specular_map = as_bool(try!(pair.value.assert_i16())); },
                        78 => { mat.specular_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        79 => { mat.specular_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        90 => {
                            if !read_ambient_color_value {
                                mat.ambient_color_value = try!(pair.value.assert_i32());
                                read_ambient_color_value = true;
                            }
                            else {
                                mat.self_illumination = try!(pair.value.assert_i32());
                            }
                        },
                        91 => { mat.diffuse_color_value = try!(pair.value.assert_i32()); },
                        92 => { mat.specular_color_value = try!(pair.value.assert_i32()); },
                        93 => { mat.illumination_model = try!(pair.value.assert_i32()); },
                        94 => { mat.channel_flags = try!(pair.value.assert_i32()); },
                        140 => { mat.opacity_factor = try!(pair.value.assert_f64()); },
                        141 => { mat.opacity_map_blend_factor = try!(pair.value.assert_f64()); },
                        142 => { mat._opacity_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        143 => { mat.bump_map_blend_factor = try!(pair.value.assert_f64()); },
                        144 => { mat._bump_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        145 => { mat.refraction_index = try!(pair.value.assert_f64()); },
                        146 => { mat.refraction_map_blend_factor = try!(pair.value.assert_f64()); },
                        147 => { mat._refraction_map_transformation_matrix_values.push(try!(pair.value.assert_f64())); },
                        148 => { mat.translucence = try!(pair.value.assert_f64()); },
                        170 => { mat.specular_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        171 => { mat.use_image_file_for_reflection_map = as_bool(try!(pair.value.assert_i16())); },
                        172 => { mat.reflection_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        173 => { mat.reflection_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        174 => { mat.reflection_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        175 => { mat.use_image_file_for_opacity_map = as_bool(try!(pair.value.assert_i16())); },
                        176 => { mat.opacity_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        177 => { mat.opacity_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        178 => { mat.opacity_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        179 => { mat.use_image_file_for_bump_map = as_bool(try!(pair.value.assert_i16())); },
                        270 => {
                            if !read_bump_map_projection_method {
                                mat.bump_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16())));
                                read_bump_map_projection_method = true;
                            }
                            else if !read_luminance_mode {
                                mat.luminance_mode = try!(pair.value.assert_i16());
                                read_luminance_mode = true;
                            }
                            else {
                                mat.map_u_tile = try!(pair.value.assert_i16());
                            }
                        },
                        271 => {
                            if !read_bump_map_tiling_method {
                                mat.bump_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16())));
                                read_bump_map_tiling_method = true;
                            }
                            else if !read_normal_map_method {
                                mat.normal_map_method = try!(pair.value.assert_i16());
                                read_normal_map_method = true;
                            }
                            else {
                                mat.gen_proc_integer_value = try!(pair.value.assert_i16());
                            }
                        },
                        272 => {
                            if !read_bump_map_auto_transform_method {
                                mat.bump_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16())));
                                read_bump_map_auto_transform_method = true;
                            }
                            else {
                                mat.global_illumination_mode = try!(pair.value.assert_i16());
                            }
                        },
                        273 => {
                            if !read_use_image_file_for_refraction_map {
                                mat.use_image_file_for_refraction_map = as_bool(try!(pair.value.assert_i16()));
                                read_use_image_file_for_refraction_map = true;
                            }
                            else {
                                mat.final_gather_mode = try!(pair.value.assert_i16());
                            }
                        },
                        274 => { mat.refraction_map_projection_method = try_result!(MapProjectionMethod::from_i16(try!(pair.value.assert_i16()))); },
                        275 => { mat.refraction_map_tiling_method = try_result!(MapTilingMethod::from_i16(try!(pair.value.assert_i16()))); },
                        276 => { mat.refraction_map_auto_transform_method = try_result!(MapAutoTransformMethod::from_i16(try!(pair.value.assert_i16()))); },
                        290 => { mat.is_two_sided = try!(pair.value.assert_bool()); },
                        291 => { mat.gen_proc_boolean_value = try!(pair.value.assert_bool()); },
                        292 => { mat.gen_proc_table_end = try!(pair.value.assert_bool()); },
                        293 => { mat.is_anonymous = try!(pair.value.assert_bool()); },
                        300 => { mat.gen_proc_name = try!(pair.value.assert_string()); },
                        301 => { mat.gen_proc_text_value = try!(pair.value.assert_string()); },
                        420 => { mat.gen_proc_color_rgb_value = try!(pair.value.assert_i32()); },
                        430 => { mat.gen_proc_color_name = try!(pair.value.assert_string()); },
                        460 => { mat.color_bleed_scale = try!(pair.value.assert_f64()); },
                        461 => { mat.indirect_dump_scale = try!(pair.value.assert_f64()); },
                        462 => { mat.reflectance_scale = try!(pair.value.assert_f64()); },
                        463 => { mat.transmittance_scale = try!(pair.value.assert_f64()); },
                        464 => { mat.luminance = try!(pair.value.assert_f64()); },
                        465 => {
                            mat.normal_map_strength = try!(pair.value.assert_f64());
                            is_reading_normal = true;
                        },
                        468 => { mat.reflectivity = try!(pair.value.assert_f64()); },
                        469 => { mat.gen_proc_real_value = try!(pair.value.assert_f64()); },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::MLineStyle(ref mut mline) => {
                let mut read_element_count = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        2 => { mline.style_name = try!(pair.value.assert_string()); },
                        3 => { mline.description = try!(pair.value.assert_string()); },
                        6 => { mline._element_line_types.push(try!(pair.value.assert_string())); },
                        49 => { mline._element_offsets.push(try!(pair.value.assert_f64())); },
                        51 => { mline.start_angle = try!(pair.value.assert_f64()); },
                        52 => { mline.end_angle = try!(pair.value.assert_f64()); },
                        62 => {
                            if read_element_count {
                                mline._element_colors.push(Color::from_raw_value(try!(pair.value.assert_i16())));
                            }
                            else {
                                mline.fill_color = Color::from_raw_value(try!(pair.value.assert_i16()));
                            }
                        },
                        70 => { mline._flags = try!(pair.value.assert_i16()) as i32; },
                        71 => {
                            mline._element_count = try!(pair.value.assert_i16()) as i32;
                            read_element_count = true;
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::SectionSettings(ref mut ss) => {
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => {
                            // value should be "SectionTypeSettings", but it doesn't realy matter
                            loop {
                                match try!(SectionTypeSettings::read(iter)) {
                                    Some(ts) => ss.geometry_settings.push(ts),
                                    None => break,
                                }
                            }
                        },
                        90 => { ss.section_type = try!(pair.value.assert_i32()); }
                        91 => (), // generation settings count; we just read as many as we're given
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::SortentsTable(ref mut sort) => {
                let mut is_ready_for_sort_handles = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        5 => {
                            if is_ready_for_sort_handles {
                                sort.sort_items.push(try!(as_u32(try!(pair.value.assert_string()))));
                            }
                            else {
                                self.common.handle = try!(as_u32(try!(pair.value.assert_string())));
                                is_ready_for_sort_handles = true;
                            }
                        },
                        100 => { is_ready_for_sort_handles = true; },
                        330 => {
                            self.common.owner_handle = try!(as_u32(try!(pair.value.assert_string())));
                            is_ready_for_sort_handles = true;
                        },
                        331 => {
                            sort.entities.push(try!(as_u32(try!(pair.value.assert_string()))));
                            is_ready_for_sort_handles = true;
                        },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::SpatialFilter(ref mut sf) => {
                let mut read_front_clipping_plane = false;
                let mut set_inverse_matrix = false;
                let mut matrix_list = vec![];
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        10 => {
                            // code 10 always starts a new point
                            sf.clip_boundary_definition_points.push(Point::origin());
                            vec_last!(sf.clip_boundary_definition_points).x = try!(pair.value.assert_f64());
                        },
                        20 => { vec_last!(sf.clip_boundary_definition_points).y = try!(pair.value.assert_f64()); },
                        30 => { vec_last!(sf.clip_boundary_definition_points).z = try!(pair.value.assert_f64()); },
                        11 => { sf.clip_boundary_origin.x = try!(pair.value.assert_f64()); },
                        21 => { sf.clip_boundary_origin.y = try!(pair.value.assert_f64()); },
                        31 => { sf.clip_boundary_origin.z = try!(pair.value.assert_f64()); },
                        40 => {
                            if !read_front_clipping_plane {
                                sf.front_clipping_plane_distance = try!(pair.value.assert_f64());
                                read_front_clipping_plane = true;
                            }
                            else {
                                matrix_list.push(try!(pair.value.assert_f64()));
                                if matrix_list.len() == 12 {
                                    let mut matrix = TransformationMatrix::default();
                                    matrix.from_vec(&vec![
                                        matrix_list[0], matrix_list[1], matrix_list[2], 0.0,
                                        matrix_list[3], matrix_list[4], matrix_list[5], 0.0,
                                        matrix_list[6], matrix_list[7], matrix_list[8], 0.0,
                                        matrix_list[9], matrix_list[10], matrix_list[11], 0.0,
                                    ]);
                                    matrix_list.clear();
                                    if !set_inverse_matrix {
                                        sf.inverse_transformation_matrix = matrix;
                                        set_inverse_matrix = true;
                                    }
                                    else {
                                        sf.transformation_matrix = matrix;
                                    }
                                }
                            }
                        },
                        41 => { sf.back_clipping_plane_distance = try!(pair.value.assert_f64()); },
                        70 => (), // boundary point count; we just read as many as we're given
                        71 => { sf.is_clip_boundary_enabled = as_bool(try!(pair.value.assert_i16())); },
                        72 => { sf.is_front_clipping_plane = as_bool(try!(pair.value.assert_i16())); },
                        73 => { sf.is_back_clipping_plane = as_bool(try!(pair.value.assert_i16())); },
                        210 => { sf.clip_boundary_normal.x = try!(pair.value.assert_f64()); },
                        220 => { sf.clip_boundary_normal.y = try!(pair.value.assert_f64()); },
                        230 => { sf.clip_boundary_normal.z = try!(pair.value.assert_f64()); },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::SunStudy(ref mut ss) => {
                let mut seen_version = false;
                let mut reading_hours = false;
                let mut julian_day = None;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        1 => { ss.sun_setup_name = try!(pair.value.assert_string()); },
                        2 => { ss.description = try!(pair.value.assert_string()); },
                        3 => { ss.sheet_set_name = try!(pair.value.assert_string()); },
                        4 => { ss.sheet_subset_name = try!(pair.value.assert_string()); },
                        40 => { ss.spacing = try!(pair.value.assert_f64()); },
                        70 => { ss.output_type = try!(pair.value.assert_i16()); },
                        73 => { reading_hours = true; },
                        74 => { ss.shade_plot_type = try!(pair.value.assert_i16()); },
                        75 => { ss.viewports_per_page = try!(pair.value.assert_i16()) as i32; },
                        76 => { ss.viewport_distribution_row_count = try!(pair.value.assert_i16()) as i32; },
                        77 => { ss.viewport_distribution_column_count = try!(pair.value.assert_i16()) as i32; },
                        90 => {
                            if !seen_version {
                                ss.version = try!(pair.value.assert_i32());
                                seen_version = true;
                            }
                            else {
                                // after the version, 90 pairs come in julian_day/seconds_past_midnight duals
                                match julian_day {
                                    Some(jd) => {
                                        let date = as_datetime_local(jd as f64);
                                        let date = date.add(Duration::seconds(try!(pair.value.assert_i32()) as i64));
                                        ss.dates.push(date);
                                        julian_day = None;
                                    },
                                    None => {
                                        julian_day = Some(try!(pair.value.assert_i32()));
                                    },
                                }
                            }
                        },
                        93 => { ss.start_time_seconds_past_midnight = try!(pair.value.assert_i32()); },
                        94 => { ss.end_time_seconds_past_midnight = try!(pair.value.assert_i32()); },
                        95 => { ss.interval_in_seconds = try!(pair.value.assert_i32()); },
                        290 => {
                            if !reading_hours {
                                ss.use_subset = try!(pair.value.assert_bool());
                                reading_hours = true;
                            }
                            else {
                                ss.hours.push(try!(pair.value.assert_i16()) as i32);
                            }
                        },
                        291 => { ss.select_dates_from_calendar = try!(pair.value.assert_bool()); },
                        292 => { ss.select_range_of_dates = try!(pair.value.assert_bool()); },
                        293 => { ss.lock_viewports = try!(pair.value.assert_bool()); },
                        294 => { ss.label_viewports = try!(pair.value.assert_bool()); },
                        340 => { ss.page_setup_wizard = try!(as_u32(try!(pair.value.assert_string()))); },
                        341 => { ss.view = try!(as_u32(try!(pair.value.assert_string()))); },
                        342 => { ss.visual_style = try!(as_u32(try!(pair.value.assert_string()))); },
                        343 => { ss.text_style = try!(as_u32(try!(pair.value.assert_string()))); },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::TableStyle(ref mut ts) => {
                let mut read_version = false;
                loop {
                    let pair = next_pair!(iter);
                    match pair.code {
                        3 => { ts.description = try!(pair.value.assert_string()); },
                        7 => {
                            iter.put_back(Ok(pair)); // let the TableCellStyle reader parse this
                            if let Some(style) = try!(TableCellStyle::read(iter)) {
                                ts.cell_styles.push(style);
                            }
                        },
                        40 => { ts.horizontal_cell_margin = try!(pair.value.assert_f64()); },
                        41 => { ts.vertical_cell_margin = try!(pair.value.assert_f64()); },
                        70 => { ts.flow_direction = try_result!(FlowDirection::from_i16(try!(pair.value.assert_i16()))); },
                        71 => { ts.flags = try!(pair.value.assert_i16()) as i32; },
                        280 => {
                            if !read_version {
                                ts.version = try_result!(Version::from_i16(try!(pair.value.assert_i16())));
                                read_version = true;
                            }
                            else {
                                ts.is_title_suppressed = as_bool(try!(pair.value.assert_i16()));
                            }
                        },
                        281 => { ts.is_column_heading_suppressed = as_bool(try!(pair.value.assert_i16())); },
                        _ => { try!(self.common.apply_individual_pair(&pair, iter)); },
                    }
                }
            },
            ObjectType::XRecordObject(ref mut xr) => {
                let mut reading_data = false;
                loop {
                    let pair = next_pair!(iter);
                    if reading_data {
                        xr.data_pairs.push(pair);
                    }
                    else {
                        if pair.code == 280 {
                            xr.duplicate_record_handling = try_result!(DictionaryDuplicateRecordHandling::from_i16(try!(pair.value.assert_i16())));
                            reading_data = true;
                            continue;
                        }

                        if try!(self.common.apply_individual_pair(&pair, iter)) {
                            continue;
                        }

                        match pair.code {
                            100 => { continue; }, // value should be "AcDbXrecord", but it doesn't really matter
                            5 | 105 => (), // these codes aren't allowed here
                            _ => {
                                xr.data_pairs.push(pair);
                                reading_data = true;
                            },
                        }
                    }
                }
            },
            _ => return Ok(false), // no custom reader
        }
    }
    #[doc(hidden)]
    pub fn write<T>(&self, version: &AcadVersion, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        if self.specific.is_supported_on_version(version) {
            try!(writer.write_code_pair(&CodePair::new_str(0, self.specific.to_type_string())));
            try!(self.common.write(version, writer));
            if !try!(self.apply_custom_writer(version, writer)) {
                try!(self.specific.write(version, writer));
                try!(self.post_write(&version, writer));
            }
            for x in &self.common.x_data {
                try!(x.write(version, writer));
            }
        }

        Ok(())
    }
    fn apply_custom_writer<T>(&self, version: &AcadVersion, writer: &mut CodePairWriter<T>) -> DxfResult<bool>
        where T: Write {

        match self.specific {
            ObjectType::DataTable(ref data) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbDataTable")));
                try!(writer.write_code_pair(&CodePair::new_i16(70, data.field)));
                try!(writer.write_code_pair(&CodePair::new_i32(90, data.column_count as i32)));
                try!(writer.write_code_pair(&CodePair::new_i32(91, data.row_count as i32)));
                try!(writer.write_code_pair(&CodePair::new_string(1, &data.name)));
                for col in 0..data.column_count {
                    let column_code = match &data.values[0][col] {
                        &Some(DataTableValue::Boolean(_)) => Some(71),
                        &Some(DataTableValue::Integer(_)) => Some(93),
                        &Some(DataTableValue::Double(_)) => Some(40),
                        &Some(DataTableValue::Str(_)) => Some(3),
                        &Some(DataTableValue::Point2D(_)) => Some(10),
                        &Some(DataTableValue::Point3D(_)) => Some(11),
                        &Some(DataTableValue::Handle(_)) => Some(331),
                        &None => None,
                    };
                    if let Some(column_code) = column_code {
                        try!(writer.write_code_pair(&CodePair::new_i32(92, column_code)));
                        try!(writer.write_code_pair(&CodePair::new_string(2, &data.column_names[col])));
                        for row in 0..data.row_count {
                            match &data.values[row][col] {
                                &Some(DataTableValue::Boolean(val)) => { try!(writer.write_code_pair(&CodePair::new_i16(71, as_i16(val)))); },
                                &Some(DataTableValue::Integer(val)) => { try!(writer.write_code_pair(&CodePair::new_i32(93, val))); },
                                &Some(DataTableValue::Double(val)) => { try!(writer.write_code_pair(&CodePair::new_f64(40, val))); },
                                &Some(DataTableValue::Str(ref val)) => { try!(writer.write_code_pair(&CodePair::new_string(3, val))); },
                                &Some(DataTableValue::Point2D(ref val)) => {
                                    try!(writer.write_code_pair(&CodePair::new_f64(10, val.x)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(20, val.y)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(30, val.z)));
                                },
                                &Some(DataTableValue::Point3D(ref val)) => {
                                    try!(writer.write_code_pair(&CodePair::new_f64(11, val.x)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(21, val.y)));
                                    try!(writer.write_code_pair(&CodePair::new_f64(31, val.z)));
                                },
                                &Some(DataTableValue::Handle(val)) => { try!(writer.write_code_pair(&CodePair::new_string(331, &as_handle(val)))); },
                                &None => (),
                            }
                        }
                    }
                }
            },
            ObjectType::Dictionary(ref dict) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbDictionary")));
                if *version >= AcadVersion::R2000 && !dict.is_hard_owner {
                    try!(writer.write_code_pair(&CodePair::new_i16(280, as_i16(dict.is_hard_owner))));
                }
                if *version >= AcadVersion::R2000 {
                    try!(writer.write_code_pair(&CodePair::new_i16(281, dict.duplicate_record_handling as i16)));
                }
                let code = if dict.is_hard_owner { 360 } else { 350 };
                for key in dict.value_handles.keys().sorted_by(|a, b| Ord::cmp(a, b)) {
                    if let Some(value) = dict.value_handles.get(key) {
                        try!(writer.write_code_pair(&CodePair::new_string(3, key)));
                        try!(writer.write_code_pair(&CodePair::new_string(code, &as_handle(*value))));
                    }
                }
            },
            ObjectType::DictionaryWithDefault(ref dict) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbDictionary")));
                if *version >= AcadVersion::R2000 {
                    try!(writer.write_code_pair(&CodePair::new_i16(281, dict.duplicate_record_handling as i16)));
                }
                try!(writer.write_code_pair(&CodePair::new_string(340, &as_handle(dict.default_handle))));
                for key in dict.value_handles.keys().sorted_by(|a, b| Ord::cmp(a, b)) {
                    if let Some(value) = dict.value_handles.get(key) {
                        try!(writer.write_code_pair(&CodePair::new_string(3, key)));
                        try!(writer.write_code_pair(&CodePair::new_string(350, &as_handle(*value))));
                    }
                }
            },
            ObjectType::LightList(ref ll) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbLightList")));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ll.version)));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ll.lights.len() as i32)));
                for light in &ll.lights {
                    try!(writer.write_code_pair(&CodePair::new_string(5, &as_handle(*light))));
                    try!(writer.write_code_pair(&CodePair::new_string(1, &String::new()))); // TODO: write the light's real name
                }
            },
            ObjectType::SectionSettings(ref ss) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbSectionSettings")));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ss.section_type)));
                try!(writer.write_code_pair(&CodePair::new_i32(91, ss.geometry_settings.len() as i32)));
                for settings in &ss.geometry_settings {
                    try!(settings.write(writer));
                }
            },
            ObjectType::SunStudy(ref ss) => {
                try!(writer.write_code_pair(&CodePair::new_string(100, &String::from("AcDbSunStudy"))));
                try!(writer.write_code_pair(&CodePair::new_i32(90, ss.version)));
                try!(writer.write_code_pair(&CodePair::new_string(1, &ss.sun_setup_name)));
                try!(writer.write_code_pair(&CodePair::new_string(2, &ss.description)));
                try!(writer.write_code_pair(&CodePair::new_i16(70, ss.output_type)));
                try!(writer.write_code_pair(&CodePair::new_string(3, &ss.sheet_set_name)));
                try!(writer.write_code_pair(&CodePair::new_bool(290, ss.use_subset)));
                try!(writer.write_code_pair(&CodePair::new_string(4, &ss.sheet_subset_name)));
                try!(writer.write_code_pair(&CodePair::new_bool(291, ss.select_dates_from_calendar)));
                try!(writer.write_code_pair(&CodePair::new_i32(91, ss.dates.len() as i32)));
                for item in &ss.dates {
                    try!(writer.write_code_pair(&CodePair::new_i32(90, as_double_local(*item) as i32)));
                }
                try!(writer.write_code_pair(&CodePair::new_bool(292, ss.select_range_of_dates)));
                try!(writer.write_code_pair(&CodePair::new_i32(93, ss.start_time_seconds_past_midnight)));
                try!(writer.write_code_pair(&CodePair::new_i32(94, ss.end_time_seconds_past_midnight)));
                try!(writer.write_code_pair(&CodePair::new_i32(95, ss.interval_in_seconds)));
                try!(writer.write_code_pair(&CodePair::new_i16(73, ss.hours.len() as i16)));
                for v in &ss.hours {
                    try!(writer.write_code_pair(&CodePair::new_i16(290, *v as i16)));
                }
                try!(writer.write_code_pair(&CodePair::new_string(340, &as_handle(ss.page_setup_wizard))));
                try!(writer.write_code_pair(&CodePair::new_string(341, &as_handle(ss.view))));
                try!(writer.write_code_pair(&CodePair::new_string(342, &as_handle(ss.visual_style))));
                try!(writer.write_code_pair(&CodePair::new_i16(74, ss.shade_plot_type)));
                try!(writer.write_code_pair(&CodePair::new_i16(75, ss.viewports_per_page as i16)));
                try!(writer.write_code_pair(&CodePair::new_i16(76, ss.viewport_distribution_row_count as i16)));
                try!(writer.write_code_pair(&CodePair::new_i16(77, ss.viewport_distribution_column_count as i16)));
                try!(writer.write_code_pair(&CodePair::new_f64(40, ss.spacing)));
                try!(writer.write_code_pair(&CodePair::new_bool(293, ss.lock_viewports)));
                try!(writer.write_code_pair(&CodePair::new_bool(294, ss.label_viewports)));
                try!(writer.write_code_pair(&CodePair::new_string(343, &as_handle(ss.text_style))));
            },
            ObjectType::XRecordObject(ref xr) => {
                try!(writer.write_code_pair(&CodePair::new_str(100, "AcDbXrecord")));
                try!(writer.write_code_pair(&CodePair::new_i16(280, xr.duplicate_record_handling as i16)));
                for pair in &xr.data_pairs {
                    try!(writer.write_code_pair(&pair));
                }
            },
            _ => return Ok(false), // no custom writer
        }

        Ok(true)
    }
    fn post_write<T>(&self, _version: &AcadVersion, _writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        match self.specific {
            _ => (),
        }

        Ok(())
    }
}
