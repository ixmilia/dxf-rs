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
        match self.color.get_raw_value() {
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

    fn read_table(table_name: &str, value_pairs: Vec<&str>) -> Drawing {
        let mut pairs = vec![
            "0",
            "SECTION",
            "2",
            "TABLES",
            "0",
            "TABLE",
            "2",
            table_name,
            "100",
            "AcDbSymbolTable",
            "70",
            "0",
        ];

        for pair in value_pairs {
            pairs.push(pair);
        }

        pairs.push("0");
        pairs.push("ENDTAB");
        pairs.push("0");
        pairs.push("ENDSEC");
        pairs.push("0");
        pairs.push("EOF");

        parse_drawing(pairs.join("\r\n").as_str())
    }

    #[test]
    fn read_unsupported_table() {
        let drawing = parse_drawing(
            vec![
                "0",
                "SECTION",
                "2",
                "TABLES",
                "0",
                "TABLE",
                "2",
                "UNSUPPORTED",
                "0",
                "UNSUPPORTED",
                "2",
                "unsupported-name",
                "0",
                "ENDTAB",
                "0",
                "TABLE",
                "2",
                "LAYER",
                "0",
                "LAYER",
                "0",
                "ENDTAB",
                "0",
                "ENDSEC",
                "0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
        assert_eq!(1, drawing.layers().count());
    }

    #[test]
    fn read_single_layer() {
        let drawing = read_table("LAYER", vec!["0", "LAYER", "2", "layer-name"]);
        let layers = drawing.layers().collect::<Vec<_>>();
        assert_eq!(1, layers.len());
        assert_eq!("layer-name", layers[0].name);
    }

    #[test]
    fn read_variable_table_items() {
        let drawing = parse_drawing(
            vec![
                "0",
                "SECTION",
                "2",
                "TABLES",
                // no app ids
                "0",
                "TABLE",
                "2",
                "APPID",
                "0",
                "ENDTAB",
                // 1 layer
                "0",
                "TABLE",
                "2",
                "LAYER",
                "0",
                "LAYER",
                "2",
                "layer-name",
                "0",
                "ENDTAB",
                // 2 styles
                "0",
                "TABLE",
                "2",
                "STYLE",
                "0",
                "STYLE",
                "40",
                "1.1",
                "0",
                "STYLE",
                "40",
                "2.2",
                "0",
                "ENDTAB",
                "0",
                "ENDSEC",
                "0",
                "EOF",
            ]
            .join("\r\n")
            .as_str(),
        );
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
        let drawing = read_table("LAYER", vec!["0", "LAYER", "62", "5"]);
        let layers = drawing.layers().collect::<Vec<_>>();
        let layer = layers[0];
        assert_eq!(Some(5), layer.color.index());
        assert!(layer.is_layer_on);
    }

    #[test]
    fn read_layer_color_and_layer_is_off() {
        let drawing = read_table("LAYER", vec!["0", "LAYER", "62", "-5"]);
        let layers = drawing.layers().collect::<Vec<_>>();
        let layer = layers[0];
        assert_eq!(Some(5), layer.color.index());
        assert!(!layer.is_layer_on);
    }

    #[test]
    fn write_layer() {
        let mut drawing = Drawing::new();
        let mut layer = Layer::default();
        layer.name = String::from("layer-name");
        layer.color = Color::from_index(3);
        drawing.add_layer(layer);
        assert_contains(
            &drawing,
            vec![
                "  0",
                "LAYER",
                "  5",
                "10",
                "100",
                "AcDbSymbolTableRecord",
                "100",
                "AcDbLayerTableRecord",
                "  2",
                "layer-name",
                " 70",
                "     0",
                " 62",
                "     3",
                "  6",
                "CONTINUOUS",
            ]
            .join("\r\n"),
        );
    }

    #[test]
    fn normalize_layer() {
        let mut layer = Layer::default();
        layer.name = String::from("layer-name");
        layer.color = Color::by_layer(); // value 256 not valid; normalized to 7
        layer.line_type_name = String::from(""); // empty string not valid; normalized to CONTINUOUS
        layer.normalize();
        assert_eq!(Some(7), layer.color.index());
        assert_eq!("CONTINUOUS", layer.line_type_name);
    }

    #[test]
    fn normalize_view() {
        let mut view = View::default();
        view.view_height = 0.0; // invalid; normalized to 1.0
        view.view_width = -1.0; // invalid; normalized to 1.0
        view.lens_length = 42.0; // valid
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
                "  0",
                "LAYER",
                "102",
                "{IXMILIA",
                "  1",
                "some string",
                "102",
                "}",
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
        assert_contains(
            &drawing,
            vec!["102", "{IXMILIA", "  1", "some string", "102", "}"].join("\r\n"),
        );
    }

    #[test]
    fn read_table_item_with_x_data() {
        let drawing = read_table(
            "LAYER",
            vec!["  0", "LAYER", "1001", "IXMILIA", "1040", "1.1"],
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
        assert_contains(
            &drawing,
            vec!["1001", "IXMILIA", "1040", "1.1"].join("\r\n"),
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
}
