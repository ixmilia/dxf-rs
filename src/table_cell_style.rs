use crate::{CodePair, Color, DxfResult};

use crate::code_pair_put_back::CodePairPutBack;
use crate::helper_functions::*;

/// Defines a style for a table's cell.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct TableCellStyle {
    pub name: String,
    pub text_height: f64,
    pub cell_alignment: i16,
    pub text_color: Color,
    pub cell_fill_color: Color,
    pub is_background_color_enabled: bool,
    pub cell_data_type: i32,
    pub cell_unit_type: i32,
    pub border_lineweight_1: i16,
    pub border_lineweight_2: i16,
    pub border_lineweight_3: i16,
    pub border_lineweight_4: i16,
    pub border_lineweight_5: i16,
    pub border_lineweight_6: i16,
    pub is_border_1_visible: bool,
    pub is_border_2_visible: bool,
    pub is_border_3_visible: bool,
    pub is_border_4_visible: bool,
    pub is_border_5_visible: bool,
    pub is_border_6_visible: bool,
    pub border_1_color: Color,
    pub border_2_color: Color,
    pub border_3_color: Color,
    pub border_4_color: Color,
    pub border_5_color: Color,
    pub border_6_color: Color,
}

// internal visibility only
impl TableCellStyle {
    #[allow(clippy::cognitive_complexity)]
    pub(crate) fn read(iter: &mut CodePairPutBack) -> DxfResult<Option<TableCellStyle>> {
        let mut seen_name = false;
        let mut style = TableCellStyle::default();
        loop {
            let pair = match iter.next() {
                Some(Ok(CodePair { code: 0, .. })) => return Ok(Some(style)),
                Some(Ok(pair)) => pair,
                Some(Err(e)) => return Err(e),
                None => return Ok(Some(style)),
            };
            match pair.code {
                7 => {
                    if seen_name {
                        // found another cell style; put the pair back and return what we have
                        iter.put_back(Ok(pair));
                        return Ok(Some(style));
                    } else {
                        style.name = pair.assert_string()?;
                        seen_name = true;
                    }
                }
                62 => {
                    style.text_color = Color::from_raw_value(pair.assert_i16()?);
                }
                63 => {
                    style.cell_fill_color = Color::from_raw_value(pair.assert_i16()?);
                }
                64 => {
                    style.border_1_color = Color::from_raw_value(pair.assert_i16()?);
                }
                65 => {
                    style.border_2_color = Color::from_raw_value(pair.assert_i16()?);
                }
                66 => {
                    style.border_3_color = Color::from_raw_value(pair.assert_i16()?);
                }
                67 => {
                    style.border_4_color = Color::from_raw_value(pair.assert_i16()?);
                }
                68 => {
                    style.border_5_color = Color::from_raw_value(pair.assert_i16()?);
                }
                69 => {
                    style.border_6_color = Color::from_raw_value(pair.assert_i16()?);
                }
                90 => {
                    style.cell_data_type = pair.assert_i32()?;
                }
                91 => {
                    style.cell_unit_type = pair.assert_i32()?;
                }
                140 => {
                    style.text_height = pair.assert_f64()?;
                }
                170 => {
                    style.cell_alignment = pair.assert_i16()?;
                }
                274 => {
                    style.border_lineweight_1 = pair.assert_i16()?;
                }
                275 => {
                    style.border_lineweight_2 = pair.assert_i16()?;
                }
                276 => {
                    style.border_lineweight_3 = pair.assert_i16()?;
                }
                277 => {
                    style.border_lineweight_4 = pair.assert_i16()?;
                }
                278 => {
                    style.border_lineweight_5 = pair.assert_i16()?;
                }
                279 => {
                    style.border_lineweight_6 = pair.assert_i16()?;
                }
                283 => {
                    style.is_background_color_enabled = as_bool(pair.assert_i16()?);
                }
                284 => {
                    style.is_border_1_visible = as_bool(pair.assert_i16()?);
                }
                285 => {
                    style.is_border_2_visible = as_bool(pair.assert_i16()?);
                }
                286 => {
                    style.is_border_3_visible = as_bool(pair.assert_i16()?);
                }
                287 => {
                    style.is_border_4_visible = as_bool(pair.assert_i16()?);
                }
                288 => {
                    style.is_border_5_visible = as_bool(pair.assert_i16()?);
                }
                289 => {
                    style.is_border_6_visible = as_bool(pair.assert_i16()?);
                }
                _ => {
                    // unexpected pair; put the pair back and return what we have
                    iter.put_back(Ok(pair));
                    return Ok(Some(style));
                }
            }
        }
    }
}
