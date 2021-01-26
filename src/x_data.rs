use std::io::{Read, Write};

use crate::{CodePair, DxfError, DxfResult, Handle, Point, Vector};

use crate::code_pair_put_back::CodePairPutBack;
use crate::code_pair_writer::CodePairWriter;
use crate::enums::AcadVersion;
use crate::helper_functions::*;

pub(crate) const XDATA_APPLICATIONNAME: i32 = 1001;
const XDATA_STRING: i32 = 1000;
const XDATA_CONTROLGROUP: i32 = 1002;
const XDATA_LAYER: i32 = 1003;
const XDATA_BINARYDATA: i32 = 1004;
const XDATA_HANDLE: i32 = 1005;
const XDATA_THREEREALS: i32 = 1010;
const XDATA_WORLDSPACEPOSITION: i32 = 1011;
const XDATA_WORLDSPACEDISPLACEMENT: i32 = 1012;
const XDATA_WORLDDIRECTION: i32 = 1013;
const XDATA_REAL: i32 = 1040;
const XDATA_DISTANCE: i32 = 1041;
const XDATA_SCALEFACTOR: i32 = 1042;
const XDATA_INTEGER: i32 = 1070;
const XDATA_LONG: i32 = 1071;

/// Represents an application name and a collection of extended data.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct XData {
    pub application_name: String,
    pub items: Vec<XDataItem>,
}

/// Represents a piece of extended data.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum XDataItem {
    Str(String),
    ControlGroup(Vec<XDataItem>),
    LayerName(String),
    BinaryData(Vec<u8>),
    Handle(Handle),
    ThreeReals(f64, f64, f64),
    WorldSpacePosition(Point),
    WorldSpaceDisplacement(Point),
    WorldDirection(Vector),
    Real(f64),
    Distance(f64),
    ScaleFactor(f64),
    Integer(i16),
    Long(i32),
}

impl XData {
    pub(crate) fn read_item<I>(
        application_name: String,
        iter: &mut CodePairPutBack<I>,
    ) -> DxfResult<XData>
    where
        I: Read,
    {
        let mut xdata = XData {
            application_name,
            items: vec![],
        };
        loop {
            let pair = match iter.next() {
                Some(Ok(pair @ CodePair { code: 0, .. })) => {
                    iter.put_back(Ok(pair));
                    return Ok(xdata);
                }
                Some(Ok(pair)) => pair,
                Some(Err(e)) => return Err(e),
                None => return Ok(xdata),
            };
            if pair.code == XDATA_APPLICATIONNAME || pair.code < XDATA_STRING {
                // new xdata or non xdata
                break;
            }
            xdata.items.push(XDataItem::read_item(&pair, iter)?);
        }
        Ok(xdata)
    }
    pub(crate) fn write<T>(
        &self,
        version: AcadVersion,
        writer: &mut CodePairWriter<T>,
    ) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        // not supported on < R2000
        if version >= AcadVersion::R2000 {
            writer.write_code_pair(&CodePair::new_string(
                XDATA_APPLICATIONNAME,
                &self.application_name,
            ))?;
            for item in &self.items {
                item.write(writer)?;
            }
        }
        Ok(())
    }
}

impl XDataItem {
    fn read_item<I>(pair: &CodePair, iter: &mut CodePairPutBack<I>) -> DxfResult<XDataItem>
    where
        I: Read,
    {
        match pair.code {
            XDATA_STRING => Ok(XDataItem::Str(pair.assert_string()?)),
            XDATA_CONTROLGROUP => {
                let mut items = vec![];
                loop {
                    let pair = match iter.next() {
                        Some(Ok(pair)) => {
                            if pair.code < XDATA_STRING {
                                return Err(DxfError::UnexpectedCodePair(
                                    pair,
                                    String::from("expected XDATA item"),
                                ));
                            }
                            pair
                        }
                        Some(Err(e)) => return Err(e),
                        None => return Err(DxfError::UnexpectedEndOfInput),
                    };
                    if pair.code == XDATA_CONTROLGROUP && pair.assert_string()? == "}" {
                        // end of group
                        break;
                    }

                    items.push(XDataItem::read_item(&pair, iter)?);
                }
                Ok(XDataItem::ControlGroup(items))
            }
            XDATA_LAYER => Ok(XDataItem::LayerName(pair.assert_string()?)),
            XDATA_BINARYDATA => {
                let mut data = vec![];
                parse_hex_string(&pair.assert_string()?, &mut data, pair.offset)?;
                Ok(XDataItem::BinaryData(data))
            }
            XDATA_HANDLE => Ok(XDataItem::Handle(pair.as_handle()?)),
            XDATA_THREEREALS => Ok(XDataItem::ThreeReals(
                pair.assert_f64()?,
                XDataItem::read_double(iter, pair.code + 10)?,
                XDataItem::read_double(iter, pair.code + 20)?,
            )),
            XDATA_WORLDSPACEDISPLACEMENT => Ok(XDataItem::WorldSpaceDisplacement(
                XDataItem::read_point(iter, pair.assert_f64()?, pair.code)?,
            )),
            XDATA_WORLDSPACEPOSITION => Ok(XDataItem::WorldSpacePosition(XDataItem::read_point(
                iter,
                pair.assert_f64()?,
                pair.code,
            )?)),
            XDATA_WORLDDIRECTION => Ok(XDataItem::WorldDirection(XDataItem::read_vector(
                iter,
                pair.assert_f64()?,
                pair.code,
            )?)),
            XDATA_REAL => Ok(XDataItem::Real(pair.assert_f64()?)),
            XDATA_DISTANCE => Ok(XDataItem::Distance(pair.assert_f64()?)),
            XDATA_SCALEFACTOR => Ok(XDataItem::ScaleFactor(pair.assert_f64()?)),
            XDATA_INTEGER => Ok(XDataItem::Integer(pair.assert_i16()?)),
            XDATA_LONG => Ok(XDataItem::Long(pair.assert_i32()?)),
            _ => Err(DxfError::UnexpectedCode(pair.code, pair.offset)),
        }
    }
    fn read_double<T>(iter: &mut CodePairPutBack<T>, expected_code: i32) -> DxfResult<f64>
    where
        T: Read,
    {
        match iter.next() {
            Some(Ok(ref pair)) if pair.code == expected_code => Ok(pair.assert_f64()?),
            Some(Ok(pair)) => Err(DxfError::UnexpectedCode(pair.code, pair.offset)),
            Some(Err(e)) => Err(e),
            None => Err(DxfError::UnexpectedEndOfInput),
        }
    }
    fn read_point<I>(
        iter: &mut CodePairPutBack<I>,
        first: f64,
        expected_code: i32,
    ) -> DxfResult<Point>
    where
        I: Read,
    {
        Ok(Point::new(
            first,
            XDataItem::read_double(iter, expected_code + 10)?,
            XDataItem::read_double(iter, expected_code + 20)?,
        ))
    }
    fn read_vector<I>(
        iter: &mut CodePairPutBack<I>,
        first: f64,
        expected_code: i32,
    ) -> DxfResult<Vector>
    where
        I: Read,
    {
        Ok(Vector::new(
            first,
            XDataItem::read_double(iter, expected_code + 10)?,
            XDataItem::read_double(iter, expected_code + 20)?,
        ))
    }
    pub(crate) fn write<T>(&self, writer: &mut CodePairWriter<T>) -> DxfResult<()>
    where
        T: Write + ?Sized,
    {
        match self {
            XDataItem::Str(ref s) => {
                writer.write_code_pair(&CodePair::new_string(XDATA_STRING, s))?;
            }
            XDataItem::ControlGroup(ref items) => {
                writer.write_code_pair(&CodePair::new_str(XDATA_CONTROLGROUP, "{"))?;
                for item in &items[..] {
                    item.write(writer)?;
                }
                writer.write_code_pair(&CodePair::new_str(XDATA_CONTROLGROUP, "}"))?;
            }
            XDataItem::LayerName(ref l) => {
                writer.write_code_pair(&CodePair::new_string(XDATA_LAYER, l))?;
            }
            XDataItem::BinaryData(ref data) => {
                let mut line = String::new();
                for b in data {
                    line.push_str(&format!("{:02X}", b));
                }
                writer.write_code_pair(&CodePair::new_string(XDATA_BINARYDATA, &line))?;
            }
            XDataItem::Handle(h) => {
                writer.write_code_pair(&CodePair::new_string(XDATA_HANDLE, &h.as_string()))?;
            }
            XDataItem::ThreeReals(x, y, z) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_THREEREALS, *x))?;
                writer.write_code_pair(&CodePair::new_f64(XDATA_THREEREALS + 10, *y))?;
                writer.write_code_pair(&CodePair::new_f64(XDATA_THREEREALS + 20, *z))?;
            }
            XDataItem::WorldSpacePosition(ref p) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEPOSITION, p.x))?;
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEPOSITION + 10, p.y))?;
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEPOSITION + 20, p.z))?;
            }
            XDataItem::WorldSpaceDisplacement(ref p) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT, p.x))?;
                writer
                    .write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT + 10, p.y))?;
                writer
                    .write_code_pair(&CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT + 20, p.z))?;
            }
            XDataItem::WorldDirection(ref v) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDDIRECTION, v.x))?;
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDDIRECTION + 10, v.y))?;
                writer.write_code_pair(&CodePair::new_f64(XDATA_WORLDDIRECTION + 20, v.z))?;
            }
            XDataItem::Real(f) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_REAL, *f))?;
            }
            XDataItem::Distance(f) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_DISTANCE, *f))?;
            }
            XDataItem::ScaleFactor(f) => {
                writer.write_code_pair(&CodePair::new_f64(XDATA_SCALEFACTOR, *f))?;
            }
            XDataItem::Integer(i) => {
                writer.write_code_pair(&CodePair::new_i16(XDATA_INTEGER, *i))?;
            }
            XDataItem::Long(i) => {
                writer.write_code_pair(&CodePair::new_i32(XDATA_LONG, *i))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::AcadVersion;
    use crate::helper_functions::tests::*;
    use crate::objects::*;
    use crate::x_data::{XData, XDataItem};
    use crate::{Drawing, Point, Vector};

    fn read_x_data_items(lines: Vec<&str>) -> Vec<XDataItem> {
        let drawing = from_section(
            "OBJECTS",
            vec![
                "0",
                "ACAD_PROXY_OBJECT",
                "1001",
                "TEST_APPLICATION_NAME",
                lines.join("\r\n").as_str(),
            ]
            .join("\r\n")
            .as_str(),
        );
        let objects = drawing.objects().collect::<Vec<_>>();
        assert_eq!(1, objects.len());
        assert_eq!(1, objects[0].common.x_data.len());
        let x_data = objects[0].common.x_data[0].clone();
        assert_eq!("TEST_APPLICATION_NAME", x_data.application_name);
        x_data.items
    }

    fn read_x_data_item(lines: Vec<&str>) -> XDataItem {
        let items = read_x_data_items(lines);
        assert_eq!(1, items.len());
        items[0].clone()
    }

    fn write_x_data_item(item: XDataItem) -> String {
        let mut drawing = Drawing::new();
        drawing.header.version = AcadVersion::R2000;
        drawing.add_object(Object {
            common: ObjectCommon {
                x_data: vec![XData {
                    application_name: String::from("TEST_APPLICATION_NAME"),
                    items: vec![item],
                }],
                ..Default::default()
            },
            specific: ObjectType::AcadProxyObject(AcadProxyObject::default()),
        });
        to_test_string(&drawing)
    }

    #[test]
    fn read_3_reals() {
        let item = read_x_data_item(vec!["1010", "1.0", "1020", "2.0", "1030", "3.0"]);
        match item {
            XDataItem::ThreeReals(x, y, z) => {
                assert_eq!(1.0, x);
                assert_eq!(2.0, y);
                assert_eq!(3.0, z);
            }
            _ => panic!("expected 3 reals"),
        }
    }

    #[test]
    fn read_world_space_position() {
        let item = read_x_data_item(vec!["1011", "1.0", "1021", "2.0", "1031", "3.0"]);
        match item {
            XDataItem::WorldSpacePosition(p) => {
                assert_eq!(1.0, p.x);
                assert_eq!(2.0, p.y);
                assert_eq!(3.0, p.z);
            }
            _ => panic!("expected 3 reals"),
        }
    }

    #[test]
    fn read_world_space_displacement() {
        let item = read_x_data_item(vec!["1012", "1.0", "1022", "2.0", "1032", "3.0"]);
        match item {
            XDataItem::WorldSpaceDisplacement(p) => {
                assert_eq!(1.0, p.x);
                assert_eq!(2.0, p.y);
                assert_eq!(3.0, p.z);
            }
            _ => panic!("expected 3 reals"),
        }
    }

    #[test]
    fn read_world_direction() {
        let item = read_x_data_item(vec!["1013", "1.0", "1023", "2.0", "1033", "3.0"]);
        match item {
            XDataItem::WorldDirection(v) => {
                assert_eq!(1.0, v.x);
                assert_eq!(2.0, v.y);
                assert_eq!(3.0, v.z);
            }
            _ => panic!("expected 3 reals"),
        }
    }

    #[test]
    fn write_3_reals() {
        let actual = write_x_data_item(XDataItem::ThreeReals(1.0, 2.0, 3.0));
        let expected = vec![
            "1001",
            "TEST_APPLICATION_NAME",
            "1010",
            "1.0",
            "1020",
            "2.0",
            "1030",
            "3.0",
            "  0",
            "ENDSEC",
        ]
        .join("\r\n");
        assert!(actual.contains(&expected));
    }

    #[test]
    fn write_world_space_position() {
        let actual = write_x_data_item(XDataItem::WorldSpacePosition(Point::new(1.0, 2.0, 3.0)));
        let expected = vec![
            "1001",
            "TEST_APPLICATION_NAME",
            "1011",
            "1.0",
            "1021",
            "2.0",
            "1031",
            "3.0",
            "  0",
            "ENDSEC",
        ]
        .join("\r\n");
        assert!(actual.contains(&expected));
    }

    #[test]
    fn write_world_space_displacement() {
        let actual =
            write_x_data_item(XDataItem::WorldSpaceDisplacement(Point::new(1.0, 2.0, 3.0)));
        let expected = vec![
            "1001",
            "TEST_APPLICATION_NAME",
            "1012",
            "1.0",
            "1022",
            "2.0",
            "1032",
            "3.0",
            "  0",
            "ENDSEC",
        ]
        .join("\r\n");
        assert!(actual.contains(&expected));
    }

    #[test]
    fn write_world_direction() {
        let actual = write_x_data_item(XDataItem::WorldDirection(Vector::new(1.0, 2.0, 3.0)));
        let expected = vec![
            "1001",
            "TEST_APPLICATION_NAME",
            "1013",
            "1.0",
            "1023",
            "2.0",
            "1033",
            "3.0",
            "  0",
            "ENDSEC",
        ]
        .join("\r\n");
        assert!(actual.contains(&expected));
    }
}
