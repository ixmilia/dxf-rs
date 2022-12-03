use crate::{CodePair, Color, DxfResult};

use crate::code_pair_put_back::CodePairPutBack;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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
    pub(crate) fn read(iter: &mut CodePairPutBack) -> DxfResult<Option<SectionGeometrySettings>> {
        // check the first pair; only code 90 can start one of these
        match iter.next() {
            Some(Ok(pair @ CodePair { code: 90, .. })) => {
                iter.put_back(Ok(pair));
            }
            Some(Ok(pair)) => {
                iter.put_back(Ok(pair));
                return Ok(None);
            }
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
                1 => {
                    gs.plot_style_name = pair.assert_string()?;
                }
                2 => {
                    gs.hatch_pattern_name = pair.assert_string()?;
                }
                3 => {
                    break;
                } // done reading; value should be "SectionGeometrySettingsEnd" but it doesn't really matter
                6 => {
                    gs.line_type_name = pair.assert_string()?;
                }
                8 => {
                    gs.layer_name = pair.assert_string()?;
                }
                40 => {
                    gs.line_type_scale = pair.assert_f64()?;
                }
                41 => {
                    gs.hatch_angle = pair.assert_f64()?;
                }
                42 => {
                    gs.hatch_scale = pair.assert_f64()?;
                }
                43 => {
                    gs.hatch_spacing = pair.assert_f64()?;
                }
                63 => {
                    gs.color = Color::from_raw_value(pair.assert_i16()?);
                }
                70 => {
                    gs.face_transparency = pair.assert_i16()?;
                }
                71 => {
                    gs.edge_transparency = pair.assert_i16()?;
                }
                72 => {
                    gs.hatch_pattern_type = pair.assert_i16()?;
                }
                90 => {
                    gs.section_type = pair.assert_i32()?;
                }
                91 => {
                    gs.geometry_count = pair.assert_i32()?;
                }
                92 => {
                    gs.bit_flags = pair.assert_i32()?;
                }
                370 => {
                    gs.line_weight = pair.assert_i16()?;
                }
                _ => {
                    // unexpected end; put the pair back and return what we have
                    iter.put_back(Ok(pair));
                    return Ok(Some(gs));
                }
            }
        }

        Ok(Some(gs))
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>) {
        pairs.push(CodePair::new_i32(90, self.section_type));
        pairs.push(CodePair::new_i32(91, self.geometry_count));
        pairs.push(CodePair::new_i32(92, self.bit_flags));
        pairs.push(CodePair::new_i16(63, self.color.raw_value()));
        pairs.push(CodePair::new_string(8, &self.layer_name));
        pairs.push(CodePair::new_string(6, &self.line_type_name));
        pairs.push(CodePair::new_f64(40, self.line_type_scale));
        pairs.push(CodePair::new_string(1, &self.plot_style_name));
        pairs.push(CodePair::new_i16(370, self.line_weight));
        pairs.push(CodePair::new_i16(70, self.face_transparency));
        pairs.push(CodePair::new_i16(71, self.edge_transparency));
        pairs.push(CodePair::new_i16(72, self.hatch_pattern_type));
        pairs.push(CodePair::new_string(2, &self.hatch_pattern_name));
        pairs.push(CodePair::new_f64(41, self.hatch_angle));
        pairs.push(CodePair::new_f64(42, self.hatch_scale));
        pairs.push(CodePair::new_f64(43, self.hatch_spacing));
        pairs.push(CodePair::new_str(3, "SectionGeometrySettingsEnd"));
    }
}
