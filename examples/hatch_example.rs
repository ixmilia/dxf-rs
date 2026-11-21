use dxf::entities::{Entity, EntityType, Hatch};
use dxf::enums::AcadVersion;
use dxf::{Drawing, Point};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Simple rectangular hatch
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // HATCH entities require R14 or later

    let rect_hatch = Hatch::new_rectangle_solid_fill(0.0, 0.0, 10.0, 5.0);
    drawing.add_entity(Entity::new(EntityType::Hatch(rect_hatch)));
    drawing.save_file("rectangle_hatch_examples.dxf")?;

    // Example 2: Circular hatch
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // HATCH entities require R14 or later

    let circle_center = Point::new(20.0, 2.5, 0.0);
    let circle_hatch = Hatch::new_circle_solid_fill(circle_center, 3.0);
    drawing.add_entity(Entity::new(EntityType::Hatch(circle_hatch)));
    drawing.save_file("circular_hatch_examples.dxf")?;

    // Example 3: Custom polygon hatch
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // HATCH entities require R14 or later

    let pentagon_points = vec![
        Point::new(40.0, 0.0, 0.0),
        Point::new(43.8, 2.9, 0.0),
        Point::new(42.4, 7.3, 0.0),
        Point::new(37.6, 7.3, 0.0),
        Point::new(36.2, 2.9, 0.0),
    ];
    let pentagon_hatch = Hatch::new_polygon_solid_fill(pentagon_points);
    drawing.add_entity(Entity::new(EntityType::Hatch(pentagon_hatch)));
    drawing.save_file("pentagon_hatch_examples.dxf")?;

    // Example 4: Rectangle with circular hole (demonstrates polygon with holes)
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // HATCH entities require R14 or later

    let outer_boundary = vec![
        Point::new(0.0, 10.0, 0.0),
        Point::new(20.0, 10.0, 0.0),
        Point::new(20.0, 25.0, 0.0),
        Point::new(0.0, 25.0, 0.0),
    ];

    let circular_hole = vec![
        Point::new(8.0, 17.5, 0.0), // approximate circle with octagon
        Point::new(10.6, 16.4, 0.0),
        Point::new(12.0, 17.5, 0.0),
        Point::new(10.6, 18.6, 0.0),
        Point::new(8.0, 17.5, 0.0),
    ];

    let mut hatch = Hatch::default();
    hatch.set_path(outer_boundary);
    hatch.set_holes(vec![circular_hole]);

    drawing.add_entity(Entity::new(EntityType::Hatch(hatch)));
    drawing.save_file("hatch_with_hole_examples.dxf")?;

    // Example 5: Complex shape with multiple holes
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R14; // HATCH entities require R14 or later

    let mut complex_hatch = Hatch::new_rectangle_solid_fill(30.0, 10.0, 50.0, 25.0);

    // // Add rectangular hole
    complex_hatch.add_rectangular_hole(32.0, 12.0, 36.0, 16.0);

    // // Add circular hole
    complex_hatch.add_circular_hole(Point::new(42.0, 20.0, 0.0), 2.0);

    // // Add triangular hole
    let triangle_hole = vec![
        Point::new(44.0, 12.0, 0.0),
        Point::new(48.0, 12.0, 0.0),
        Point::new(46.0, 16.0, 0.0),
    ];
    complex_hatch.add_hole(triangle_hole);

    drawing.add_entity(Entity::new(EntityType::Hatch(complex_hatch)));

    // Save the drawing
    drawing.save_file("complex_hatch_examples.dxf")?;

    println!("Hatch examples saved to 'hatch_examples.dxf'");
    println!("This file contains:");
    println!("1. Simple rectangle hatch at (0,0) to (10,5)");
    println!("2. Circular hatch centered at (20, 2.5) with radius 3");
    println!("3. Pentagon polygon hatch");
    println!("4. Rectangle with a hole demonstrating polygon with holes");
    println!("5. Complex rectangle with multiple different types of holes");
    println!("\nYou can open this file in any CAD software that supports DXF files.");

    Ok(())
}
