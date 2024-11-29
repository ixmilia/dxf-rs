// other implementation is in `generated/objects.rs`

use enum_primitive::FromPrimitive;
use itertools::Itertools;
use std::ops::Add;

use chrono::Duration;

use crate::{
    CodePair, Color, DataTableValue, DxfError, DxfResult, Point, SectionTypeSettings,
    TableCellStyle, TransformationMatrix,
};

use crate::code_pair_put_back::CodePairPutBack;
use crate::enums::*;
use crate::helper_functions::*;
use crate::objects::*;

//------------------------------------------------------------------------------
//                                                                  GeoMeshPoint
//------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct GeoMeshPoint {
    pub source: Point,
    pub destination: Point,
}

impl GeoMeshPoint {
    pub fn new(source: Point, destination: Point) -> Self {
        GeoMeshPoint {
            source,
            destination,
        }
    }
}

//------------------------------------------------------------------------------
//                                                             MLineStyleElement
//------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MLineStyleElement {
    pub offset: f64,
    pub color: Color,
    pub line_type: String,
}

impl MLineStyleElement {
    pub fn new(offset: f64, color: Color, line_type: String) -> Self {
        MLineStyleElement {
            offset,
            color,
            line_type,
        }
    }
}

//------------------------------------------------------------------------------
//                                                                     DataTable
//------------------------------------------------------------------------------
impl DataTable {
    pub(crate) fn set_value(&mut self, row: usize, col: usize, val: DataTableValue) {
        if row < self.row_count && col < self.column_count {
            self.values[row][col] = Some(val);
        }
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
            specific,
        }
    }
    /// Ensures all object values are valid.
    pub fn normalize(&mut self) {
        self.common.normalize();
        // no object-specific values to set
    }
    pub(crate) fn read(iter: &mut CodePairPutBack) -> DxfResult<Option<Object>> {
        loop {
            match iter.next() {
                // first code pair must be 0/object-type
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    let type_string = pair.assert_string()?;
                    if type_string == "ENDSEC" || type_string == "ENDBLK" {
                        iter.put_back(Ok(pair));
                        return Ok(None);
                    }

                    match ObjectType::from_type_string(&type_string) {
                        Some(e) => {
                            let mut obj = Object::new(e);
                            if !obj.apply_custom_reader(iter)? {
                                // no custom reader, use the auto-generated one
                                loop {
                                    match iter.next() {
                                        Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                            // new object or ENDSEC
                                            iter.put_back(Ok(pair));
                                            break;
                                        }
                                        Some(Ok(pair)) => obj.apply_code_pair(&pair, iter)?,
                                        Some(Err(e)) => return Err(e),
                                        None => return Err(DxfError::UnexpectedEndOfInput),
                                    }
                                }

                                obj.post_parse()?;
                            }

                            return Ok(Some(obj));
                        }
                        None => {
                            // swallow unsupported object
                            loop {
                                match iter.next() {
                                    Some(Ok(pair @ CodePair { code: 0, .. })) => {
                                        // found another object or ENDSEC
                                        iter.put_back(Ok(pair));
                                        break;
                                    }
                                    Some(Ok(_)) => (), // part of the unsupported object
                                    Some(Err(e)) => return Err(e),
                                    None => return Err(DxfError::UnexpectedEndOfInput),
                                }
                            }
                        }
                    }
                }
                Some(Ok(pair)) => {
                    return Err(DxfError::UnexpectedCodePair(
                        pair,
                        String::from("expected 0/object-type or 0/ENDSEC"),
                    ))
                }
                Some(Err(e)) => return Err(e),
                None => return Err(DxfError::UnexpectedEndOfInput),
            }
        }
    }
    fn apply_code_pair(&mut self, pair: &CodePair, iter: &mut CodePairPutBack) -> DxfResult<()> {
        if !self.specific.try_apply_code_pair(pair)? {
            self.common.apply_individual_pair(pair, iter)?;
        }
        Ok(())
    }
    fn post_parse(&mut self) -> DxfResult<()> {
        match self.specific {
            ObjectType::AcadProxyObject(ref mut proxy) => {
                for item in &proxy.__object_ids_a {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy.__object_ids_b {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy.__object_ids_c {
                    proxy.object_ids.push(item.clone());
                }
                for item in &proxy.__object_ids_d {
                    proxy.object_ids.push(item.clone());
                }
                proxy.__object_ids_a.clear();
                proxy.__object_ids_b.clear();
                proxy.__object_ids_c.clear();
                proxy.__object_ids_d.clear();
            }
            ObjectType::GeoData(ref mut geo) => {
                let mut source_points = vec![];
                let mut destination_points = vec![];
                combine_points_2(
                    &mut geo.__source_mesh_x_points,
                    &mut geo.__source_mesh_y_points,
                    &mut source_points,
                    Point::new,
                );
                combine_points_2(
                    &mut geo.__destination_mesh_x_points,
                    &mut geo.__destination_mesh_y_points,
                    &mut destination_points,
                    Point::new,
                );
                for (s, d) in source_points.drain(..).zip(destination_points.drain(..)) {
                    geo.geo_mesh_points.push(GeoMeshPoint::new(s, d));
                }

                combine_points_3(
                    &mut geo.__face_point_index_x,
                    &mut geo.__face_point_index_y,
                    &mut geo.__face_point_index_z,
                    &mut geo.face_indices,
                    Point::new,
                );
            }
            ObjectType::Material(ref mut material) => {
                material.diffuse_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__diffuse_map_transformation_matrix_values,
                );
                material.specular_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__specular_map_transformation_matrix_values,
                );
                material.reflection_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__reflection_map_transformation_matrix_values,
                );
                material.opacity_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__opacity_map_transformation_matrix_values,
                );
                material.bump_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__bump_map_transformation_matrix_values,
                );
                material.refraction_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__refraction_map_transformation_matrix_values,
                );
                material.normal_map_transformation_matrix = TransformationMatrix::from_vec(
                    &material.__normal_map_transformation_matrix_values,
                );
                material.__diffuse_map_transformation_matrix_values.clear();
                material.__specular_map_transformation_matrix_values.clear();
                material
                    .__reflection_map_transformation_matrix_values
                    .clear();
                material.__opacity_map_transformation_matrix_values.clear();
                material.__bump_map_transformation_matrix_values.clear();
                material
                    .__refraction_map_transformation_matrix_values
                    .clear();
                material.__normal_map_transformation_matrix_values.clear();
            }
            ObjectType::MLineStyle(ref mut mline) => {
                for (o, (c, l)) in mline.__element_offsets.drain(..).zip(
                    mline
                        .__element_colors
                        .drain(..)
                        .zip(mline.__element_line_types.drain(..)),
                ) {
                    mline.elements.push(MLineStyleElement::new(o, c, l));
                }
            }
            ObjectType::VbaProject(ref mut vba) => {
                // each char in each _hex_data should be added to `data` byte array
                let mut result = vec![];
                for b1 in &vba.__hex_data {
                    for b2 in b1 {
                        result.push(*b2);
                    }
                }

                vba.data = result;
                vba.__hex_data.clear();
            }
            _ => (),
        }

        Ok(())
    }
    fn apply_custom_reader(&mut self, iter: &mut CodePairPutBack) -> DxfResult<bool> {
        match self.specific {
            ObjectType::DataTable(ref mut data) => {
                Object::apply_custom_reader_datatable(&mut self.common, data, iter)
            }
            ObjectType::Dictionary(ref mut dict) => {
                Object::apply_custom_reader_dictionary(&mut self.common, dict, iter)
            }
            ObjectType::DictionaryWithDefault(ref mut dict) => {
                Object::apply_custom_reader_dictionarywithdefault(&mut self.common, dict, iter)
            }
            ObjectType::Layout(ref mut layout) => {
                Object::apply_custom_reader_layout(&mut self.common, layout, iter)
            }
            ObjectType::LightList(ref mut ll) => {
                Object::apply_custom_reader_lightlist(&mut self.common, ll, iter)
            }
            ObjectType::Material(ref mut mat) => {
                Object::apply_custom_reader_material(&mut self.common, mat, iter)
            }
            ObjectType::MLineStyle(ref mut mline) => {
                Object::apply_custom_reader_mlinestyle(&mut self.common, mline, iter)
            }
            ObjectType::SectionSettings(ref mut ss) => {
                Object::apply_custom_reader_sectionsettings(&mut self.common, ss, iter)
            }
            ObjectType::SortentsTable(ref mut sort) => {
                Object::apply_custom_reader_sortentstable(&mut self.common, sort, iter)
            }
            ObjectType::SpatialFilter(ref mut sf) => {
                Object::apply_custom_reader_spatialfilter(&mut self.common, sf, iter)
            }
            ObjectType::SunStudy(ref mut ss) => {
                Object::apply_custom_reader_sunstudy(&mut self.common, ss, iter)
            }
            ObjectType::TableStyle(ref mut ts) => {
                Object::apply_custom_reader_tablestyle(&mut self.common, ts, iter)
            }
            ObjectType::XRecordObject(ref mut xr) => {
                Object::apply_custom_reader_xrecordobject(&mut self.common, xr, iter)
            }
            _ => Ok(false), // no custom reader
        }
    }
    fn apply_custom_reader_datatable(
        common: &mut ObjectCommon,
        data: &mut DataTable,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
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
                1 => {
                    data.name = pair.assert_string()?;
                }
                70 => {
                    data.field = pair.assert_i16()?;
                }
                90 => {
                    data.column_count = pair.assert_i32()? as usize;
                    read_column_count = true;
                }
                91 => {
                    data.row_count = pair.assert_i32()? as usize;
                    read_row_count = true;
                }

                // column headers
                2 => {
                    data.column_names.push(pair.assert_string()?);
                }
                92 => {
                    _current_column_code = pair.assert_i32()?;
                    current_column += 1;
                    current_row = 0;
                }

                // column values
                3 => {
                    data.set_value(
                        current_row,
                        current_column,
                        DataTableValue::Str(pair.assert_string()?),
                    );
                }
                40 => {
                    data.set_value(
                        current_row,
                        current_column,
                        DataTableValue::Double(pair.assert_f64()?),
                    );
                }
                71 => {
                    data.set_value(
                        current_row,
                        current_column,
                        DataTableValue::Boolean(as_bool(pair.assert_i16()?)),
                    );
                }
                93 => {
                    data.set_value(
                        current_row,
                        current_column,
                        DataTableValue::Integer(pair.assert_i32()?),
                    );
                }
                10 => {
                    current_2d_point.x = pair.assert_f64()?;
                }
                20 => {
                    current_2d_point.y = pair.assert_f64()?;
                }
                30 => {
                    current_2d_point.z = pair.assert_f64()?;
                    data.set_value(
                        current_row,
                        current_column,
                        DataTableValue::Point2D(current_2d_point.clone()),
                    );
                    current_2d_point = Point::origin();
                }
                11 => {
                    current_3d_point.x = pair.assert_f64()?;
                }
                21 => {
                    current_3d_point.y = pair.assert_f64()?;
                }
                31 => {
                    current_3d_point.z = pair.assert_f64()?;
                    data.set_value(
                        current_row,
                        current_column,
                        DataTableValue::Point3D(current_3d_point.clone()),
                    );
                    current_3d_point = Point::origin();
                }
                330 | 331 | 340 | 350 | 360 => {
                    if read_row_count || read_column_count {
                        data.set_value(
                            current_row,
                            current_column,
                            DataTableValue::Handle(pair.as_handle()?),
                        );
                    } else {
                        common.apply_individual_pair(&pair, iter)?;
                    }
                }

                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
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
    }
    fn apply_custom_reader_dictionary(
        common: &mut ObjectCommon,
        dict: &mut Dictionary,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut last_entry_name = String::new();
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                3 => {
                    last_entry_name = pair.assert_string()?;
                }
                280 => {
                    dict.is_hard_owner = as_bool(pair.assert_i16()?);
                }
                281 => {
                    dict.duplicate_record_handling = enum_from_number!(
                        DictionaryDuplicateRecordHandling,
                        NotApplicable,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                350 | 360 => {
                    let handle = pair.as_handle()?;
                    dict.value_handles.insert(last_entry_name.clone(), handle);
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_dictionarywithdefault(
        common: &mut ObjectCommon,
        dict: &mut DictionaryWithDefault,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut last_entry_name = String::new();
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                3 => {
                    last_entry_name = pair.assert_string()?;
                }
                281 => {
                    dict.duplicate_record_handling = enum_from_number!(
                        DictionaryDuplicateRecordHandling,
                        NotApplicable,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                340 => {
                    dict.default_handle = pair.as_handle()?;
                }
                350 | 360 => {
                    let handle = pair.as_handle()?;
                    dict.value_handles.insert(last_entry_name.clone(), handle);
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_layout(
        common: &mut ObjectCommon,
        layout: &mut Layout,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut is_reading_plot_settings = true;
        loop {
            let pair = next_pair!(iter);
            if is_reading_plot_settings {
                if pair.code == 100 && pair.assert_string()? == "AcDbLayout" {
                    is_reading_plot_settings = false;
                } else {
                    common.apply_individual_pair(&pair, iter)?;
                }
            } else {
                match pair.code {
                    1 => {
                        layout.layout_name = pair.assert_string()?;
                    }
                    10 => {
                        layout.minimum_limits.x = pair.assert_f64()?;
                    }
                    20 => {
                        layout.minimum_limits.y = pair.assert_f64()?;
                    }
                    11 => {
                        layout.maximum_limits.x = pair.assert_f64()?;
                    }
                    21 => {
                        layout.maximum_limits.y = pair.assert_f64()?;
                    }
                    12 => {
                        layout.insertion_base_point.x = pair.assert_f64()?;
                    }
                    22 => {
                        layout.insertion_base_point.y = pair.assert_f64()?;
                    }
                    32 => {
                        layout.insertion_base_point.z = pair.assert_f64()?;
                    }
                    13 => {
                        layout.ucs_origin.x = pair.assert_f64()?;
                    }
                    23 => {
                        layout.ucs_origin.y = pair.assert_f64()?;
                    }
                    33 => {
                        layout.ucs_origin.z = pair.assert_f64()?;
                    }
                    14 => {
                        layout.minimum_extents.x = pair.assert_f64()?;
                    }
                    24 => {
                        layout.minimum_extents.y = pair.assert_f64()?;
                    }
                    34 => {
                        layout.minimum_extents.z = pair.assert_f64()?;
                    }
                    15 => {
                        layout.maximum_extents.x = pair.assert_f64()?;
                    }
                    25 => {
                        layout.maximum_extents.y = pair.assert_f64()?;
                    }
                    35 => {
                        layout.maximum_extents.z = pair.assert_f64()?;
                    }
                    16 => {
                        layout.ucs_x_axis.x = pair.assert_f64()?;
                    }
                    26 => {
                        layout.ucs_x_axis.y = pair.assert_f64()?;
                    }
                    36 => {
                        layout.ucs_x_axis.z = pair.assert_f64()?;
                    }
                    17 => {
                        layout.ucs_y_axis.x = pair.assert_f64()?;
                    }
                    27 => {
                        layout.ucs_y_axis.y = pair.assert_f64()?;
                    }
                    37 => {
                        layout.ucs_y_axis.z = pair.assert_f64()?;
                    }
                    70 => {
                        layout.layout_flags = i32::from(pair.assert_i16()?);
                    }
                    71 => {
                        layout.tab_order = i32::from(pair.assert_i16()?);
                    }
                    76 => {
                        layout.ucs_orthographic_type = enum_from_number!(
                            UcsOrthographicType,
                            NotOrthographic,
                            from_i16,
                            pair.assert_i16()?
                        );
                    }
                    146 => {
                        layout.elevation = pair.assert_f64()?;
                    }
                    330 => {
                        layout.__viewport_handle = pair.as_handle()?;
                    }
                    345 => {
                        layout.__table_record_handle = pair.as_handle()?;
                    }
                    346 => {
                        layout.__table_record_base_handle = pair.as_handle()?;
                    }
                    _ => {
                        common.apply_individual_pair(&pair, iter)?;
                    }
                }
            }
        }
    }
    fn apply_custom_reader_lightlist(
        common: &mut ObjectCommon,
        ll: &mut LightList,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut read_version_number = false;
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                1 => {} // don't worry about the light's name; it'll be read from the light entity directly
                5 => {
                    if read_version_number {
                        // pointer to a new light
                        ll.__lights_handle.push(pair.as_handle()?);
                    } else {
                        // might still be the handle
                        common.apply_individual_pair(&pair, iter)?;
                    }
                }
                90 => {
                    if read_version_number {
                        // count of lights is ignored since it's implicitly set by reading the values
                    } else {
                        ll.version = pair.assert_i32()?;
                        read_version_number = false;
                    }
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    #[allow(clippy::cognitive_complexity)]
    fn apply_custom_reader_material(
        common: &mut ObjectCommon,
        mat: &mut Material,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
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
                1 => {
                    mat.name = pair.assert_string()?;
                }
                2 => {
                    mat.description = pair.assert_string()?;
                }
                3 => {
                    if !read_diffuse_map_file_name {
                        mat.diffuse_map_file_name = pair.assert_string()?;
                        read_diffuse_map_file_name = true;
                    } else {
                        mat.normal_map_file_name = pair.assert_string()?;
                        is_reading_normal = true;
                    }
                }
                4 => {
                    mat.normal_map_file_name = pair.assert_string()?;
                }
                6 => {
                    mat.reflection_map_file_name = pair.assert_string()?;
                }
                7 => {
                    mat.opacity_map_file_name = pair.assert_string()?;
                }
                8 => {
                    mat.bump_map_file_name = pair.assert_string()?;
                }
                9 => {
                    mat.refraction_map_file_name = pair.assert_string()?;
                }
                40 => {
                    mat.ambient_color_factor = pair.assert_f64()?;
                }
                41 => {
                    mat.diffuse_color_factor = pair.assert_f64()?;
                }
                42 => {
                    if !read_diffuse_map_blend_factor {
                        mat.diffuse_map_blend_factor = pair.assert_f64()?;
                        read_diffuse_map_blend_factor = true;
                    } else {
                        mat.normal_map_blend_factor = pair.assert_f64()?;
                        is_reading_normal = true;
                    }
                }
                43 => {
                    if is_reading_normal {
                        mat.__normal_map_transformation_matrix_values
                            .push(pair.assert_f64()?);
                    } else {
                        mat.__diffuse_map_transformation_matrix_values
                            .push(pair.assert_f64()?);
                    }
                }
                44 => {
                    mat.specular_gloss_factor = pair.assert_f64()?;
                }
                45 => {
                    mat.specular_color_factor = pair.assert_f64()?;
                }
                46 => {
                    mat.specular_map_blend_factor = pair.assert_f64()?;
                }
                47 => {
                    mat.__specular_map_transformation_matrix_values
                        .push(pair.assert_f64()?);
                }
                48 => {
                    mat.reflection_map_blend_factor = pair.assert_f64()?;
                }
                49 => {
                    mat.__reflection_map_transformation_matrix_values
                        .push(pair.assert_f64()?);
                }
                62 => {
                    mat.gen_proc_color_index_value = Color::from_raw_value(pair.assert_i16()?);
                }
                70 => {
                    mat.override_ambient_color = as_bool(pair.assert_i16()?);
                }
                71 => {
                    mat.override_diffuse_color = as_bool(pair.assert_i16()?);
                }
                72 => {
                    if !read_image_file_diffuse_map {
                        mat.use_image_file_for_diffuse_map = as_bool(pair.assert_i16()?);
                        read_image_file_diffuse_map = true;
                    } else {
                        mat.use_image_file_for_normal_map = as_bool(pair.assert_i16()?);
                    }
                }
                73 => {
                    if !read_diffuse_map_projection_method {
                        mat.diffuse_map_projection_method = enum_from_number!(
                            MapProjectionMethod,
                            Planar,
                            from_i16,
                            pair.assert_i16()?
                        );
                        read_diffuse_map_projection_method = true;
                    } else {
                        mat.normal_map_projection_method = enum_from_number!(
                            MapProjectionMethod,
                            Planar,
                            from_i16,
                            pair.assert_i16()?
                        );
                        is_reading_normal = true;
                    }
                }
                74 => {
                    if !read_diffuse_map_tiling_method {
                        mat.diffuse_map_tiling_method =
                            enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                        read_diffuse_map_tiling_method = true;
                    } else {
                        mat.normal_map_tiling_method =
                            enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                        is_reading_normal = true;
                    }
                }
                75 => {
                    if !read_diffuse_map_auto_transform_method {
                        mat.diffuse_map_auto_transform_method = enum_from_number!(
                            MapAutoTransformMethod,
                            NoAutoTransform,
                            from_i16,
                            pair.assert_i16()?
                        );
                        read_diffuse_map_auto_transform_method = true;
                    } else {
                        mat.normal_map_auto_transform_method = enum_from_number!(
                            MapAutoTransformMethod,
                            NoAutoTransform,
                            from_i16,
                            pair.assert_i16()?
                        );
                        is_reading_normal = true;
                    }
                }
                76 => {
                    mat.override_specular_color = as_bool(pair.assert_i16()?);
                }
                77 => {
                    mat.use_image_file_for_specular_map = as_bool(pair.assert_i16()?);
                }
                78 => {
                    mat.specular_map_projection_method = enum_from_number!(
                        MapProjectionMethod,
                        Planar,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                79 => {
                    mat.specular_map_tiling_method =
                        enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                }
                90 => {
                    if !read_ambient_color_value {
                        mat.ambient_color_value = pair.assert_i32()?;
                        read_ambient_color_value = true;
                    } else {
                        mat.self_illumination = pair.assert_i32()?;
                    }
                }
                91 => {
                    mat.diffuse_color_value = pair.assert_i32()?;
                }
                92 => {
                    mat.specular_color_value = pair.assert_i32()?;
                }
                93 => {
                    mat.illumination_model = pair.assert_i32()?;
                }
                94 => {
                    mat.channel_flags = pair.assert_i32()?;
                }
                140 => {
                    mat.opacity_factor = pair.assert_f64()?;
                }
                141 => {
                    mat.opacity_map_blend_factor = pair.assert_f64()?;
                }
                142 => {
                    mat.__opacity_map_transformation_matrix_values
                        .push(pair.assert_f64()?);
                }
                143 => {
                    mat.bump_map_blend_factor = pair.assert_f64()?;
                }
                144 => {
                    mat.__bump_map_transformation_matrix_values
                        .push(pair.assert_f64()?);
                }
                145 => {
                    mat.refraction_index = pair.assert_f64()?;
                }
                146 => {
                    mat.refraction_map_blend_factor = pair.assert_f64()?;
                }
                147 => {
                    mat.__refraction_map_transformation_matrix_values
                        .push(pair.assert_f64()?);
                }
                148 => {
                    mat.translucence = pair.assert_f64()?;
                }
                170 => {
                    mat.specular_map_auto_transform_method = enum_from_number!(
                        MapAutoTransformMethod,
                        NoAutoTransform,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                171 => {
                    mat.use_image_file_for_reflection_map = as_bool(pair.assert_i16()?);
                }
                172 => {
                    mat.reflection_map_projection_method = enum_from_number!(
                        MapProjectionMethod,
                        Planar,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                173 => {
                    mat.reflection_map_tiling_method =
                        enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                }
                174 => {
                    mat.reflection_map_auto_transform_method = enum_from_number!(
                        MapAutoTransformMethod,
                        NoAutoTransform,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                175 => {
                    mat.use_image_file_for_opacity_map = as_bool(pair.assert_i16()?);
                }
                176 => {
                    mat.opacity_map_projection_method = enum_from_number!(
                        MapProjectionMethod,
                        Planar,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                177 => {
                    mat.opacity_map_tiling_method =
                        enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                }
                178 => {
                    mat.opacity_map_auto_transform_method = enum_from_number!(
                        MapAutoTransformMethod,
                        NoAutoTransform,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                179 => {
                    mat.use_image_file_for_bump_map = as_bool(pair.assert_i16()?);
                }
                270 => {
                    if !read_bump_map_projection_method {
                        mat.bump_map_projection_method = enum_from_number!(
                            MapProjectionMethod,
                            Planar,
                            from_i16,
                            pair.assert_i16()?
                        );
                        read_bump_map_projection_method = true;
                    } else if !read_luminance_mode {
                        mat.luminance_mode = pair.assert_i16()?;
                        read_luminance_mode = true;
                    } else {
                        mat.map_u_tile = pair.assert_i16()?;
                    }
                }
                271 => {
                    if !read_bump_map_tiling_method {
                        mat.bump_map_tiling_method =
                            enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                        read_bump_map_tiling_method = true;
                    } else if !read_normal_map_method {
                        mat.normal_map_method = pair.assert_i16()?;
                        read_normal_map_method = true;
                    } else {
                        mat.gen_proc_integer_value = pair.assert_i16()?;
                    }
                }
                272 => {
                    if !read_bump_map_auto_transform_method {
                        mat.bump_map_auto_transform_method = enum_from_number!(
                            MapAutoTransformMethod,
                            NoAutoTransform,
                            from_i16,
                            pair.assert_i16()?
                        );
                        read_bump_map_auto_transform_method = true;
                    } else {
                        mat.global_illumination_mode = pair.assert_i16()?;
                    }
                }
                273 => {
                    if !read_use_image_file_for_refraction_map {
                        mat.use_image_file_for_refraction_map = as_bool(pair.assert_i16()?);
                        read_use_image_file_for_refraction_map = true;
                    } else {
                        mat.final_gather_mode = pair.assert_i16()?;
                    }
                }
                274 => {
                    mat.refraction_map_projection_method = enum_from_number!(
                        MapProjectionMethod,
                        Planar,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                275 => {
                    mat.refraction_map_tiling_method =
                        enum_from_number!(MapTilingMethod, Tile, from_i16, pair.assert_i16()?);
                }
                276 => {
                    mat.refraction_map_auto_transform_method = enum_from_number!(
                        MapAutoTransformMethod,
                        NoAutoTransform,
                        from_i16,
                        pair.assert_i16()?
                    );
                }
                290 => {
                    mat.is_two_sided = pair.assert_bool()?;
                }
                291 => {
                    mat.gen_proc_boolean_value = pair.assert_bool()?;
                }
                292 => {
                    mat.gen_proc_table_end = pair.assert_bool()?;
                }
                293 => {
                    mat.is_anonymous = pair.assert_bool()?;
                }
                300 => {
                    mat.gen_proc_name = pair.assert_string()?;
                }
                301 => {
                    mat.gen_proc_text_value = pair.assert_string()?;
                }
                420 => {
                    mat.gen_proc_color_rgb_value = pair.assert_i32()?;
                }
                430 => {
                    mat.gen_proc_color_name = pair.assert_string()?;
                }
                460 => {
                    mat.color_bleed_scale = pair.assert_f64()?;
                }
                461 => {
                    mat.indirect_dump_scale = pair.assert_f64()?;
                }
                462 => {
                    mat.reflectance_scale = pair.assert_f64()?;
                }
                463 => {
                    mat.transmittance_scale = pair.assert_f64()?;
                }
                464 => {
                    mat.luminance = pair.assert_f64()?;
                }
                465 => {
                    mat.normal_map_strength = pair.assert_f64()?;
                    is_reading_normal = true;
                }
                468 => {
                    mat.reflectivity = pair.assert_f64()?;
                }
                469 => {
                    mat.gen_proc_real_value = pair.assert_f64()?;
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_mlinestyle(
        common: &mut ObjectCommon,
        mline: &mut MLineStyle,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut read_element_count = false;
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                2 => {
                    mline.style_name = pair.assert_string()?;
                }
                3 => {
                    mline.description = pair.assert_string()?;
                }
                6 => {
                    mline.__element_line_types.push(pair.assert_string()?);
                }
                49 => {
                    mline.__element_offsets.push(pair.assert_f64()?);
                }
                51 => {
                    mline.start_angle = pair.assert_f64()?;
                }
                52 => {
                    mline.end_angle = pair.assert_f64()?;
                }
                62 => {
                    if read_element_count {
                        mline
                            .__element_colors
                            .push(Color::from_raw_value(pair.assert_i16()?));
                    } else {
                        mline.fill_color = Color::from_raw_value(pair.assert_i16()?);
                    }
                }
                70 => {
                    mline.__flags = i32::from(pair.assert_i16()?);
                }
                71 => {
                    mline.__element_count = i32::from(pair.assert_i16()?);
                    read_element_count = true;
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_sectionsettings(
        common: &mut ObjectCommon,
        ss: &mut SectionSettings,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                1 => {
                    // value should be "SectionTypeSettings", but it doesn't really matter
                    while let Some(ts) = SectionTypeSettings::read(iter)? {
                        ss.geometry_settings.push(ts);
                    }
                }
                90 => {
                    ss.section_type = pair.assert_i32()?;
                }
                91 => (), // generation settings count; we just read as many as we're given
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_sortentstable(
        common: &mut ObjectCommon,
        sort: &mut SortentsTable,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut is_ready_for_sort_handles = false;
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                5 => {
                    if is_ready_for_sort_handles {
                        sort.__sort_items_handle.push(pair.as_handle()?);
                    } else {
                        common.handle = pair.as_handle()?;
                        is_ready_for_sort_handles = true;
                    }
                }
                100 => {
                    is_ready_for_sort_handles = true;
                }
                330 => {
                    common.__owner_handle = pair.as_handle()?;
                    is_ready_for_sort_handles = true;
                }
                331 => {
                    sort.__entities_handle.push(pair.as_handle()?);
                    is_ready_for_sort_handles = true;
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_spatialfilter(
        common: &mut ObjectCommon,
        sf: &mut SpatialFilter,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut read_front_clipping_plane = false;
        let mut set_inverse_matrix = false;
        let mut matrix_list = vec![];
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                10 => {
                    // code 10 always starts a new point
                    sf.clip_boundary_definition_points.push(Point::origin());
                    vec_last!(sf.clip_boundary_definition_points).x = pair.assert_f64()?;
                }
                20 => {
                    vec_last!(sf.clip_boundary_definition_points).y = pair.assert_f64()?;
                }
                30 => {
                    vec_last!(sf.clip_boundary_definition_points).z = pair.assert_f64()?;
                }
                11 => {
                    sf.clip_boundary_origin.x = pair.assert_f64()?;
                }
                21 => {
                    sf.clip_boundary_origin.y = pair.assert_f64()?;
                }
                31 => {
                    sf.clip_boundary_origin.z = pair.assert_f64()?;
                }
                40 => {
                    if !read_front_clipping_plane {
                        sf.front_clipping_plane_distance = pair.assert_f64()?;
                        read_front_clipping_plane = true;
                    } else {
                        matrix_list.push(pair.assert_f64()?);
                        if matrix_list.len() == 12 {
                            let matrix = TransformationMatrix::from_vec(&[
                                matrix_list[0],
                                matrix_list[1],
                                matrix_list[2],
                                0.0,
                                matrix_list[3],
                                matrix_list[4],
                                matrix_list[5],
                                0.0,
                                matrix_list[6],
                                matrix_list[7],
                                matrix_list[8],
                                0.0,
                                matrix_list[9],
                                matrix_list[10],
                                matrix_list[11],
                                0.0,
                            ]);
                            matrix_list.clear();
                            if !set_inverse_matrix {
                                sf.inverse_transformation_matrix = matrix;
                                set_inverse_matrix = true;
                            } else {
                                sf.transformation_matrix = matrix;
                            }
                        }
                    }
                }
                41 => {
                    sf.back_clipping_plane_distance = pair.assert_f64()?;
                }
                70 => (), // boundary point count; we just read as many as we're given
                71 => {
                    sf.is_clip_boundary_enabled = as_bool(pair.assert_i16()?);
                }
                72 => {
                    sf.is_front_clipping_plane = as_bool(pair.assert_i16()?);
                }
                73 => {
                    sf.is_back_clipping_plane = as_bool(pair.assert_i16()?);
                }
                210 => {
                    sf.clip_boundary_normal.x = pair.assert_f64()?;
                }
                220 => {
                    sf.clip_boundary_normal.y = pair.assert_f64()?;
                }
                230 => {
                    sf.clip_boundary_normal.z = pair.assert_f64()?;
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_sunstudy(
        common: &mut ObjectCommon,
        ss: &mut SunStudy,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut seen_version = false;
        let mut reading_hours = false;
        let mut julian_day = None;
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                1 => {
                    ss.sun_setup_name = pair.assert_string()?;
                }
                2 => {
                    ss.description = pair.assert_string()?;
                }
                3 => {
                    ss.sheet_set_name = pair.assert_string()?;
                }
                4 => {
                    ss.sheet_subset_name = pair.assert_string()?;
                }
                40 => {
                    ss.spacing = pair.assert_f64()?;
                }
                70 => {
                    ss.output_type = pair.assert_i16()?;
                }
                73 => {
                    reading_hours = true;
                }
                74 => {
                    ss.shade_plot_type = pair.assert_i16()?;
                }
                75 => {
                    ss.viewports_per_page = i32::from(pair.assert_i16()?);
                }
                76 => {
                    ss.viewport_distribution_row_count = i32::from(pair.assert_i16()?);
                }
                77 => {
                    ss.viewport_distribution_column_count = i32::from(pair.assert_i16()?);
                }
                90 => {
                    if !seen_version {
                        ss.version = pair.assert_i32()?;
                        seen_version = true;
                    } else {
                        // after the version, 90 pairs come in julian_day/seconds_past_midnight duals
                        match julian_day {
                            Some(jd) => {
                                let date = as_datetime_local(f64::from(jd));
                                let date =
                                    date.add(Duration::seconds(i64::from(pair.assert_i32()?)));
                                ss.dates.push(date);
                                julian_day = None;
                            }
                            None => {
                                julian_day = Some(pair.assert_i32()?);
                            }
                        }
                    }
                }
                93 => {
                    ss.start_time_seconds_past_midnight = pair.assert_i32()?;
                }
                94 => {
                    ss.end_time_seconds_past_midnight = pair.assert_i32()?;
                }
                95 => {
                    ss.interval_in_seconds = pair.assert_i32()?;
                }
                290 => {
                    if !reading_hours {
                        ss.use_subset = pair.assert_bool()?;
                        reading_hours = true;
                    } else {
                        ss.hours.push(i32::from(pair.assert_i16()?));
                    }
                }
                291 => {
                    ss.select_dates_from_calendar = pair.assert_bool()?;
                }
                292 => {
                    ss.select_range_of_dates = pair.assert_bool()?;
                }
                293 => {
                    ss.lock_viewports = pair.assert_bool()?;
                }
                294 => {
                    ss.label_viewports = pair.assert_bool()?;
                }
                340 => {
                    ss.__page_setup_wizard_handle = pair.as_handle()?;
                }
                341 => {
                    ss.__view_handle = pair.as_handle()?;
                }
                342 => {
                    ss.__visual_style_handle = pair.as_handle()?;
                }
                343 => {
                    ss.__text_style_handle = pair.as_handle()?;
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_tablestyle(
        common: &mut ObjectCommon,
        ts: &mut TableStyle,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut read_version = false;
        loop {
            let pair = next_pair!(iter);
            match pair.code {
                3 => {
                    ts.description = pair.assert_string()?;
                }
                7 => {
                    iter.put_back(Ok(pair)); // let the TableCellStyle reader parse this
                    if let Some(style) = TableCellStyle::read(iter)? {
                        ts.cell_styles.push(style);
                    }
                }
                40 => {
                    ts.horizontal_cell_margin = pair.assert_f64()?;
                }
                41 => {
                    ts.vertical_cell_margin = pair.assert_f64()?;
                }
                70 => {
                    ts.flow_direction =
                        enum_from_number!(FlowDirection, Down, from_i16, pair.assert_i16()?);
                }
                71 => {
                    ts.flags = i32::from(pair.assert_i16()?);
                }
                280 => {
                    if !read_version {
                        ts.version =
                            enum_from_number!(Version, R2010, from_i16, pair.assert_i16()?);
                        read_version = true;
                    } else {
                        ts.is_title_suppressed = as_bool(pair.assert_i16()?);
                    }
                }
                281 => {
                    ts.is_column_heading_suppressed = as_bool(pair.assert_i16()?);
                }
                _ => {
                    common.apply_individual_pair(&pair, iter)?;
                }
            }
        }
    }
    fn apply_custom_reader_xrecordobject(
        common: &mut ObjectCommon,
        xr: &mut XRecordObject,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<bool> {
        let mut reading_data = false;
        loop {
            let pair = next_pair!(iter);
            if reading_data {
                xr.data_pairs.push(pair);
            } else {
                if pair.code == 280 {
                    xr.duplicate_record_handling = enum_from_number!(
                        DictionaryDuplicateRecordHandling,
                        NotApplicable,
                        from_i16,
                        pair.assert_i16()?
                    );
                    reading_data = true;
                    continue;
                }

                if common.apply_individual_pair(&pair, iter)? {
                    continue;
                }

                match pair.code {
                    100 => {
                        reading_data = true;
                        continue;
                    } // value should be "AcDbXrecord", some dxf files still need to keep the object read
                    5 | 105 => (), // these codes aren't allowed here
                    _ => {
                        xr.data_pairs.push(pair);
                        reading_data = true;
                    }
                }
            }
        }
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) {
        if self.specific.is_supported_on_version(version) {
            pairs.push(CodePair::new_str(0, self.specific.to_type_string()));
            self.common.add_code_pairs(pairs, version);
            if !self.add_custom_code_pairs(pairs, version) {
                self.specific.add_code_pairs(pairs, version);
                self.add_post_code_pairs(pairs, version);
            }
            for x in &self.common.x_data {
                x.add_code_pairs(pairs, version);
            }
        }
    }
    fn add_custom_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) -> bool {
        match self.specific {
            ObjectType::DataTable(ref data) => {
                pairs.push(CodePair::new_str(100, "AcDbDataTable"));
                pairs.push(CodePair::new_i16(70, data.field));
                pairs.push(CodePair::new_i32(90, data.column_count as i32));
                pairs.push(CodePair::new_i32(91, data.row_count as i32));
                pairs.push(CodePair::new_string(1, &data.name));
                for col in 0..data.column_count {
                    let column_code = match data.values[0][col] {
                        Some(DataTableValue::Boolean(_)) => Some(71),
                        Some(DataTableValue::Integer(_)) => Some(93),
                        Some(DataTableValue::Double(_)) => Some(40),
                        Some(DataTableValue::Str(_)) => Some(3),
                        Some(DataTableValue::Point2D(_)) => Some(10),
                        Some(DataTableValue::Point3D(_)) => Some(11),
                        Some(DataTableValue::Handle(_)) => Some(331),
                        None => None,
                    };
                    if let Some(column_code) = column_code {
                        pairs.push(CodePair::new_i32(92, column_code));
                        pairs.push(CodePair::new_string(2, &data.column_names[col]));
                        for row in 0..data.row_count {
                            match data.values[row][col] {
                                Some(DataTableValue::Boolean(val)) => {
                                    pairs.push(CodePair::new_i16(71, as_i16(val)));
                                }
                                Some(DataTableValue::Integer(val)) => {
                                    pairs.push(CodePair::new_i32(93, val));
                                }
                                Some(DataTableValue::Double(val)) => {
                                    pairs.push(CodePair::new_f64(40, val));
                                }
                                Some(DataTableValue::Str(ref val)) => {
                                    pairs.push(CodePair::new_string(3, val));
                                }
                                Some(DataTableValue::Point2D(ref val)) => {
                                    pairs.push(CodePair::new_f64(10, val.x));
                                    pairs.push(CodePair::new_f64(20, val.y));
                                    pairs.push(CodePair::new_f64(30, val.z));
                                }
                                Some(DataTableValue::Point3D(ref val)) => {
                                    pairs.push(CodePair::new_f64(11, val.x));
                                    pairs.push(CodePair::new_f64(21, val.y));
                                    pairs.push(CodePair::new_f64(31, val.z));
                                }
                                Some(DataTableValue::Handle(val)) => {
                                    pairs.push(CodePair::new_string(331, &val.as_string()));
                                }
                                None => (),
                            }
                        }
                    }
                }
            }
            ObjectType::Dictionary(ref dict) => {
                pairs.push(CodePair::new_str(100, "AcDbDictionary"));
                if version >= AcadVersion::R2000 && !dict.is_hard_owner {
                    pairs.push(CodePair::new_i16(280, as_i16(dict.is_hard_owner)));
                }
                if version >= AcadVersion::R2000 {
                    pairs.push(CodePair::new_i16(
                        281,
                        dict.duplicate_record_handling as i16,
                    ));
                }
                let code = if dict.is_hard_owner { 360 } else { 350 };
                for key in dict.value_handles.keys().sorted_by(Ord::cmp) {
                    if let Some(value) = dict.value_handles.get(key) {
                        pairs.push(CodePair::new_string(3, key));
                        pairs.push(CodePair::new_string(code, &value.as_string()));
                    }
                }
            }
            ObjectType::DictionaryWithDefault(ref dict) => {
                pairs.push(CodePair::new_str(100, "AcDbDictionary"));
                if version >= AcadVersion::R2000 {
                    pairs.push(CodePair::new_i16(
                        281,
                        dict.duplicate_record_handling as i16,
                    ));
                }
                pairs.push(CodePair::new_string(340, &dict.default_handle.as_string()));
                for key in dict.value_handles.keys().sorted_by(Ord::cmp) {
                    if let Some(value) = dict.value_handles.get(key) {
                        pairs.push(CodePair::new_string(3, key));
                        pairs.push(CodePair::new_string(350, &value.as_string()));
                    }
                }
            }
            ObjectType::LightList(ref ll) => {
                pairs.push(CodePair::new_str(100, "AcDbLightList"));
                pairs.push(CodePair::new_i32(90, ll.version));
                pairs.push(CodePair::new_i32(90, ll.__lights_handle.len() as i32));
                for light in &ll.__lights_handle {
                    pairs.push(CodePair::new_string(5, &light.as_string()));
                    // TODO: write the light's real name
                    pairs.push(CodePair::new_string(1, ""));
                }
            }
            ObjectType::SectionSettings(ref ss) => {
                pairs.push(CodePair::new_str(100, "AcDbSectionSettings"));
                pairs.push(CodePair::new_i32(90, ss.section_type));
                pairs.push(CodePair::new_i32(91, ss.geometry_settings.len() as i32));
                for settings in &ss.geometry_settings {
                    settings.add_code_pairs(pairs);
                }
            }
            ObjectType::SunStudy(ref ss) => {
                pairs.push(CodePair::new_string(100, &String::from("AcDbSunStudy")));
                pairs.push(CodePair::new_i32(90, ss.version));
                pairs.push(CodePair::new_string(1, &ss.sun_setup_name));
                pairs.push(CodePair::new_string(2, &ss.description));
                pairs.push(CodePair::new_i16(70, ss.output_type));
                pairs.push(CodePair::new_string(3, &ss.sheet_set_name));
                pairs.push(CodePair::new_bool(290, ss.use_subset));
                pairs.push(CodePair::new_string(4, &ss.sheet_subset_name));
                pairs.push(CodePair::new_bool(291, ss.select_dates_from_calendar));
                pairs.push(CodePair::new_i32(91, ss.dates.len() as i32));
                for item in &ss.dates {
                    pairs.push(CodePair::new_i32(90, as_double_local(*item) as i32));
                }
                pairs.push(CodePair::new_bool(292, ss.select_range_of_dates));
                pairs.push(CodePair::new_i32(93, ss.start_time_seconds_past_midnight));
                pairs.push(CodePair::new_i32(94, ss.end_time_seconds_past_midnight));
                pairs.push(CodePair::new_i32(95, ss.interval_in_seconds));
                pairs.push(CodePair::new_i16(73, ss.hours.len() as i16));
                for v in &ss.hours {
                    pairs.push(CodePair::new_i16(290, *v as i16));
                }
                pairs.push(CodePair::new_string(
                    340,
                    &ss.__page_setup_wizard_handle.as_string(),
                ));
                pairs.push(CodePair::new_string(341, &ss.__view_handle.as_string()));
                pairs.push(CodePair::new_string(
                    342,
                    &ss.__visual_style_handle.as_string(),
                ));
                pairs.push(CodePair::new_i16(74, ss.shade_plot_type));
                pairs.push(CodePair::new_i16(75, ss.viewports_per_page as i16));
                pairs.push(CodePair::new_i16(
                    76,
                    ss.viewport_distribution_row_count as i16,
                ));
                pairs.push(CodePair::new_i16(
                    77,
                    ss.viewport_distribution_column_count as i16,
                ));
                pairs.push(CodePair::new_f64(40, ss.spacing));
                pairs.push(CodePair::new_bool(293, ss.lock_viewports));
                pairs.push(CodePair::new_bool(294, ss.label_viewports));
                pairs.push(CodePair::new_string(
                    343,
                    &ss.__text_style_handle.as_string(),
                ));
            }
            ObjectType::XRecordObject(ref xr) => {
                pairs.push(CodePair::new_str(100, "AcDbXrecord"));
                pairs.push(CodePair::new_i16(280, xr.duplicate_record_handling as i16));
                for pair in &xr.data_pairs {
                    pairs.push(pair.clone());
                }
            }
            _ => return false, // no custom writer
        }

        true
    }
    fn add_post_code_pairs(&self, _pairs: &mut [CodePair], _version: AcadVersion) {
        // use the following pattern if this method is needed
        // match self.specific {
        //     _ => (),
        // }
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::*;
    use crate::helper_functions::tests::*;
    use crate::objects::*;
    use crate::*;

    fn read_object(object_type: &str, body: Vec<CodePair>) -> Object {
        let mut pairs = vec![CodePair::new_str(0, object_type)];
        for pair in body {
            pairs.push(pair);
        }
        let drawing = from_section("OBJECTS", pairs);
        let objects = drawing.objects().collect::<Vec<_>>();
        assert_eq!(1, objects.len());
        objects[0].clone()
    }

    #[test]
    fn read_empty_objects_section() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(0, drawing.objects().count());
    }

    #[test]
    fn read_unsupported_object() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "UNSUPPORTED_OBJECT"),
            CodePair::new_str(1, "unsupported string"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(0, drawing.objects().count());
    }

    #[test]
    fn read_unsupported_object_between_supported_objects() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "OBJECTS"),
            CodePair::new_str(0, "DICTIONARYVAR"),
            CodePair::new_str(0, "UNSUPPORTED_OBJECT"),
            CodePair::new_str(1, "unsupported string"),
            CodePair::new_str(0, "IMAGEDEF"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        let objects = drawing.objects().collect::<Vec<_>>();
        assert_eq!(2, objects.len());
        match objects[0].specific {
            ObjectType::DictionaryVariable(_) => (),
            _ => panic!("expected a dictionary variable"),
        }
        match objects[1].specific {
            ObjectType::ImageDefinition(_) => (),
            _ => panic!("expected an image definition"),
        }
    }

    #[test]
    fn read_common_object_fields() {
        let obj = read_object("IMAGEDEF", vec![CodePair::new_str(5, "DEADBEEF")]);
        assert_eq!(Handle(0xDEAD_BEEF), obj.common.handle);
    }

    #[test]
    fn read_image_def() {
        let obj = read_object(
            "IMAGEDEF",
            vec![
                CodePair::new_str(1, "path/to/file"), // path
                CodePair::new_f64(10, 11.0),          // image_width
                CodePair::new_f64(20, 22.0),          // image_height
            ],
        );
        match obj.specific {
            ObjectType::ImageDefinition(ref img) => {
                assert_eq!(11, img.image_width);
                assert_eq!(22, img.image_height);
            }
            _ => panic!("expected an image definition"),
        }
    }

    #[test]
    fn write_common_object_fields() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14; // IMAGEDEF is only supported on R14+
        let obj = Object {
            common: Default::default(),
            specific: ObjectType::ImageDefinition(Default::default()),
        };
        drawing.add_object(obj);
        assert_contains_pairs(
            &drawing,
            vec![CodePair::new_str(0, "IMAGEDEF"), CodePair::new_str(5, "10")],
        );
    }

    #[test]
    fn write_specific_object_fields() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14; // IMAGEDEF is only supported on R14+
        let img = ImageDefinition {
            file_path: String::from("path/to/file"),
            ..Default::default()
        };
        drawing.add_object(Object::new(ObjectType::ImageDefinition(img)));
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(100, "AcDbRasterImageDef"),
                CodePair::new_i32(90, 0),
                CodePair::new_str(1, "path/to/file"),
            ],
        );
    }

    #[test]
    fn read_multiple_objects() {
        let drawing = from_section(
            "OBJECTS",
            vec![
                CodePair::new_str(0, "DICTIONARYVAR"),
                CodePair::new_str(1, "value"), // value
                CodePair::new_str(0, "IMAGEDEF"),
                CodePair::new_str(1, "path/to/file"), // file_path
                CodePair::new_f64(10, 11.0),          // image_width
            ],
        );
        let objects = drawing.objects().collect::<Vec<_>>();
        assert_eq!(2, objects.len());

        // verify dictionary value
        match objects[0].specific {
            ObjectType::DictionaryVariable(ref var) => {
                assert_eq!("value", var.value);
            }
            _ => panic!("expected a dictionary variable"),
        }

        // verify image definition
        match objects[1].specific {
            ObjectType::ImageDefinition(ref img) => {
                assert_eq!("path/to/file", img.file_path);
                assert_eq!(11, img.image_width);
            }
            _ => panic!("expected an image definition"),
        }
    }

    #[test]
    fn read_field_with_multiples_specific() {
        let obj = read_object(
            "LAYER_FILTER",
            vec![
                CodePair::new_str(8, "one"),
                CodePair::new_str(8, "two"),
                CodePair::new_str(8, "three"),
            ],
        );
        match obj.specific {
            ObjectType::LayerFilter(ref layer_filter) => {
                assert_eq!(vec!["one", "two", "three"], layer_filter.layer_names);
            }
            _ => panic!("expected a layer filter"),
        }
    }

    #[test]
    fn write_field_with_multiples_specific() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2004; // LAYER_FILTER is only supported up to 2004
        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::LayerFilter(LayerFilter {
                layer_names: vec![
                    String::from("one"),
                    String::from("two"),
                    String::from("three"),
                ],
            }),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(8, "one"),
                CodePair::new_str(8, "two"),
                CodePair::new_str(8, "three"),
            ],
        );
    }

    #[test]
    fn read_object_with_post_parse() {
        let obj = read_object(
            "VBA_PROJECT",
            vec![
                CodePair::new_binary(310, vec![0xDE, 0xAD, 0xBE, 0xEF]), // data
                CodePair::new_binary(310, vec![0x01, 0x23, 0x45, 0x67]),
            ],
        );
        match obj.specific {
            ObjectType::VbaProject(ref vba) => {
                assert_eq!(8, vba.data.len());
                assert_eq!(
                    vec![0xde, 0xad, 0xbe, 0xef, 0x01, 0x23, 0x45, 0x67],
                    vba.data
                );
            }
            _ => panic!("expected a VBA_PROJECT"),
        }
    }

    #[test]
    fn write_object_with_write_order() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2004; // LAYER_FILTER is only supported up to 2004
        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::LayerFilter(LayerFilter {
                layer_names: vec![
                    String::from("one"),
                    String::from("two"),
                    String::from("three"),
                ],
            }),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(100, "AcDbFilter"),
                CodePair::new_str(100, "AcDbLayerFilter"),
                CodePair::new_str(8, "one"),
                CodePair::new_str(8, "two"),
                CodePair::new_str(8, "three"),
            ],
        );
    }

    #[test]
    fn read_object_with_flags() {
        let obj = read_object(
            "LAYOUT",
            vec![
                CodePair::new_str(100, "AcDbLayout"),
                CodePair::new_i16(70, 3),
            ],
        );
        match obj.specific {
            ObjectType::Layout(ref layout) => {
                assert!(layout.is_ps_lt_scale());
                assert!(layout.is_lim_check());
            }
            _ => panic!("expected a LAYOUT"),
        }
    }

    #[test]
    fn write_object_with_flags() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2000; // LAYOUT is only supported up to R2000
        let mut layout = Layout::default();
        assert_eq!(0, layout.layout_flags);
        layout.set_is_ps_lt_scale(true);
        layout.set_is_lim_check(true);
        layout.tab_order = -54;
        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::Layout(layout),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_i16(70, 3),   // flags
                CodePair::new_i16(71, -54), // sentinel to make sure we're not reading a header value
            ],
        );
    }

    #[test]
    fn read_object_with_handles() {
        let obj = read_object(
            "LIGHTLIST",
            vec![
                CodePair::new_str(5, "A1"),
                CodePair::new_str(330, "A2"), // owner handle
            ],
        );
        assert_eq!(Handle(0xa1), obj.common.handle);
        assert_eq!(Handle(0xa2), obj.common.__owner_handle);
        match obj.specific {
            ObjectType::LightList(_) => (),
            _ => panic!("expected a light list"),
        }
    }

    #[test]
    fn write_object_with_handles() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2007; // LIGHTLIST only supported up to 2007
        drawing.add_object(Object {
            common: ObjectCommon {
                __owner_handle: Handle(0xa2),
                ..Default::default()
            },
            specific: ObjectType::LightList(Default::default()),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "LIGHTLIST"),
                CodePair::new_str(5, "10"),
                CodePair::new_str(330, "A2"),
            ],
        );
    }

    #[test]
    fn read_dictionary() {
        let dict = read_object(
            "DICTIONARY",
            vec![
                CodePair::new_str(3, "key1"),
                CodePair::new_str(350, "AAAA"),
                CodePair::new_str(3, "key2"),
                CodePair::new_str(350, "BBBB"),
            ],
        );
        match dict.specific {
            ObjectType::Dictionary(ref dict) => {
                assert_eq!(2, dict.value_handles.len());
                assert_eq!(Some(&Handle(0xAAAA)), dict.value_handles.get("key1"));
                assert_eq!(Some(&Handle(0xBBBB)), dict.value_handles.get("key2"));
            }
            _ => panic!("expected a dictionary"),
        }
    }

    #[test]
    fn write_dictionary() {
        let mut dict = Dictionary::default();
        dict.value_handles
            .insert(String::from("key1"), Handle(0xAAAA));
        dict.value_handles
            .insert(String::from("key2"), Handle(0xBBBB));
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R13; // OBJECTS section only written on R13+
        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::Dictionary(dict),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(3, "key1"),
                CodePair::new_str(350, "AAAA"),
                CodePair::new_str(3, "key2"),
                CodePair::new_str(350, "BBBB"),
            ],
        );
    }

    #[test]
    fn read_sunstudy() {
        // validates that code 290 values (ideally boolean) can be read as integers, too
        let ss = read_object(
            "SUNSTUDY",
            vec![
                CodePair::new_bool(290, true), // use_subset
                CodePair::new_i16(290, 3),     // hours
                CodePair::new_i16(290, 4),
                CodePair::new_i16(290, 5),
            ],
        );
        match ss.specific {
            ObjectType::SunStudy(ref ss) => {
                assert!(ss.use_subset);
                assert_eq!(vec![3, 4, 5], ss.hours);
            }
            _ => panic!("expected a sunstudy"),
        }
    }

    #[test]
    fn write_version_specific_object() {
        let mut drawing = Drawing::new();
        drawing.add_object(Object {
            common: Default::default(),
            specific: ObjectType::AcadProxyObject(Default::default()),
        });

        // ACAD_PROXY_OBJECT not supported in R14 and below
        drawing.header.version = AcadVersion::R14;
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "OBJECTS"),
                CodePair::new_str(0, "ENDSEC"),
            ],
        );

        // but it is in R2000 and above
        drawing.header.version = AcadVersion::R2000;
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "SECTION"),
                CodePair::new_str(2, "OBJECTS"),
                CodePair::new_str(0, "ACAD_PROXY_OBJECT"),
            ],
        );
    }

    #[test]
    fn read_extension_data() {
        let obj = read_object(
            "IDBUFFER",
            vec![
                CodePair::new_str(102, "{IXMILIA"),
                CodePair::new_str(1, "some string"),
                CodePair::new_str(102, "}"),
            ],
        );
        assert_eq!(1, obj.common.extension_data_groups.len());
        let group = &obj.common.extension_data_groups[0];
        assert_eq!("IXMILIA", group.application_name);
        match group.items[0] {
            ExtensionGroupItem::CodePair(ref p) => {
                assert_eq!(&CodePair::new_str(1, "some string"), p)
            }
            _ => panic!("expected a code pair"),
        }
    }

    #[test]
    fn write_extension_data() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14;
        drawing.add_object(Object {
            common: ObjectCommon {
                extension_data_groups: vec![ExtensionGroup {
                    application_name: String::from("IXMILIA"),
                    items: vec![ExtensionGroupItem::CodePair(CodePair::new_str(
                        1,
                        "some string",
                    ))],
                }],
                ..Default::default()
            },
            specific: ObjectType::IdBuffer(IdBuffer::default()),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(102, "{IXMILIA"),
                CodePair::new_str(1, "some string"),
                CodePair::new_str(102, "}"),
            ],
        );
    }

    #[test]
    fn read_x_data() {
        let obj = read_object(
            "IDBUFFER",
            vec![
                CodePair::new_str(1001, "IXMILIA"),
                CodePair::new_str(1000, "some string"),
            ],
        );
        assert_eq!(1, obj.common.x_data.len());
        let x = &obj.common.x_data[0];
        assert_eq!("IXMILIA", x.application_name);
        match x.items[0] {
            XDataItem::Str(ref s) => assert_eq!("some string", s),
            _ => panic!("expected a string"),
        }
    }

    #[test]
    fn write_x_data() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2000;
        drawing.add_object(Object {
            common: ObjectCommon {
                x_data: vec![XData {
                    application_name: String::from("IXMILIA"),
                    items: vec![XDataItem::Real(1.1)],
                }],
                ..Default::default()
            },
            specific: ObjectType::IdBuffer(IdBuffer::default()),
        });
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(1001, "IXMILIA"),
                CodePair::new_f64(1040, 1.1),
                CodePair::new_str(0, "ENDSEC"), // xdata is written after all the object's other code pairs
            ],
        );
    }

    #[test]
    fn read_xrecord_with_extension_data() {
        let obj = read_object(
            "XRECORD",
            vec![
                CodePair::new_str(102, "{ACAD_REACTORS"),
                CodePair::new_str(330, "111"),
                CodePair::new_str(102, "}"),
                CodePair::new_str(330, "123"),
                CodePair::new_str(100, "AcDbXrecord"),
                CodePair::new_str(102, "VTR_0.000_0.000_1.000_1.000_VISUALSTYLE"),
                CodePair::new_str(340, "2F"),
            ],
        );
        assert_eq!(1, obj.common.extension_data_groups.len());
        assert_eq!(
            "ACAD_REACTORS",
            obj.common.extension_data_groups[0].application_name
        );
        match obj.specific {
            ObjectType::XRecordObject(ref xr) => {
                assert_eq!(2, xr.data_pairs.len());
                assert_eq!(102, xr.data_pairs[0].code);
                assert_eq!(
                    "VTR_0.000_0.000_1.000_1.000_VISUALSTYLE",
                    xr.data_pairs[0].value.to_string()
                );
            }
            _ => panic!("expected a xrecord object"),
        }
    }

    #[test]
    fn read_all_types() {
        for (type_string, expected_type, _) in all_types::all_object_types() {
            println!("parsing {}", type_string);
            let obj = read_object(
                type_string,
                vec![
                    CodePair::new_str(102, "{IXMILIA"), // read extension data
                    CodePair::new_str(1, "some string"),
                    CodePair::new_str(102, "}"),
                    CodePair::new_str(1001, "IXMILIA"), // read x data
                    CodePair::new_f64(1040, 1.1),
                ],
            );

            // validate specific
            match (&expected_type, &obj.specific) {
                (ObjectType::LayerIndex(a), ObjectType::LayerIndex(b)) => {
                    // LayerIndex has a timestamp that will obviously differ; the remaining fields must be checked manually
                    assert_eq!(a.layer_names, b.layer_names);
                    assert_eq!(a.__id_buffers_handle, b.__id_buffers_handle);
                    assert_eq!(a.id_buffer_counts, b.id_buffer_counts);
                }
                (&ObjectType::SpatialIndex(_), &ObjectType::SpatialIndex(_)) => {
                    // SpatialIndex has a timestamp that will obviously differ; there are no other fields
                }
                _ => assert_eq!(expected_type, obj.specific),
            }

            // validate extension data
            assert_eq!(1, obj.common.extension_data_groups.len());
            assert_eq!(
                "IXMILIA",
                obj.common.extension_data_groups[0].application_name
            );
            assert_eq!(1, obj.common.extension_data_groups[0].items.len());
            assert_eq!(
                ExtensionGroupItem::CodePair(CodePair::new_str(1, "some string")),
                obj.common.extension_data_groups[0].items[0]
            );

            // validate x data
            assert_eq!(1, obj.common.x_data.len());
            assert_eq!("IXMILIA", obj.common.x_data[0].application_name);
            assert_eq!(1, obj.common.x_data[0].items.len());
            assert_eq!(XDataItem::Real(1.1), obj.common.x_data[0].items[0]);
        }
    }

    #[test]
    fn write_all_types() {
        for (type_string, expected_type, max_version) in all_types::all_object_types() {
            println!("writing {}", type_string);
            let mut common = ObjectCommon::default();
            common.extension_data_groups.push(ExtensionGroup {
                application_name: String::from("IXMILIA"),
                items: vec![ExtensionGroupItem::CodePair(CodePair::new_str(
                    1,
                    "some string",
                ))],
            });
            common.x_data.push(XData {
                application_name: String::from("IXMILIA"),
                items: vec![XDataItem::Real(1.1)],
            });
            let mut drawing = Drawing::new();
            drawing.header.version = max_version;
            drawing.add_object(Object {
                common,
                specific: expected_type,
            });
            assert_contains_pairs(&drawing, vec![CodePair::new_str(0, type_string)]);
            if max_version >= AcadVersion::R14 {
                // only written on R14+
                assert_contains_pairs(
                    &drawing,
                    vec![
                        CodePair::new_str(102, "{IXMILIA"),
                        CodePair::new_str(1, "some string"),
                        CodePair::new_str(102, "}"),
                    ],
                );
            }
            if max_version >= AcadVersion::R2000 {
                // only written on R2000+
                assert_contains_pairs(
                    &drawing,
                    vec![
                        CodePair::new_str(1001, "IXMILIA"),
                        CodePair::new_f64(1040, 1.1),
                    ],
                );
            }
        }
    }
}
