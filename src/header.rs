// other implementation is in `generated/header.rs`

use crate::code_pair_put_back::CodePairPutBack;
use crate::enums::*;
use crate::helper_functions::*;
use crate::{CodePair, DxfError, DxfResult};

pub use crate::generated::header::*;

impl Header {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.default_text_height, 0.2);
        ensure_positive_or_default(&mut self.trace_width, 0.05);
        default_if_empty(&mut self.text_style, "STANDARD");
        default_if_empty(&mut self.current_layer, "0");
        default_if_empty(&mut self.current_entity_line_type, "BYLAYER");
        default_if_empty(&mut self.dimension_style_name, "STANDARD");
        default_if_empty(&mut self.file_name, ".");
    }
    pub(crate) fn read(iter: &mut CodePairPutBack) -> DxfResult<Header> {
        let mut header = Header::default();
        loop {
            match iter.next() {
                Some(Ok(pair)) => {
                    match pair.code {
                        0 => {
                            iter.put_back(Ok(pair));
                            break;
                        }
                        9 => {
                            let last_header_variable = pair.assert_string()?;
                            loop {
                                match iter.next() {
                                    Some(Ok(pair)) => {
                                        if pair.code == 0 || pair.code == 9 {
                                            // ENDSEC or a new header variable
                                            iter.put_back(Ok(pair));
                                            break;
                                        } else {
                                            header
                                                .set_header_value(&last_header_variable, &pair)?;
                                            if last_header_variable == "$ACADVER"
                                                && header.version >= AcadVersion::R2007
                                            {
                                                iter.read_as_utf8();
                                            }
                                        }
                                    }
                                    Some(Err(e)) => return Err(e),
                                    None => break,
                                }
                            }
                        }
                        _ => return Err(DxfError::UnexpectedCodePair(pair, String::from(""))),
                    }
                }
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        Ok(header)
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>) {
        pairs.push(CodePair::new_str(0, "SECTION"));
        pairs.push(CodePair::new_str(2, "HEADER"));
        self.add_code_pairs_internal(pairs);
        pairs.push(CodePair::new_str(0, "ENDSEC"));
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::*;
    use crate::enums::*;
    use crate::helper_functions::tests::*;
    use crate::*;
    use float_cmp::approx_eq;
    use std::time::Duration;

    #[test]
    fn empty_header() {
        let _file = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "HEADER"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
    }

    #[test]
    fn specific_header_values() {
        let drawing = from_section_pairs(
            "HEADER",
            vec![
                CodePair::new_str(9, "$ACADMAINTVER"),
                CodePair::new_i16(70, 16),
                CodePair::new_str(9, "$ACADVER"),
                CodePair::new_str(1, "AC1012"),
                CodePair::new_str(9, "$ANGBASE"),
                CodePair::new_f64(50, 55.0),
                CodePair::new_str(9, "$ANGDIR"),
                CodePair::new_i16(70, 1),
                CodePair::new_str(9, "$ATTMODE"),
                CodePair::new_i16(70, 1),
                CodePair::new_str(9, "$AUNITS"),
                CodePair::new_i16(70, 3),
                CodePair::new_str(9, "$AUPREC"),
                CodePair::new_i16(70, 7),
                CodePair::new_str(9, "$CLAYER"),
                CodePair::new_str(8, "<current layer>"),
                CodePair::new_str(9, "$LUNITS"),
                CodePair::new_i16(70, 6),
                CodePair::new_str(9, "$LUPREC"),
                CodePair::new_i16(70, 7),
            ],
        );
        assert_eq!(16, drawing.header.maintenance_version);
        assert_eq!(AcadVersion::R13, drawing.header.version);
        assert!(approx_eq!(f64, 55.0, drawing.header.angle_zero_direction));
        assert_eq!(AngleDirection::Clockwise, drawing.header.angle_direction);
        assert_eq!(
            AttributeVisibility::Normal,
            drawing.header.attribute_visibility
        );
        assert_eq!(AngleFormat::Radians, drawing.header.angle_unit_format);
        assert_eq!(7, drawing.header.angle_unit_precision);
        assert_eq!("<current layer>", drawing.header.current_layer);
        assert_eq!(UnitFormat::Architectural, drawing.header.unit_format);
        assert_eq!(7, drawing.header.unit_precision);
    }

    #[test]
    fn read_alternate_maintenance_version() {
        let drawing = from_section_pairs(
            "HEADER",
            vec![
                CodePair::new_str(9, "$ACADMAINTVER"),
                CodePair::new_i32(90, 4242),
            ],
        );
        assert_eq!(4242, drawing.header.maintenance_version);
    }

    #[test]
    fn maintenance_version_is_only_written_with_code_70() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14; // $ACADMAINTVER is only written for R14 and later
        drawing.header.maintenance_version = 4242;
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$ACADMAINTVER"),
                CodePair::new_i16(70, 4242),
            ],
        );
        assert_not_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$ACADMAINTVER"),
                CodePair::new_i32(90, 4242),
            ],
        );
    }

    #[test]
    fn read_alternate_version() {
        let drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$ACADVER"),
                CodePair::new_str(1, "15.05"),
            ],
        );
        assert_eq!(AcadVersion::R2000, drawing.header.version);
    }

    #[test]
    fn read_invalid_version() {
        let drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$ACADVER"),
                CodePair::new_str(1, "AC3.14159"),
            ],
        );
        assert_eq!(AcadVersion::R12, drawing.header.version);
    }

    #[test]
    fn read_multi_value_variable() {
        let drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$EXTMIN"),
                CodePair::new_f64(10, 1.1),
                CodePair::new_f64(20, 2.2),
                CodePair::new_f64(30, 3.3),
            ],
        );
        assert_eq!(
            Point::new(1.1, 2.2, 3.3),
            drawing.header.minimum_drawing_extents
        )
    }

    #[test]
    fn write_multiple_value_variable() {
        let mut drawing = Drawing::new();
        drawing.header.minimum_drawing_extents = Point::new(1.1, 2.2, 3.3);
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$EXTMIN"),
                CodePair::new_f64(10, 1.1),
                CodePair::new_f64(20, 2.2),
                CodePair::new_f64(30, 3.3),
            ],
        );
    }

    #[test]
    fn normalize_header() {
        let mut header = Header {
            default_text_height: -1.0,               // $TEXTSIZE; normalized to 0.2,
            trace_width: 0.0,                        // $TRACEWID; normalized to 0.05
            text_style: String::new(),               // $TEXTSTYLE; normalized to "STANDARD"
            current_layer: String::new(),            // $CLAYER; normalized to "0"
            current_entity_line_type: String::new(), // $CELTYPE; normalized to "BYLAYER"
            dimension_style_name: String::new(),     // $DIMSTYLE; normalized to "STANDARD"
            file_name: String::new(),                // $MENU; normalized to "."
            ..Default::default()
        };
        header.normalize();
        assert!(approx_eq!(f64, 0.2, header.default_text_height));
        assert!(approx_eq!(f64, 0.05, header.trace_width));
        assert_eq!("STANDARD", header.text_style);
        assert_eq!("0", header.current_layer);
        assert_eq!("BYLAYER", header.current_entity_line_type);
        assert_eq!("STANDARD", header.dimension_style_name);
        assert_eq!(".", header.file_name);
    }

    #[test]
    fn read_header_flags() {
        let drawing = from_section(
            "HEADER",
            vec![CodePair::new_str(9, "$OSMODE"), CodePair::new_i16(70, 12)],
        );
        assert!(!drawing.header.end_point_snap());
        assert!(!drawing.header.mid_point_snap());
        assert!(drawing.header.center_snap());
        assert!(drawing.header.node_snap());
        assert!(!drawing.header.quadrant_snap());
        assert!(!drawing.header.intersection_snap());
        assert!(!drawing.header.insertion_snap());
        assert!(!drawing.header.perpendicular_snap());
        assert!(!drawing.header.tangent_snap());
        assert!(!drawing.header.nearest_snap());
        assert!(!drawing.header.apparent_intersection_snap());
        assert!(!drawing.header.extension_snap());
        assert!(!drawing.header.parallel_snap());
    }

    #[test]
    fn write_header_flags() {
        let mut drawing = Drawing::new();
        drawing.header.set_end_point_snap(false);
        drawing.header.set_mid_point_snap(false);
        drawing.header.set_center_snap(true);
        drawing.header.set_node_snap(true);
        drawing.header.set_quadrant_snap(false);
        drawing.header.set_intersection_snap(false);
        drawing.header.set_insertion_snap(false);
        drawing.header.set_perpendicular_snap(false);
        drawing.header.set_tangent_snap(false);
        drawing.header.set_nearest_snap(false);
        drawing.header.set_apparent_intersection_snap(false);
        drawing.header.set_extension_snap(false);
        drawing.header.set_parallel_snap(false);
        assert_contains_pairs(
            &drawing,
            vec![CodePair::new_str(9, "$OSMODE"), CodePair::new_i16(70, 12)],
        );
    }

    #[test]
    fn read_variable_with_different_codes() {
        // read $CMLSTYLE as code 7
        let drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$CMLSTYLE"),
                CodePair::new_str(7, "cml-style-7"),
            ],
        );
        assert_eq!("cml-style-7", drawing.header.current_multiline_style);

        // read $CMLSTYLE as code 2
        let drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$CMLSTYLE"),
                CodePair::new_str(2, "cml-style-2"),
            ],
        );
        assert_eq!("cml-style-2", drawing.header.current_multiline_style);
    }

    #[test]
    fn write_variable_with_different_codes() {
        // R13 writes $CMLSTYLE as a code 7
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R13;
        drawing.header.current_multiline_style = String::from("cml-style-7");
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$CMLSTYLE"),
                CodePair::new_str(7, "cml-style-7"),
            ],
        );

        // R14+ writes $CMLSTYLE as a code 2
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14;
        drawing.header.current_multiline_style = String::from("cml-style-2");
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$CMLSTYLE"),
                CodePair::new_str(2, "cml-style-2"),
            ],
        );
    }

    #[test]
    fn read_drawing_edit_duration() {
        let drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$TDINDWG"),
                CodePair::new_f64(40, 100.0),
            ],
        );
        assert_eq!(Duration::from_secs(100), drawing.header.time_in_drawing);
    }

    #[test]
    fn write_proper_handseed_on_new_file() {
        let mut drawing = Drawing::new();
        drawing.add_entity(Entity::new(EntityType::Line(Line::new(
            Point::origin(),
            Point::origin(),
        ))));
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$HANDSEED"),
                CodePair::new_str(5, "11"),
            ],
        );
    }

    #[test]
    fn write_proper_handseed_on_read_file() {
        let mut drawing = from_section(
            "HEADER",
            vec![
                CodePair::new_str(9, "$HANDSEED"),
                CodePair::new_str(5, "11"),
            ],
        );
        drawing.add_entity(Entity::new(EntityType::Line(Line::new(
            Point::origin(),
            Point::origin(),
        ))));
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(9, "$HANDSEED"),
                CodePair::new_str(5, "15"),
            ],
        );
    }

    #[test]
    fn do_not_write_suppressed_variables() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2004;
        assert_contains_pairs(
            &drawing,
            vec![CodePair::new_str(9, "$HIDETEXT"), CodePair::new_i16(280, 0)],
        );
        assert_not_contains_pairs(
            &drawing,
            vec![CodePair::new_str(9, "$HIDETEXT"), CodePair::new_i16(290, 0)],
        );
    }
}
