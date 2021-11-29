use crate::{CodePair, DxfResult, Handle, SectionGeometrySettings};

use crate::code_pair_put_back::CodePairPutBack;
use crate::helper_functions::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct SectionTypeSettings {
    pub section_type: i32,
    pub is_generation_option: bool,
    pub source_object_handles: Vec<Handle>,
    pub destination_object_handle: Handle,
    pub destination_file_name: String,
    pub geometry_settings: Vec<SectionGeometrySettings>,
}

impl Default for SectionTypeSettings {
    fn default() -> Self {
        SectionTypeSettings {
            section_type: 0,
            is_generation_option: false,
            source_object_handles: vec![],
            destination_object_handle: Handle::empty(),
            destination_file_name: String::new(),
            geometry_settings: vec![],
        }
    }
}

// internal visibility only
impl SectionTypeSettings {
    pub(crate) fn read(iter: &mut CodePairPutBack) -> DxfResult<Option<SectionTypeSettings>> {
        // check the first pair and only continue if it's not 0
        match iter.next() {
            Some(Ok(pair @ CodePair { code: 0, .. })) => {
                iter.put_back(Ok(pair));
                return Ok(None);
            }
            Some(Ok(pair)) => {
                iter.put_back(Ok(pair));
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(None),
        }

        let mut ss = SectionTypeSettings::default();
        loop {
            let pair = match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    iter.put_back(Ok(pair));
                    return Ok(Some(ss));
                }
                Some(Ok(pair)) => pair,
                Some(Err(e)) => return Err(e),
                None => return Ok(Some(ss)),
            };

            match pair.code {
                1 => {
                    ss.destination_file_name = pair.assert_string()?;
                }
                2 => {
                    // value should be "SectionGeometrySettings", but it doesn't really matter
                    while let Some(gs) = SectionGeometrySettings::read(iter)? {
                        ss.geometry_settings.push(gs);
                    }
                }
                3 => (), // value should be "SectionTypeSettingsEnd", but it doesn't really matter
                90 => {
                    ss.section_type = pair.assert_i32()?;
                }
                91 => {
                    ss.is_generation_option = as_bool(pair.assert_i32()? as i16);
                }
                92 => (), // source objects count; we just read as many as we're given
                93 => (), // generation settings count; we just read as many as we're given
                330 => {
                    ss.source_object_handles.push(pair.as_handle()?);
                }
                331 => {
                    ss.destination_object_handle = pair.as_handle()?;
                }
                _ => {
                    // unexpected end; put the pair back and return what we have
                    iter.put_back(Ok(pair));
                    return Ok(Some(ss));
                }
            }
        }
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>) {
        pairs.push(CodePair::new_str(1, "SectionTypeSettings"));
        pairs.push(CodePair::new_i32(90, self.section_type));
        pairs.push(CodePair::new_i32(
            91,
            i32::from(as_i16(self.is_generation_option)),
        ));
        pairs.push(CodePair::new_i32(
            92,
            self.source_object_handles.len() as i32,
        ));
        for handle in &self.source_object_handles {
            pairs.push(CodePair::new_string(330, &handle.as_string()));
        }
        pairs.push(CodePair::new_string(
            331,
            &self.destination_object_handle.as_string(),
        ));
        pairs.push(CodePair::new_string(1, &self.destination_file_name));
        pairs.push(CodePair::new_i32(93, self.geometry_settings.len() as i32));
        pairs.push(CodePair::new_str(2, "SectionGeometrySettings"));
        for geometry_settings in &self.geometry_settings {
            geometry_settings.add_code_pairs(pairs);
        }
        pairs.push(CodePair::new_str(3, "SectionTypeSettingsEnd"));
    }
}
