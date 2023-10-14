mod entity_generator;
mod header_generator;
mod object_generator;
mod other_helpers;
mod table_generator;
mod test_helper_generator;
mod xml_helpers;

use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

include!("../src/expected_type.rs");

fn main() -> Result<(), Box<dyn Error>> {
    // watch everything under the `build/` and `spec/` directories and also one specific file
    rerun_if_changed("src/expected_type.rs");
    let dirs_to_watch = vec!["build/", "spec/"];
    for sub_dir in dirs_to_watch {
        rerun_if_changed(sub_dir);
        let path = Path::new(&sub_dir);
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                rerun_if_changed(&entry.path().as_os_str());
            }
        }
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_dir = Path::new(&out_dir).join("generated");
    let _ = std::fs::create_dir(&generated_dir); // might fail if it's already there

    let mut file = File::create(generated_dir.join("mod.rs")).ok().unwrap();
    file.write_all("// The contents of this file are automatically generated and should not be modified directly.  See the `build` directory.

pub mod entities;
pub mod header;
pub mod objects;
pub mod tables;
".as_bytes()).ok().unwrap();

    entity_generator::generate_entities(&generated_dir);
    header_generator::generate_header(&generated_dir);
    object_generator::generate_objects(&generated_dir);
    table_generator::generate_tables(&generated_dir);

    test_helper_generator::generate_test_helpers(&generated_dir);

    Ok(())
}

fn rerun_if_changed<S: Debug + ?Sized>(s: &S) {
    let s = format!("{:?}", s)
        .replace("\\\\", "/") // normalize directory separators
        .replace('\"', ""); // ignore surrounding quotes
    println!("cargo:rerun-if-changed={}", s);
}
