use crate::{entities::*, enums::*, *};
use glob::glob;
use std::{
    fs::{create_dir_all, read_to_string, remove_dir_all},
    path::Path,
    process::Command,
    thread::panicking,
    time::SystemTime,
};

struct Oda {
    temp_path: String,
    input_path: String,
    output_path: String,
    oda_path: String,
}

impl Drop for Oda {
    fn drop(&mut self) {
        if panicking() {
            println!("Test drawings at '{}'", self.temp_path);
        } else {
            remove_dir_all(Path::new(&self.temp_path)).unwrap();
        }
    }
}

impl Oda {
    pub fn convert_drawing(&self, drawing: &Drawing, version: AcadVersion) -> Drawing {
        drawing
            .save_file(&format!("{}/drawing.dxf", self.input_path))
            .unwrap();
        // e.g.,
        //   ODAFileConverter.exe input_dir output_dir ACAD2000 DXF 0 1
        let mut oda_convert = Command::new(&self.oda_path)
            .arg(&self.input_path)
            .arg(&self.output_path)
            .arg(&Oda::version_string(version))
            .arg("DXF")
            .arg("0") // recurse
            .arg("1") // audit
            .spawn()
            .expect("Failed to spawn ODA converter");
        let exit_code = oda_convert
            .wait()
            .expect("Failed to wait for ODA converter");
        let mut error_messages = String::from("");
        for entry in glob(format!("{}/*.err", &self.output_path).as_str())
            .expect("failed to glob for ODA error logs")
        {
            match entry {
                Ok(path) if path.is_file() => {
                    error_messages.push_str(format!("{}:\n", path.to_str().unwrap()).as_str());
                    let error_contents = read_to_string(path).unwrap();
                    error_messages.push_str(error_contents.as_str());
                    error_messages.push_str("\n\n");
                }
                Ok(_) => (),
                Err(_) => (),
            }
        }
        if !error_messages.is_empty() {
            panic!("Error converting files:\n{}", error_messages);
        }

        assert!(exit_code.success());
        Drawing::load_file(&format!("{}/drawing.dxf", self.output_path)).unwrap()
    }
    fn version_string(version: AcadVersion) -> String {
        let s = match version {
            AcadVersion::R9 => "ACAD9",
            AcadVersion::R10 => "ACAD10",
            AcadVersion::R12 => "ACAD12",
            AcadVersion::R13 => "ACAD13",
            AcadVersion::R14 => "ACAD14",
            AcadVersion::R2000 => "ACAD2000",
            AcadVersion::R2004 => "ACAD2004",
            AcadVersion::R2007 => "ACAD2007",
            AcadVersion::R2010 => "ACAD2010",
            AcadVersion::R2013 => "ACAD2013",
            AcadVersion::R2018 => "ACAD2018",
            _ => panic!("Unsupported ODA version {}", version),
        };
        String::from(s)
    }
}

macro_rules! require_oda {
    () => {{
        // Find ODA converter.  Final path looks something like:
        //   C:\Program Files\ODA\ODAFileConverter_title 20.12.0\ODAFileConverter.exe
        let mut full_oda_path = None;
        for entry in glob("C:/Program Files/ODA/ODAFileConverter*/ODAFileConverter.exe")
            .expect("failed to glob for ODA converter")
        {
            match entry {
                Ok(path) if path.is_file() => full_oda_path = Some(path),
                Ok(_) => (),
                Err(_) => (),
            }
        }
        if full_oda_path == None {
            // didn't find ODA converter directory
            return;
        }
        let full_oda_path = full_oda_path.unwrap();

        // make and report temporary directory
        let nanos = format!(
            "{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_path = format!("{}/integration-tests/{}", env!("OUT_DIR"), nanos);
        let input_path = format!("{}/input", temp_path);
        let output_path = format!("{}/output", temp_path);
        create_dir_all(Path::new(&temp_path)).unwrap();
        create_dir_all(Path::new(&input_path)).unwrap();
        create_dir_all(Path::new(&output_path)).unwrap();
        Oda {
            temp_path,
            input_path,
            output_path,
            oda_path: String::from(full_oda_path.to_str().unwrap()),
        }
    }};
}

#[test]
fn simple_line_can_be_read_by_oda() {
    let oda = require_oda!();
    let mut drawing = Drawing::new();
    drawing.add_entity(Entity {
        common: Default::default(),
        specific: EntityType::Line(Line {
            p1: Point::new(1.0, 2.0, 3.0),
            p2: Point::new(4.0, 5.0, 6.0),
            ..Default::default()
        }),
    });

    let round_tripped = oda.convert_drawing(&drawing, AcadVersion::R2000);
    let entities = round_tripped.entities().collect::<Vec<_>>();
    assert_eq!(1, entities.len());
    match entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(1.0, 2.0, 3.0), line.p1);
            assert_eq!(Point::new(4.0, 5.0, 6.0), line.p2);
        }
        _ => panic!("expected a line"),
    }
}
