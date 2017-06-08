// Copyright (c) IxMilia.  All Rights Reserved.  Licensed under the Apache License, Version 2.0.  See License.txt in the project root for license information.

use std::io::Write;

use ::{
    CodePair,
    Color,
    DxfResult,
};

use ::code_pair_writer::CodePairWriter;

use itertools::PutBack;

#[derive(Clone, Debug, PartialEq)]
pub struct SectionGeometrySettings {
    pub section_type: i32,
    pub geometry_count: i32,
    pub bit_flags: i32,
    pub color: Color,
    pub layer_name: String,
    pub line_type_name: String,
    pub line_type_scale: f64,
    pub plot_style_name: String,
    pub line_weight: i16,
    pub face_transparency: i16,
    pub edge_transparency: i16,
    pub hatch_pattern_type: i16,
    pub hatch_pattern_name: String,
    pub hatch_angle: f64,
    pub hatch_scale: f64,
    pub hatch_spacing: f64,
}

impl Default for SectionGeometrySettings {
    fn default() -> Self {
        SectionGeometrySettings {
            section_type: 0,
            geometry_count: 0,
            bit_flags: 0,
            color: Color::by_block(),
            layer_name: String::new(),
            line_type_name: String::new(),
            line_type_scale: 1.0,
            plot_style_name: String::new(),
            line_weight: 0,
            face_transparency: 0,
            edge_transparency: 0,
            hatch_pattern_type: 0,
            hatch_pattern_name: String::new(),
            hatch_angle: 0.0,
            hatch_scale: 1.0,
            hatch_spacing: 0.0,
        }
    }
}

// internal visibility only
impl SectionGeometrySettings {
    pub(crate) fn read<I>(iter: &mut PutBack<I>) -> DxfResult<Option<SectionGeometrySettings>>
        where I: Iterator<Item = DxfResult<CodePair>> {

        // check the first pair; only code 90 can start one of these
        match iter.next() {
            Some(Ok(pair @ CodePair { code: 90, .. })) => {
                iter.put_back(Ok(pair));
            },
            Some(Ok(pair)) => {
                iter.put_back(Ok(pair));
                return Ok(None);
            },
            Some(Err(e)) => return Err(e),
            None => return Ok(None),
        }

        let mut gs = SectionGeometrySettings::default();
        loop {
            let pair = match iter.next() {
                Some(Ok(pair)) => pair,
                Some(Err(e)) => return Err(e),
                None => return Ok(Some(gs)),
            };

            match pair.code {
                1 => { gs.plot_style_name = pair.value.assert_string()?; },
                2 => { gs.hatch_pattern_name = pair.value.assert_string()?; },
                3 => { break; }, // done reading; value should be "SectionGeometrySettingsEnd" but it doesn't really matter
                6 => { gs.line_type_name = pair.value.assert_string()?; },
                8 => { gs.layer_name = pair.value.assert_string()?; },
                40 => { gs.line_type_scale = pair.value.assert_f64()?; },
                41 => { gs.hatch_angle = pair.value.assert_f64()?; },
                42 => { gs.hatch_scale = pair.value.assert_f64()?; },
                43 => { gs.hatch_spacing = pair.value.assert_f64()?; },
                63 => { gs.color = Color::from_raw_value(pair.value.assert_i16()?); },
                70 => { gs.face_transparency = pair.value.assert_i16()?; },
                71 => { gs.edge_transparency = pair.value.assert_i16()?; },
                72 => { gs.hatch_pattern_type = pair.value.assert_i16()?; },
                90 => { gs.section_type = pair.value.assert_i32()?; },
                91 => { gs.geometry_count = pair.value.assert_i32()?; },
                92 => { gs.bit_flags = pair.value.assert_i32()?; },
                370 => { gs.line_weight = pair.value.assert_i16()?; },
                _ => {
                    // unexpected end; put the pair back and return what we have
                    iter.put_back(Ok(pair));
                    return Ok(Some(gs));
                },
            }
        }

        return Ok(Some(gs));
    }
    pub(crate) fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
        where T: Write {

        writer.write_code_pair(&CodePair::new_i32(90, self.section_type))?;
        writer.write_code_pair(&CodePair::new_i32(91, self.geometry_count))?;
        writer.write_code_pair(&CodePair::new_i32(92, self.bit_flags))?;
        writer.write_code_pair(&CodePair::new_i16(63, self.color.get_raw_value()))?;
        writer.write_code_pair(&CodePair::new_string(8, &self.layer_name))?;
        writer.write_code_pair(&CodePair::new_string(6, &self.line_type_name))?;
        writer.write_code_pair(&CodePair::new_f64(40, self.line_type_scale))?;
        writer.write_code_pair(&CodePair::new_string(1, &self.plot_style_name))?;
        writer.write_code_pair(&CodePair::new_i16(370, self.line_weight))?;
        writer.write_code_pair(&CodePair::new_i16(70, self.face_transparency))?;
        writer.write_code_pair(&CodePair::new_i16(71, self.edge_transparency))?;
        writer.write_code_pair(&CodePair::new_i16(72, self.hatch_pattern_type))?;
        writer.write_code_pair(&CodePair::new_string(2, &self.hatch_pattern_name))?;
        writer.write_code_pair(&CodePair::new_f64(41, self.hatch_angle))?;
        writer.write_code_pair(&CodePair::new_f64(42, self.hatch_scale))?;
        writer.write_code_pair(&CodePair::new_f64(43, self.hatch_spacing))?;
        writer.write_code_pair(&CodePair::new_str(3, "SectionGeometrySettingsEnd"))?;
        Ok(())
    }
}
