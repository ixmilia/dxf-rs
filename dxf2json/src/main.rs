use dxf::Drawing;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dxf_path = &args[1];
    let mut json_path = dxf_path.clone();
    json_path.push_str(".json");

    let drawing = Drawing::load_file(&dxf_path).unwrap();
    let json = serde_json::to_string_pretty(&drawing).unwrap();

    let file = File::create(&json_path).unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(json.as_bytes()).unwrap();
}
