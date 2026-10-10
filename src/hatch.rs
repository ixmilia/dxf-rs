use crate::{Handle, Point, Vector};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundaryPath {
    pub boundary_type_flags: i32,
    pub edges: Vec<BoundaryPathEdge>,
    pub source_boundary_objects: Vec<Handle>,
    pub is_closed: bool,
}

impl BoundaryPath {
    pub fn new() -> BoundaryPath {
        BoundaryPath {
            boundary_type_flags: 0,
            edges: vec![],
            source_boundary_objects: vec![],
            is_closed: true,
        }
    }

    pub fn is_external(&self) -> bool {
        (self.boundary_type_flags & 1) != 0
    }

    pub fn set_external(&mut self, is_external: bool) {
        if is_external {
            self.boundary_type_flags |= 1;
        } else {
            self.boundary_type_flags &= !1;
        }
    }

    pub fn is_polyline(&self) -> bool {
        (self.boundary_type_flags & 2) != 0
    }

    pub fn set_polyline(&mut self, is_polyline: bool) {
        if is_polyline {
            self.boundary_type_flags |= 2;
        } else {
            self.boundary_type_flags &= !2;
        }
    }

    pub fn add_polyline_vertex(&mut self, vertex: Point) {
        self.edges.push(BoundaryPathEdge::Polyline { vertex });
        self.set_polyline(true);
    }

    pub fn add_line_edge(&mut self, start: Point, end: Point) {
        self.edges.push(BoundaryPathEdge::Line { start, end });
    }

    pub fn add_circular_arc_edge(
        &mut self,
        center: Point,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        is_counter_clockwise: bool,
    ) {
        self.edges.push(BoundaryPathEdge::CircularArc {
            center,
            radius,
            start_angle,
            end_angle,
            is_counter_clockwise,
        });
    }

    pub fn add_elliptical_arc_edge(
        &mut self,
        center: Point,
        major_axis_endpoint: Point,
        minor_axis_ratio: f64,
        start_angle: f64,
        end_angle: f64,
        is_counter_clockwise: bool,
    ) {
        self.edges.push(BoundaryPathEdge::EllipticalArc {
            center,
            major_axis_endpoint,
            minor_axis_ratio,
            start_angle,
            end_angle,
            is_counter_clockwise,
        });
    }

    pub fn add_spline_edge(
        &mut self,
        degree: i32,
        rational: bool,
        periodic: bool,
        knots: Vec<f64>,
        control_points: Vec<Point>,
        weights: Option<Vec<f64>>,
    ) {
        self.edges.push(BoundaryPathEdge::Spline {
            degree,
            rational,
            periodic,
            knots,
            control_points,
            weights,
        });
    }

    /// Create a rectangular boundary path
    pub fn from_rectangle(
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        is_external: bool,
    ) -> BoundaryPath {
        let mut path = BoundaryPath::new();
        path.set_external(is_external);

        let corners = vec![
            Point::new(min_x, min_y, 0.0),
            Point::new(max_x, min_y, 0.0),
            Point::new(max_x, max_y, 0.0),
            Point::new(min_x, max_y, 0.0),
        ];

        for vertex in corners {
            path.add_polyline_vertex(vertex);
        }

        path
    }

    /// Create a circular boundary path
    pub fn from_circle(center: Point, radius: f64, is_external: bool) -> BoundaryPath {
        let mut path = BoundaryPath::new();
        path.set_external(is_external);
        path.add_circular_arc_edge(center, radius, 0.0, 360.0, false);
        path
    }

    /// Create a polygon boundary path from points
    pub fn from_polygon(points: Vec<Point>, is_external: bool) -> BoundaryPath {
        let mut path = BoundaryPath::new();
        path.set_external(is_external);

        for point in points {
            path.add_polyline_vertex(point);
        }

        path
    }
}

impl Default for BoundaryPath {
    fn default() -> Self {
        BoundaryPath::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum BoundaryPathEdge {
    Line {
        start: Point,
        end: Point,
    },
    CircularArc {
        center: Point,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        is_counter_clockwise: bool,
    },
    EllipticalArc {
        center: Point,
        major_axis_endpoint: Point,
        minor_axis_ratio: f64,
        start_angle: f64,
        end_angle: f64,
        is_counter_clockwise: bool,
    },
    Spline {
        degree: i32,
        rational: bool,
        periodic: bool,
        knots: Vec<f64>,
        control_points: Vec<Point>,
        weights: Option<Vec<f64>>,
    },
    Polyline {
        vertex: Point,
    },
}

impl BoundaryPathEdge {
    pub fn edge_type(&self) -> i32 {
        match self {
            BoundaryPathEdge::Line { .. } => 1,
            BoundaryPathEdge::CircularArc { .. } => 2,
            BoundaryPathEdge::EllipticalArc { .. } => 3,
            BoundaryPathEdge::Spline { .. } => 4,
            BoundaryPathEdge::Polyline { .. } => 0, // Special case for polyline boundary
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternDefinitionLine {
    pub angle: f64,
    pub base_point: Point,
    pub offset: Vector,
    pub dash_lengths: Vec<f64>,
}

impl PatternDefinitionLine {
    pub fn new(angle: f64, base_point: Point, offset: Vector) -> PatternDefinitionLine {
        PatternDefinitionLine {
            angle,
            base_point,
            offset,
            dash_lengths: vec![],
        }
    }

    pub fn new_with_dashes(
        angle: f64,
        base_point: Point,
        offset: Vector,
        dash_lengths: Vec<f64>,
    ) -> PatternDefinitionLine {
        PatternDefinitionLine {
            angle,
            base_point,
            offset,
            dash_lengths,
        }
    }

    pub fn add_dash_length(&mut self, length: f64) {
        self.dash_lengths.push(length);
    }
}

impl Default for PatternDefinitionLine {
    fn default() -> Self {
        PatternDefinitionLine::new(0.0, Point::origin(), Vector::new(0.0, 0.0, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Entity, EntityType, Hatch};
    use crate::enums::{HatchPatternType, HatchStyle};
    use crate::{Drawing, Point};

    #[test]
    fn test_create_simple_rectangle_hatch() {
        let hatch = Hatch::new_rectangle_solid_fill(0.0, 0.0, 10.0, 5.0);

        assert!(hatch.solid_fill);
        assert_eq!(hatch.hatch_pattern_name, "SOLID");
        assert_eq!(hatch.boundary_paths.len(), 1);

        let boundary = &hatch.boundary_paths[0];
        assert!(boundary.is_external());
        assert!(boundary.is_polyline());
        assert_eq!(boundary.edges.len(), 4);
    }

    #[test]
    fn test_create_circular_hatch() {
        let center = Point::new(5.0, 5.0, 0.0);
        let hatch = Hatch::new_circle_solid_fill(center.clone(), 3.0);

        assert!(hatch.solid_fill);
        assert_eq!(hatch.boundary_paths.len(), 1);

        let boundary = &hatch.boundary_paths[0];
        assert!(boundary.is_external());
        assert_eq!(boundary.edges.len(), 1);

        match &boundary.edges[0] {
            BoundaryPathEdge::CircularArc {
                center: arc_center,
                radius,
                ..
            } => {
                assert_eq!(*arc_center, center);
                assert_eq!(*radius, 3.0);
            }
            _ => panic!("Expected circular arc edge"),
        }
    }

    #[test]
    fn test_create_polygon_hatch() {
        let points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
            Point::new(10.0, 5.0, 0.0),
            Point::new(5.0, 8.0, 0.0),
            Point::new(0.0, 5.0, 0.0),
        ];

        let hatch = Hatch::new_polygon_solid_fill(points.clone());

        assert!(hatch.solid_fill);
        assert_eq!(hatch.boundary_paths.len(), 1);

        let boundary = &hatch.boundary_paths[0];
        assert!(boundary.is_external());
        assert!(boundary.is_polyline());
        assert_eq!(boundary.edges.len(), points.len());
    }

    #[test]
    fn test_create_polygon_with_holes() {
        let outer_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(20.0, 0.0, 0.0),
            Point::new(20.0, 20.0, 0.0),
            Point::new(0.0, 20.0, 0.0),
        ];

        let hole1_points = vec![
            Point::new(5.0, 5.0, 0.0),
            Point::new(8.0, 5.0, 0.0),
            Point::new(8.0, 8.0, 0.0),
            Point::new(5.0, 8.0, 0.0),
        ];

        let hole2_points = vec![
            Point::new(12.0, 12.0, 0.0),
            Point::new(15.0, 12.0, 0.0),
            Point::new(15.0, 15.0, 0.0),
            Point::new(12.0, 15.0, 0.0),
        ];

        let hatch = Hatch::new_polygon_with_holes_solid_fill(
            outer_points.clone(),
            vec![hole1_points.clone(), hole2_points.clone()],
        );

        assert!(hatch.solid_fill);
        assert_eq!(hatch.hatch_style, HatchStyle::OddParity);
        assert_eq!(hatch.boundary_paths.len(), 3); // 1 outer + 2 holes

        // Check outer boundary
        let outer_boundary = &hatch.boundary_paths[0];
        assert!(outer_boundary.is_external());
        assert_eq!(outer_boundary.edges.len(), outer_points.len());

        // Check holes
        let hole1_boundary = &hatch.boundary_paths[1];
        assert!(!hole1_boundary.is_external()); // holes should not be external
        assert_eq!(hole1_boundary.edges.len(), hole1_points.len());

        let hole2_boundary = &hatch.boundary_paths[2];
        assert!(!hole2_boundary.is_external()); // holes should not be external
        assert_eq!(hole2_boundary.edges.len(), hole2_points.len());
    }

    #[test]
    fn test_add_holes_to_existing_hatch() {
        let mut hatch = Hatch::new_rectangle_solid_fill(0.0, 0.0, 20.0, 20.0);

        // Add a circular hole
        hatch.add_circular_hole(Point::new(5.0, 5.0, 0.0), 2.0);

        // Add a rectangular hole
        hatch.add_rectangular_hole(10.0, 10.0, 15.0, 15.0);

        // Add a polygon hole
        let triangle_hole = vec![
            Point::new(2.0, 15.0, 0.0),
            Point::new(5.0, 18.0, 0.0),
            Point::new(8.0, 15.0, 0.0),
        ];
        hatch.add_hole(triangle_hole);

        assert_eq!(hatch.boundary_paths.len(), 4); // 1 outer + 3 holes

        // Verify all holes are marked as internal (not external)
        for i in 1..hatch.boundary_paths.len() {
            assert!(!hatch.boundary_paths[i].is_external());
        }
    }

    #[test]
    fn test_boundary_path_edge_types() {
        let mut boundary = BoundaryPath::new();

        // Add different types of edges
        boundary.add_line_edge(Point::new(0.0, 0.0, 0.0), Point::new(5.0, 0.0, 0.0));
        boundary.add_circular_arc_edge(Point::new(7.5, 2.5, 0.0), 2.5, 0.0, 180.0, false);
        boundary.add_elliptical_arc_edge(
            Point::new(10.0, 0.0, 0.0),
            Point::new(5.0, 0.0, 0.0),
            0.5,
            0.0,
            180.0,
            false,
        );

        assert_eq!(boundary.edges.len(), 3);

        match &boundary.edges[0] {
            BoundaryPathEdge::Line { .. } => {
                assert_eq!(boundary.edges[0].edge_type(), 1);
            }
            _ => panic!("Expected line edge"),
        }

        match &boundary.edges[1] {
            BoundaryPathEdge::CircularArc { .. } => {
                assert_eq!(boundary.edges[1].edge_type(), 2);
            }
            _ => panic!("Expected circular arc edge"),
        }

        match &boundary.edges[2] {
            BoundaryPathEdge::EllipticalArc { .. } => {
                assert_eq!(boundary.edges[2].edge_type(), 3);
            }
            _ => panic!("Expected elliptical arc edge"),
        }
    }

    #[test]
    fn test_hatch_in_drawing() {
        let mut drawing = Drawing::new();

        // Create a hatch with a hole
        let outer_points = vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(10.0, 0.0, 0.0),
            Point::new(10.0, 10.0, 0.0),
            Point::new(0.0, 10.0, 0.0),
        ];

        let hole_points = vec![
            Point::new(3.0, 3.0, 0.0),
            Point::new(7.0, 3.0, 0.0),
            Point::new(7.0, 7.0, 0.0),
            Point::new(3.0, 7.0, 0.0),
        ];

        let hatch = Hatch::new_polygon_with_holes_solid_fill(outer_points, vec![hole_points]);

        let entity = Entity::new(EntityType::Hatch(hatch));
        drawing.add_entity(entity);

        assert_eq!(drawing.entities().count(), 1);

        let first_entity = drawing.entities().next().unwrap();
        match &first_entity.specific {
            EntityType::Hatch(h) => {
                assert!(h.solid_fill);
                assert_eq!(h.boundary_paths.len(), 2);
            }
            _ => panic!("Expected hatch entity"),
        }
    }

    #[test]
    fn test_hatch_default_values() {
        let hatch = Hatch::default();

        assert!(hatch.solid_fill);
        assert!(!hatch.associative);
        assert_eq!(hatch.hatch_style, HatchStyle::OddParity);
        assert_eq!(hatch.hatch_pattern_type, HatchPatternType::Predefined);
        assert_eq!(hatch.hatch_pattern_angle, 0.0);
        assert_eq!(hatch.hatch_pattern_scale, 1.0);
        assert!(!hatch.hatch_pattern_double);
        assert_eq!(hatch.elevation, 0.0);
        assert_eq!(hatch.pixel_size, 0.0);
        assert!(hatch.boundary_paths.is_empty());
        assert!(hatch.pattern_definition_lines.is_empty());
        assert!(hatch.seed_points.is_empty());
    }

    #[test]
    fn test_hatch_with_pattern_definition_lines() {
        let boundary = BoundaryPath::from_rectangle(0.0, 0.0, 10.0, 10.0, true);

        let pattern_line = PatternDefinitionLine::new_with_dashes(
            45.0,
            Point::new(0.0, 0.0, 0.0),
            Vector::new(0.125, 0.0, 0.0),
            vec![0.125, -0.0625],
        );

        let hatch = Hatch::new_pattern_fill(boundary, String::from("ANSI31"), vec![pattern_line]);

        assert!(!hatch.solid_fill);
        assert_eq!(hatch.hatch_pattern_name, "ANSI31");
        assert_eq!(hatch.pattern_definition_lines.len(), 1);

        let pattern = &hatch.pattern_definition_lines[0];
        assert_eq!(pattern.angle, 45.0);
        assert_eq!(pattern.base_point, Point::new(0.0, 0.0, 0.0));
        assert_eq!(pattern.offset, Vector::new(0.125, 0.0, 0.0));
        assert_eq!(pattern.dash_lengths.len(), 2);
        assert_eq!(pattern.dash_lengths[0], 0.125);
        assert_eq!(pattern.dash_lengths[1], -0.0625);
    }

    #[test]
    fn test_hatch_with_seed_points() {
        let mut hatch = Hatch::new_rectangle_solid_fill(0.0, 0.0, 10.0, 10.0);

        hatch.add_seed_point(Point::new(5.0, 5.0, 0.0));
        hatch.add_seed_point(Point::new(2.5, 2.5, 0.0));
        hatch.add_seed_point(Point::new(7.5, 7.5, 0.0));

        assert_eq!(hatch.seed_points.len(), 3);
        assert_eq!(hatch.seed_points[0], Point::new(5.0, 5.0, 0.0));
        assert_eq!(hatch.seed_points[1], Point::new(2.5, 2.5, 0.0));
        assert_eq!(hatch.seed_points[2], Point::new(7.5, 7.5, 0.0));
    }

    #[test]
    fn test_hatch_boundary_path_with_edges() {
        let mut boundary = BoundaryPath::new();
        boundary.set_external(true);

        // Add line edge
        boundary.add_line_edge(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));

        // Add circular arc edge
        boundary.add_circular_arc_edge(Point::new(15.0, 5.0, 0.0), 5.0, 0.0, 90.0, false);

        // Add elliptical arc edge
        boundary.add_elliptical_arc_edge(
            Point::new(25.0, 5.0, 0.0),
            Point::new(5.0, 0.0, 0.0),
            0.5,
            0.0,
            180.0,
            true,
        );

        assert_eq!(boundary.edges.len(), 3);
        assert!(boundary.is_external());
        assert!(!boundary.is_polyline());

        // Test edge types
        match &boundary.edges[0] {
            BoundaryPathEdge::Line { start, end } => {
                assert_eq!(*start, Point::new(0.0, 0.0, 0.0));
                assert_eq!(*end, Point::new(10.0, 0.0, 0.0));
            }
            _ => panic!("Expected line edge"),
        }

        match &boundary.edges[1] {
            BoundaryPathEdge::CircularArc {
                center,
                radius,
                start_angle,
                end_angle,
                is_counter_clockwise,
            } => {
                assert_eq!(*center, Point::new(15.0, 5.0, 0.0));
                assert_eq!(*radius, 5.0);
                assert_eq!(*start_angle, 0.0);
                assert_eq!(*end_angle, 90.0);
                assert!(!is_counter_clockwise);
            }
            _ => panic!("Expected circular arc edge"),
        }

        match &boundary.edges[2] {
            BoundaryPathEdge::EllipticalArc {
                center,
                major_axis_endpoint,
                minor_axis_ratio,
                start_angle,
                end_angle,
                is_counter_clockwise,
            } => {
                assert_eq!(*center, Point::new(25.0, 5.0, 0.0));
                assert_eq!(*major_axis_endpoint, Point::new(5.0, 0.0, 0.0));
                assert_eq!(*minor_axis_ratio, 0.5);
                assert_eq!(*start_angle, 0.0);
                assert_eq!(*end_angle, 180.0);
                assert!(*is_counter_clockwise);
            }
            _ => panic!("Expected elliptical arc edge"),
        }
    }

    #[test]
    fn test_hatch_with_source_boundary_objects() {
        let mut boundary = BoundaryPath::new();
        boundary.set_external(true);
        boundary.add_line_edge(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 0.0, 0.0));

        // Add source boundary object handles
        boundary.source_boundary_objects.push(Handle(0x1A));
        boundary.source_boundary_objects.push(Handle(0x1B));

        assert_eq!(boundary.source_boundary_objects.len(), 2);
        assert_eq!(boundary.source_boundary_objects[0], Handle(0x1A));
        assert_eq!(boundary.source_boundary_objects[1], Handle(0x1B));
    }

    #[test]
    fn test_hatch_with_no_boundary_no_pattern() {
        let mut hatch = Hatch::default();
        hatch.pixel_size = 1.0;
        hatch.associative = true;

        assert!(hatch.boundary_paths.is_empty());
        assert!(hatch.pattern_definition_lines.is_empty());
        assert_eq!(hatch.pixel_size, 1.0);
        assert!(hatch.associative);
    }

    #[test]
    fn test_pattern_definition_line_methods() {
        let mut pattern_line = PatternDefinitionLine::new(
            30.0,
            Point::new(1.0, 2.0, 0.0),
            Vector::new(0.1, 0.05, 0.0),
        );

        assert_eq!(pattern_line.angle, 30.0);
        assert_eq!(pattern_line.base_point, Point::new(1.0, 2.0, 0.0));
        assert_eq!(pattern_line.offset, Vector::new(0.1, 0.05, 0.0));
        assert!(pattern_line.dash_lengths.is_empty());

        pattern_line.add_dash_length(0.5);
        pattern_line.add_dash_length(-0.25);
        pattern_line.add_dash_length(0.75);

        assert_eq!(pattern_line.dash_lengths.len(), 3);
        assert_eq!(pattern_line.dash_lengths[0], 0.5);
        assert_eq!(pattern_line.dash_lengths[1], -0.25);
        assert_eq!(pattern_line.dash_lengths[2], 0.75);
    }

    #[test]
    fn test_hatch_associative_pixel_size_behavior() {
        let mut hatch1 = Hatch::new_rectangle_solid_fill(0.0, 0.0, 10.0, 10.0);
        hatch1.pixel_size = 2.0;
        hatch1.associative = false;

        assert_eq!(hatch1.pixel_size, 2.0);
        assert!(!hatch1.associative);

        let mut hatch2 = Hatch::new_rectangle_solid_fill(0.0, 0.0, 10.0, 10.0);
        hatch2.pixel_size = 1.5;
        hatch2.associative = true;

        assert_eq!(hatch2.pixel_size, 1.5);
        assert!(hatch2.associative);
    }

    #[test]
    fn test_read_hatch_pattern_definition() {
        let mut hatch = Hatch::default();
        hatch.hatch_pattern_double = true;
        hatch.pixel_size = 99.0;
        hatch.associative = true;

        let line1 = PatternDefinitionLine::new_with_dashes(
            1.0,
            Point::new(2.0, 3.0, 0.0),
            Vector::new(4.0, 5.0, 0.0),
            vec![6.0, 7.0],
        );

        let line2 = PatternDefinitionLine::new_with_dashes(
            8.0,
            Point::new(9.0, 10.0, 0.0),
            Vector::new(11.0, 12.0, 0.0),
            vec![13.0, 14.0],
        );

        hatch.pattern_definition_lines = vec![line1, line2];

        assert!(hatch.hatch_pattern_double);
        assert_eq!(99.0, hatch.pixel_size);

        assert_eq!(2, hatch.pattern_definition_lines.len());
        let line1 = &hatch.pattern_definition_lines[0];
        assert_eq!(1.0, line1.angle);
        assert_eq!(2.0, line1.base_point.x);
        assert_eq!(3.0, line1.base_point.y);
        assert_eq!(4.0, line1.offset.x);
        assert_eq!(5.0, line1.offset.y);
        assert_eq!(vec![6.0, 7.0], line1.dash_lengths);

        let line2 = &hatch.pattern_definition_lines[1];
        assert_eq!(8.0, line2.angle);
        assert_eq!(9.0, line2.base_point.x);
        assert_eq!(10.0, line2.base_point.y);
        assert_eq!(11.0, line2.offset.x);
        assert_eq!(12.0, line2.offset.y);
        assert_eq!(vec![13.0, 14.0], line2.dash_lengths);
    }

    #[test]
    fn test_write_hatch_pattern_definition() {
        let mut hatch = Hatch::default();
        hatch.hatch_pattern_double = true;

        let line1 = PatternDefinitionLine::new_with_dashes(
            1.0,
            Point::new(2.0, 3.0, 0.0),
            Vector::new(4.0, 5.0, 0.0),
            vec![6.0, 7.0],
        );

        let line2 = PatternDefinitionLine::new_with_dashes(
            8.0,
            Point::new(9.0, 10.0, 0.0),
            Vector::new(11.0, 12.0, 0.0),
            vec![13.0, 14.0],
        );

        hatch.pattern_definition_lines.push(line1);
        hatch.pattern_definition_lines.push(line2);

        assert!(hatch.hatch_pattern_double);
        assert_eq!(2, hatch.pattern_definition_lines.len());

        assert_eq!(hatch.pattern_definition_lines[0].angle, 1.0);
        assert_eq!(hatch.pattern_definition_lines[0].base_point.x, 2.0);
        assert_eq!(hatch.pattern_definition_lines[0].base_point.y, 3.0);
        assert_eq!(hatch.pattern_definition_lines[1].angle, 8.0);
        assert_eq!(hatch.pattern_definition_lines[1].base_point.x, 9.0);
        assert_eq!(hatch.pattern_definition_lines[1].base_point.y, 10.0);
    }

    #[test]
    fn test_read_hatch_seed_points() {
        let mut hatch = Hatch::default();
        hatch.pixel_size = 99.0;

        hatch.seed_points.push(Point::new(1.0, 2.0, 0.0));
        hatch.seed_points.push(Point::new(3.0, 4.0, 0.0));

        assert_eq!(99.0, hatch.pixel_size);

        assert_eq!(2, hatch.seed_points.len());
        assert_eq!(Point::new(1.0, 2.0, 0.0), hatch.seed_points[0]);
        assert_eq!(Point::new(3.0, 4.0, 0.0), hatch.seed_points[1]);
    }

    #[test]
    fn test_write_hatch_seed_points() {
        let mut hatch = Hatch::default();
        hatch.seed_points.push(Point::new(1.0, 2.0, 0.0));
        hatch.seed_points.push(Point::new(3.0, 4.0, 0.0));

        assert_eq!(2, hatch.seed_points.len());
        assert_eq!(hatch.seed_points[0], Point::new(1.0, 2.0, 0.0));
        assert_eq!(hatch.seed_points[1], Point::new(3.0, 4.0, 0.0));
    }

    #[test]
    fn test_read_hatch_boundary_path_data() {
        let mut hatch = Hatch::default();
        hatch.elevation = 99.0;
        hatch.associative = true;
        hatch.hatch_style = HatchStyle::OddParity;

        let mut polyline_boundary = BoundaryPath::new();
        polyline_boundary.boundary_type_flags = 3;
        polyline_boundary.set_polyline(true);
        polyline_boundary.set_external(true);

        polyline_boundary.add_polyline_vertex(Point::new(1.0, 2.0, 0.0));
        polyline_boundary.add_polyline_vertex(Point::new(3.0, 4.0, 0.0));

        polyline_boundary
            .source_boundary_objects
            .push(Handle(0xABC));
        polyline_boundary
            .source_boundary_objects
            .push(Handle(0xDEF));

        // Second boundary path - non-polyline with edges
        let mut edge_boundary = BoundaryPath::new();
        edge_boundary.boundary_type_flags = 8; // non polyline text box (92, 8)

        // Add line edge
        edge_boundary.add_line_edge(
            Point::new(1.0, 2.0, 0.0), // start point
            Point::new(3.0, 4.0, 0.0), // end point
        );

        // Add circular arc edge
        edge_boundary.add_circular_arc_edge(
            Point::new(1.0, 2.0, 0.0), // center
            3.0,                       // radius
            4.0,                       // start angle
            5.0,                       // end angle
            true,                      // is counter clockwise
        );

        // Add elliptical arc edge
        edge_boundary.add_elliptical_arc_edge(
            Point::new(1.0, 2.0, 0.0), // center
            Point::new(3.0, 4.0, 0.0), // major axis endpoint
            5.0,                       // minor axis ratio
            6.0,                       // start angle
            7.0,                       // end angle
            true,                      // is counter clockwise
        );

        edge_boundary.add_spline_edge(
            2,
            true,
            true,
            vec![1.0, 2.0],
            vec![Point::new(3.0, 4.0, 0.0), Point::new(5.0, 6.0, 0.0)],
            Some(vec![7.0, 8.0]),
        );

        hatch.boundary_paths = vec![polyline_boundary, edge_boundary];

        assert_eq!(99.0, hatch.elevation);
        assert!(hatch.associative);
        assert_eq!(HatchStyle::OddParity, hatch.hatch_style);

        assert_eq!(2, hatch.boundary_paths.len());

        let poly_path = &hatch.boundary_paths[0];
        assert!(poly_path.is_polyline());
        assert!(poly_path.is_external());
        assert_eq!(2, poly_path.edges.len());
        assert_eq!(2, poly_path.source_boundary_objects.len());
        assert_eq!(Handle(0xABC), poly_path.source_boundary_objects[0]);
        assert_eq!(Handle(0xDEF), poly_path.source_boundary_objects[1]);

        let edge_path = &hatch.boundary_paths[1];
        assert!(!edge_path.is_polyline());
        assert_eq!(4, edge_path.edges.len());

        match &edge_path.edges[0] {
            BoundaryPathEdge::Line { start, end } => {
                assert_eq!(*start, Point::new(1.0, 2.0, 0.0));
                assert_eq!(*end, Point::new(3.0, 4.0, 0.0));
            }
            _ => panic!("Expected line edge"),
        }

        match &edge_path.edges[1] {
            BoundaryPathEdge::CircularArc {
                center,
                radius,
                start_angle,
                end_angle,
                is_counter_clockwise,
            } => {
                assert_eq!(*center, Point::new(1.0, 2.0, 0.0));
                assert_eq!(*radius, 3.0);
                assert_eq!(*start_angle, 4.0);
                assert_eq!(*end_angle, 5.0);
                assert!(*is_counter_clockwise);
            }
            _ => panic!("Expected circular arc edge"),
        }

        match &edge_path.edges[2] {
            BoundaryPathEdge::EllipticalArc {
                center,
                major_axis_endpoint,
                minor_axis_ratio,
                start_angle,
                end_angle,
                is_counter_clockwise,
            } => {
                assert_eq!(*center, Point::new(1.0, 2.0, 0.0));
                assert_eq!(*major_axis_endpoint, Point::new(3.0, 4.0, 0.0));
                assert_eq!(*minor_axis_ratio, 5.0);
                assert_eq!(*start_angle, 6.0);
                assert_eq!(*end_angle, 7.0);
                assert!(*is_counter_clockwise);
            }
            _ => panic!("Expected elliptical arc edge"),
        }

        match &edge_path.edges[3] {
            BoundaryPathEdge::Spline {
                degree,
                rational,
                periodic,
                knots,
                control_points,
                weights,
            } => {
                assert_eq!(*degree, 2);
                assert!(*rational);
                assert!(*periodic);
                assert_eq!(*knots, vec![1.0, 2.0]);
                assert_eq!(
                    *control_points,
                    vec![Point::new(3.0, 4.0, 0.0), Point::new(5.0, 6.0, 0.0)]
                );
                assert_eq!(*weights, Some(vec![7.0, 8.0]));
            }
            _ => panic!("Expected spline edge"),
        }
    }

    #[test]
    fn test_write_hatch_boundary_path_data() {
        let mut hatch = Hatch::default();
        hatch.associative = true;
        hatch.hatch_style = HatchStyle::EntireArea;

        let mut polyline_boundary = BoundaryPath::new();
        polyline_boundary.set_polyline(true);
        polyline_boundary.set_external(true);
        polyline_boundary.add_polyline_vertex(Point::new(1.0, 2.0, 0.0));
        polyline_boundary.add_polyline_vertex(Point::new(3.0, 4.0, 0.0));
        polyline_boundary
            .source_boundary_objects
            .push(Handle(0xABC));
        polyline_boundary
            .source_boundary_objects
            .push(Handle(0xDEF));

        let mut edge_boundary = BoundaryPath::new();
        edge_boundary.boundary_type_flags = 8;
        edge_boundary.add_line_edge(Point::new(1.0, 2.0, 0.0), Point::new(3.0, 4.0, 0.0));
        edge_boundary.add_circular_arc_edge(Point::new(1.0, 2.0, 0.0), 3.0, 4.0, 5.0, true);
        edge_boundary.add_elliptical_arc_edge(
            Point::new(1.0, 2.0, 0.0),
            Point::new(3.0, 4.0, 0.0),
            5.0,
            6.0,
            7.0,
            true,
        );
        edge_boundary.add_spline_edge(
            2,
            true,
            true,
            vec![1.0, 2.0],
            vec![Point::new(3.0, 4.0, 0.0), Point::new(5.0, 6.0, 0.0)],
            Some(vec![7.0, 8.0]),
        );

        hatch.boundary_paths = vec![polyline_boundary, edge_boundary];

        assert!(hatch.associative);
        assert_eq!(HatchStyle::EntireArea, hatch.hatch_style);
        assert_eq!(2, hatch.boundary_paths.len());
    }

    #[test]
    fn test_read_hatch_boundary_path_with_source_boundary_object_handles() {
        let mut hatch = Hatch::default();

        let mut boundary = BoundaryPath::new();
        boundary.boundary_type_flags = 1; // external non-polyline boundary type
        boundary.source_boundary_objects.push(Handle(0xABC));

        hatch.boundary_paths = vec![boundary];

        assert_eq!(1, hatch.boundary_paths.len());
        let external_path = &hatch.boundary_paths[0];
        assert_eq!(1, external_path.source_boundary_objects.len());
        let boundary_handle = external_path.source_boundary_objects[0];
        assert_eq!(Handle(0xABC), boundary_handle);
    }

    #[test]
    fn test_read_hatch_with_no_boundary_no_pattern() {
        let mut hatch = Hatch::default();
        hatch.pixel_size = 42.0;

        assert!(hatch.boundary_paths.is_empty());
        assert!(hatch.pattern_definition_lines.is_empty());
        assert_eq!(42.0, hatch.pixel_size);
    }

    #[test]
    fn test_hatch_extrusion_direction_default() {
        let hatch = Hatch::default();

        // Default extrusion direction should be Z-axis (0, 0, 1)
        assert_eq!(hatch.extrusion_direction, Vector::z_axis());
        assert_eq!(hatch.extrusion_direction.x, 0.0);
        assert_eq!(hatch.extrusion_direction.y, 0.0);
        assert_eq!(hatch.extrusion_direction.z, 1.0);
    }

    #[test]
    fn test_hatch_custom_extrusion_direction() {
        let mut hatch = Hatch::new_rectangle_solid_fill(0.0, 0.0, 10.0, 10.0);

        // Set custom extrusion direction (normal vector)
        hatch.extrusion_direction = Vector::new(1.0, 0.0, 0.0); // X-axis normal

        assert_eq!(hatch.extrusion_direction.x, 1.0);
        assert_eq!(hatch.extrusion_direction.y, 0.0);
        assert_eq!(hatch.extrusion_direction.z, 0.0);
    }

    #[test]
    fn test_hatch_with_y_axis_normal() {
        let mut hatch = Hatch::new_circle_solid_fill(Point::new(5.0, 5.0, 0.0), 3.0);

        // Set Y-axis as normal vector
        hatch.extrusion_direction = Vector::new(0.0, 1.0, 0.0);

        assert_eq!(hatch.extrusion_direction, Vector::new(0.0, 1.0, 0.0));
        assert!(hatch.solid_fill);
        assert_eq!(hatch.boundary_paths.len(), 1);
    }

    #[test]
    fn test_hatch_with_diagonal_normal() {
        let mut hatch = Hatch::default();

        // Set diagonal normal vector (normalized)
        let diagonal_normal = Vector::new(0.5773, 0.5773, 0.5773); // roughly (1,1,1) normalized
        hatch.extrusion_direction = diagonal_normal.clone();

        assert_eq!(hatch.extrusion_direction, diagonal_normal);
    }
}
