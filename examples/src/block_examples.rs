use dxf::entities::*;
use dxf::enums::AcadVersion;
use dxf::{Block, Drawing, Point};

pub fn all() -> dxf::DxfResult<()> {
    basic_block_and_insert()?;
    Ok(())
}

fn basic_block_and_insert() -> dxf::DxfResult<()> {
    let mut drawing = Drawing::new();
    drawing.header.version = AcadVersion::R12; // this example only tested on R12

    //
    // create a block with a unique name...
    //
    let mut block = Block::default();
    block.name = "my-block-name".to_string();

    //
    // ...and populate it with entities
    //
    block.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Line(Line::new(
            // line from (0,0) to (1,1)
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
        )),
    });

    //
    // add the block to the drawing
    //
    drawing.add_block(block);

    //
    // add a reference to the block with an `INSERT` entity
    //
    let mut insert = Insert::default();
    insert.name = "my-block-name".to_string(); // use the same name as the block defined above
    insert.location = Point::new(3.0, 3.0, 0.0); // select the base-point of the insertion
    drawing.add_entity(Entity {
        common: Default::default(),
        specific: EntityType::Insert(insert),
    }); // the end result is a line from (3,3) to (4,4)

    drawing.save_file("basic_block_and_insert.dxf")?;
    Ok(())
}
