mod block_examples;
mod line_type_examples;

fn main() -> dxf::DxfResult<()> {
    block_examples::all()?;
    line_type_examples::all()?;
    Ok(())
}
