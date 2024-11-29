use crate::entities::*;
use crate::enums::*;
use crate::*;

use std::fs::{create_dir_all, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::panicking;
use std::time::SystemTime;

use glob::glob;

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
    pub fn convert_drawing(&self, drawing: &mut Drawing, version: AcadVersion) -> Drawing {
        drawing.header.version = version;
        drawing
            .save_file(format!("{}/drawing.dxf", self.input_path))
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
        Drawing::load_file(format!("{}/drawing.dxf", self.output_path)).unwrap()
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

struct AutoCAD {
    temp_path: String,
    acad_path: String,
}

impl Drop for AutoCAD {
    fn drop(&mut self) {
        if panicking() {
            println!("Test drawings at '{}'", self.temp_path);
        } else {
            remove_dir_all(Path::new(&self.temp_path)).unwrap();
        }
    }
}

impl AutoCAD {
    pub fn convert_drawing(&self, drawing: &mut Drawing, version: AcadVersion) -> Drawing {
        drawing.header.version = version;
        drawing
            .save_file(format!("{}/input.dxf", self.temp_path))
            .unwrap();
        // e.g.,
        //   accoreconsole.exe /i /path/to/input.dxf /b script.scr
        let mut input_file = PathBuf::new();
        input_file.push(&self.temp_path);
        input_file.push("input.dxf");
        let input_file = input_file.to_str().unwrap();

        let mut output_file = PathBuf::new();
        output_file.push(&self.temp_path);
        output_file.push("output.dxf");
        let output_file = output_file.to_str().unwrap();

        let mut script_contents = String::new();
        script_contents.push_str(&format!(
            "DXFOUT \"{}\" V {} 16\n",
            output_file,
            &AutoCAD::version_string(version),
        ));
        script_contents.push_str("QUIT Y\n");
        let mut script_path = PathBuf::new();
        script_path.push(&self.temp_path);
        script_path.push("script.scr");
        let script_path = script_path.to_str().unwrap();
        write(script_path, &script_contents).expect("failed to write script file");

        let mut acad_convert = Command::new(&self.acad_path)
            .arg("/i")
            .arg(input_file)
            .arg("/s")
            .arg(script_path)
            .spawn()
            .expect("Failed to spawn acad");
        let exit_code = acad_convert.wait().expect("Failed to wait for acad");
        let mut error_messages = String::from("");
        for entry in glob(format!("{}/*.err", &self.temp_path).as_str())
            .expect("failed to glob for acad error logs")
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
        Drawing::load_file(format!("{}/output.dxf", self.temp_path)).unwrap()
    }
    fn version_string(version: AcadVersion) -> String {
        let s = match version {
            AcadVersion::R12 => "R12",
            AcadVersion::R2000 => "2000",
            AcadVersion::R2004 => "2004",
            AcadVersion::R2007 => "2007",
            AcadVersion::R2010 => "2010",
            AcadVersion::R2013 => "2013",
            AcadVersion::R2018 => "2018",
            _ => panic!("Unsupported acad version {}", version),
        };
        String::from(s)
    }
}

macro_rules! require_acad {
    () => {{
        // Find AutoCAD.  Final path looks something like:
        //   C:\Program Files\Autodesk\AutoCAD 2016\acad.exe
        let mut full_acad_path = None;
        for entry in glob("C:/Program Files/Autodesk/AutoCAD */accoreconsole.exe")
            .expect("failed to glob for acad.exe")
        {
            match entry {
                Ok(path) if path.is_file() => full_acad_path = Some(path),
                Ok(_) => (),
                Err(_) => (),
            }
        }
        if full_acad_path == None {
            // didn't find AutoCAD directory
            return;
        }
        let full_acad_path = full_acad_path.unwrap();

        // make and report temporary directory
        let nanos = format!(
            "{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_path = format!("{}/integration-tests/{}", env!("OUT_DIR"), nanos);
        create_dir_all(Path::new(&temp_path)).unwrap();
        AutoCAD {
            temp_path,
            acad_path: String::from(full_acad_path.to_str().unwrap()),
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

    let round_tripped = oda.convert_drawing(&mut drawing, AcadVersion::R2000);
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

#[test]
fn simple_line_can_be_read_by_acad_r12() {
    let acad = require_acad!();
    let mut drawing = Drawing::new();
    drawing.add_entity(Entity {
        common: Default::default(),
        specific: EntityType::Line(Line {
            p1: Point::new(1.0, 2.0, 3.0),
            p2: Point::new(4.0, 5.0, 6.0),
            ..Default::default()
        }),
    });

    let round_tripped = acad.convert_drawing(&mut drawing, AcadVersion::R12);
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

#[test]
fn block_insert_can_be_read_by_acad_r12() {
    let acad = require_acad!();
    let mut drawing = Drawing::new();
    let mut block = Block {
        name: "my-block-name".to_string(),
        ..Default::default()
    };
    block.entities.push(Entity {
        common: Default::default(),
        specific: EntityType::Line(Line::new(
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
        )),
    });
    drawing.add_block(block);
    let mut insert = Insert {
        name: "my-block-name".to_string(),
        ..Default::default()
    };
    insert.location = Point::new(3.0, 3.0, 0.0);
    drawing.add_entity(Entity {
        common: Default::default(),
        specific: EntityType::Insert(insert),
    });

    let round_tripped = acad.convert_drawing(&mut drawing, AcadVersion::R12);
    let entities = round_tripped.entities().collect::<Vec<_>>();

    // verify insert
    assert_eq!(1, entities.len());
    match entities[0].specific {
        EntityType::Insert(ref insert) => {
            assert_eq!("my-block-name", insert.name.to_lowercase());
        }
        _ => panic!("expected an insert"),
    }

    // verify line
    let block = round_tripped
        .blocks()
        .filter(|b| b.name.to_lowercase() == "my-block-name")
        .collect::<Vec<_>>()[0];
    assert_eq!(1, block.entities.len());
    match block.entities[0].specific {
        EntityType::Line(ref line) => {
            assert_eq!(Point::new(0.0, 0.0, 0.0), line.p1);
            assert_eq!(Point::new(1.0, 1.0, 0.0), line.p2);
        }
        _ => panic!("expected a line"),
    }
}
