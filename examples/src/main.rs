extern crate dxf;

mod line_type_examples;

fn main() -> dxf::DxfResult<()> {
    line_type_examples::all()?;
    Ok(())
}
