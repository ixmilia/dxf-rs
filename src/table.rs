use crate::helper_functions::*;
use crate::tables::*;
use crate::Color;

//------------------------------------------------------------------------------
//                                                                         Layer
//------------------------------------------------------------------------------
impl Layer {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        default_if_empty(&mut self.line_type_name, "CONTINUOUS");
        match self.color.raw_value() {
            0 | 256 => self.color = Color::from_raw_value(7), // BYLAYER and BYBLOCK aren't valid layer colors
            _ => (),
        }
    }
}

//------------------------------------------------------------------------------
//                                                                         Style
//------------------------------------------------------------------------------
impl Style {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.text_height, 0.0);
        ensure_positive_or_default(&mut self.width_factor, 1.0);
    }
}

//------------------------------------------------------------------------------
//                                                                          View
//------------------------------------------------------------------------------
impl View {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.view_height, 1.0);
        ensure_positive_or_default(&mut self.view_width, 1.0);
        ensure_positive_or_default(&mut self.lens_length, 1.0);
    }
}

//------------------------------------------------------------------------------
//                                                                      ViewPort
//------------------------------------------------------------------------------
impl ViewPort {
    /// Ensure all values are valid.
    pub fn normalize(&mut self) {
        ensure_positive_or_default(&mut self.snap_spacing.x, 1.0);
        ensure_positive_or_default(&mut self.snap_spacing.y, 1.0);
        ensure_positive_or_default(&mut self.snap_spacing.z, 1.0);
        ensure_positive_or_default(&mut self.grid_spacing.x, 1.0);
        ensure_positive_or_default(&mut self.grid_spacing.y, 1.0);
        ensure_positive_or_default(&mut self.grid_spacing.z, 1.0);
        ensure_positive_or_default(&mut self.view_height, 1.0);
        ensure_positive_or_default(&mut self.view_port_aspect_ratio, 1.0);
        ensure_positive_or_default(&mut self.lens_length, 50.0);
        ensure_positive_or_default_i16(&mut self.ucs_icon, 3);
        ensure_positive_or_default_i32(&mut self.circle_sides, 1000);
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::*;
    use crate::enums::*;
    use crate::helper_functions::tests::*;
    use crate::objects::*;
    use crate::tables::*;
    use crate::*;
    use float_cmp::approx_eq;

    fn read_table(table_name: &str, value_pairs: Vec<CodePair>) -> Drawing {
        let mut pairs = vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "TABLES"),
            CodePair::new_str(0, "TABLE"),
            CodePair::new_str(2, table_name),
            CodePair::new_str(100, "AcDbSymbolTable"),
            CodePair::new_i16(70, 0),
        ];
        for pair in value_pairs {
            pairs.push(pair);
        }
        pairs.push(CodePair::new_str(0, "ENDTAB"));
        pairs.push(CodePair::new_str(0, "ENDSEC"));
        pairs.push(CodePair::new_str(0, "EOF"));
        drawing_from_pairs(pairs)
    }

    #[test]
    fn read_unsupported_table() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "TABLES"),
            CodePair::new_str(0, "TABLE"),
            CodePair::new_str(2, "UNSUPPORTED"),
            CodePair::new_str(0, "UNSUPPORTED"),
            CodePair::new_str(2, "unsupported-name"),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "TABLE"),
            CodePair::new_str(2, "LAYER"),
            CodePair::new_str(0, "LAYER"),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(1, drawing.layers().count());
    }

    #[test]
    fn read_single_layer() {
        let drawing = read_table(
            "LAYER",
            vec![
                CodePair::new_str(0, "LAYER"),
                CodePair::new_str(2, "layer-name"),
            ],
        );
        let layers = drawing.layers().collect::<Vec<_>>();
        assert_eq!(1, layers.len());
        assert_eq!("layer-name", layers[0].name);
    }

    #[test]
    fn read_variable_table_items() {
        let drawing = drawing_from_pairs(vec![
            CodePair::new_str(0, "SECTION"),
            CodePair::new_str(2, "TABLES"),
            CodePair::new_str(0, "TABLE"), // no app ids
            CodePair::new_str(2, "APPID"),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "TABLE"), // 1 layer
            CodePair::new_str(2, "LAYER"),
            CodePair::new_str(0, "LAYER"),
            CodePair::new_str(2, "layer-name"),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "TABLE"), // 2 styles
            CodePair::new_str(2, "STYLE"),
            CodePair::new_str(0, "STYLE"),
            CodePair::new_f64(40, 1.1),
            CodePair::new_str(0, "STYLE"),
            CodePair::new_f64(40, 2.2),
            CodePair::new_str(0, "ENDTAB"),
            CodePair::new_str(0, "ENDSEC"),
            CodePair::new_str(0, "EOF"),
        ]);
        assert_eq!(0, drawing.block_records().count()); // not listed in file, but make sure there are still 0
        assert_eq!(0, drawing.app_ids().count());
        let layers = drawing.layers().collect::<Vec<_>>();
        assert_eq!(1, layers.len());
        assert_eq!("layer-name", layers[0].name);
        let styles = drawing.styles().collect::<Vec<_>>();
        assert_eq!(2, styles.len());
        assert!(approx_eq!(f64, 1.1, styles[0].text_height));
        assert!(approx_eq!(f64, 2.2, styles[1].text_height));
    }

    #[test]
    fn read_layer_color_and_layer_is_on() {
        let drawing = read_table(
            "LAYER",
            vec![CodePair::new_str(0, "LAYER"), CodePair::new_i16(62, 5)],
        );
        let layers = drawing.layers().collect::<Vec<_>>();
        let layer = layers[0];
        assert_eq!(Some(5), layer.color.index());
        assert!(layer.is_layer_on);
    }

    #[test]
    fn read_layer_color_and_layer_is_off() {
        let drawing = read_table(
            "LAYER",
            vec![CodePair::new_str(0, "LAYER"), CodePair::new_i16(62, -5)],
        );
        let layers = drawing.layers().collect::<Vec<_>>();
        let layer = layers[0];
        assert_eq!(Some(5), layer.color.index());
        assert!(!layer.is_layer_on);
    }

    #[test]
    fn write_layer() {
        let mut drawing = Drawing::new();
        let layer = Layer {
            name: String::from("layer-name"),
            color: Color::from_index(3),
            ..Default::default()
        };

        drawing.add_layer(layer);
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(0, "LAYER"),
                CodePair::new_str(5, "10"),
                CodePair::new_str(100, "AcDbSymbolTableRecord"),
                CodePair::new_str(100, "AcDbLayerTableRecord"),
                CodePair::new_str(2, "layer-name"),
                CodePair::new_i16(70, 0),
                CodePair::new_i16(62, 3),
                CodePair::new_str(6, "CONTINUOUS"),
            ],
        );
    }

    #[test]
    fn normalize_layer() {
        let mut layer = Layer {
            name: String::from("layer-name"),
            color: Color::by_layer(), // value 256 not valid; normalized to 7
            line_type_name: String::from(""), // empty string not valid; normalized to CONTINUOUS
            ..Default::default()
        };
        layer.normalize();
        assert_eq!(Some(7), layer.color.index());
        assert_eq!("CONTINUOUS", layer.line_type_name);
    }

    #[test]
    fn normalize_view() {
        let mut view = View {
            view_height: 0.0,  // invalid; normalized to 1.0
            view_width: -1.0,  // invalid; normalized to 1.0
            lens_length: 42.0, // valid
            ..Default::default()
        };
        view.normalize();
        assert!(approx_eq!(f64, 1.0, view.view_height));
        assert!(approx_eq!(f64, 1.0, view.view_width));
        assert!(approx_eq!(f64, 42.0, view.lens_length));
    }

    #[test]
    fn read_table_item_with_extended_data() {
        let drawing = read_table(
            "LAYER",
            vec![
                CodePair::new_str(0, "LAYER"),
                CodePair::new_str(102, "{IXMILIA"),
                CodePair::new_str(1, "some string"),
                CodePair::new_str(102, "}"),
            ],
        );
        let layers = drawing.layers().collect::<Vec<_>>();
        let layer = layers[0];
        assert_eq!(1, layer.extension_data_groups.len());
        let group = &layer.extension_data_groups[0];
        assert_eq!("IXMILIA", group.application_name);
        assert_eq!(1, group.items.len());
        match group.items[0] {
            ExtensionGroupItem::CodePair(ref p) => {
                assert_eq!(&CodePair::new_str(1, "some string"), p);
            }
            _ => panic!("expected a code pair"),
        }
    }

    #[test]
    fn write_table_item_with_extended_data() {
        let layer = Layer {
            extension_data_groups: vec![ExtensionGroup {
                application_name: String::from("IXMILIA"),
                items: vec![ExtensionGroupItem::CodePair(CodePair::new_str(
                    1,
                    "some string",
                ))],
            }],
            ..Default::default()
        };
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R14;
        drawing.add_layer(layer);
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
    fn read_table_item_with_x_data() {
        let drawing = read_table(
            "LAYER",
            vec![
                CodePair::new_str(0, "LAYER"),
                CodePair::new_str(1001, "IXMILIA"),
                CodePair::new_f64(1040, 1.1),
            ],
        );
        let layers = drawing.layers().collect::<Vec<_>>();
        let layer = layers[0];
        assert_eq!(1, layer.x_data.len());
        let x = &layer.x_data[0];
        assert_eq!("IXMILIA", x.application_name);
        assert_eq!(1, x.items.len());
        match x.items[0] {
            XDataItem::Real(r) => assert!(approx_eq!(f64, 1.1, r)),
            _ => panic!("expected a code pair"),
        }
    }

    #[test]
    fn write_table_item_with_x_data() {
        let layer = Layer {
            x_data: vec![XData {
                application_name: String::from("IXMILIA"),
                items: vec![XDataItem::Real(1.1)],
            }],
            ..Default::default()
        };
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2000;
        drawing.add_layer(layer);
        assert_contains_pairs(
            &drawing,
            vec![
                CodePair::new_str(1001, "IXMILIA"),
                CodePair::new_f64(1040, 1.1),
            ],
        );
    }

    #[test]
    fn normalize_layers() {
        let mut file = Drawing::new();
        file.clear();
        assert_eq!(0, file.layers().count());
        file.header.current_layer = String::from("current layer");
        file.normalize();
        let layers = file.layers().collect::<Vec<_>>();
        assert_eq!(2, layers.len());
        assert_eq!("0", layers[0].name);
        assert_eq!("current layer", layers[1].name);
    }

    #[test]
    fn normalize_line_types() {
        let mut file = Drawing::new();
        file.clear();
        assert_eq!(0, file.line_types().count());
        file.add_entity(Entity {
            common: EntityCommon {
                line_type_name: String::from("line type"),
                ..Default::default()
            },
            specific: EntityType::Line(Default::default()),
        });
        file.normalize();
        let line_types = file.line_types().collect::<Vec<_>>();
        assert_eq!(4, line_types.len());
        assert_eq!("BYBLOCK", line_types[0].name);
        assert_eq!("BYLAYER", line_types[1].name);
        assert_eq!("CONTINUOUS", line_types[2].name);
        assert_eq!("line type", line_types[3].name);
    }

    #[test]
    fn normalize_text_styles() {
        let mut file = Drawing::new();
        file.clear();
        assert_eq!(0, file.styles().count());
        file.add_entity(Entity::new(EntityType::Attribute(Attribute {
            text_style_name: String::from("text style"),
            ..Default::default()
        })));
        file.normalize();
        let styles = file.styles().collect::<Vec<_>>();
        assert_eq!(3, styles.len());
        assert_eq!("ANNOTATIVE", styles[0].name);
        assert_eq!("STANDARD", styles[1].name);
        assert_eq!("text style", styles[2].name);
    }

    #[test]
    fn normalize_view_ports() {
        let mut file = Drawing::new();
        file.clear();
        assert_eq!(0, file.view_ports().count());
        file.normalize();
        let view_ports = file.view_ports().collect::<Vec<_>>();
        assert_eq!(1, view_ports.len());
        assert_eq!("*ACTIVE", view_ports[0].name);
    }

    #[test]
    fn normalize_views() {
        let mut file = Drawing::new();
        file.clear();
        assert_eq!(0, file.views().count());
        file.add_object(Object::new(ObjectType::PlotSettings(PlotSettings {
            plot_view_name: String::from("some view"),
            ..Default::default()
        })));
        file.normalize();
        let views = file.views().collect::<Vec<_>>();
        assert_eq!(1, views.len());
        assert_eq!("some view", views[0].name);
    }

    #[test]
    fn normalize_ucs() {
        let mut file = Drawing::new();
        file.clear();
        assert_eq!(0, file.ucss().count());
        file.header.ucs_name = String::from("primary ucs");
        file.normalize();
        let ucss = file.ucss().collect::<Vec<_>>();
        assert_eq!(1, ucss.len());
        assert_eq!("primary ucs", ucss[0].name);
    }

    #[test]
    fn block_record_table_not_written_on_r12() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R12;
        assert_not_contains_pairs(&drawing, vec![CodePair::new_str(0, "BLOCK_RECORD")]);
    }

    #[test]
    fn block_record_table_is_written_on_r13() {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R13;
        assert_contains_pairs(&drawing, vec![CodePair::new_str(0, "BLOCK_RECORD")]);
    }
}
