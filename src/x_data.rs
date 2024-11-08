use crate::{CodePair, DxfError, DxfResult, Handle, Point, Vector};

use crate::code_pair_put_back::CodePairPutBack;
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
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct XData {
    pub application_name: String,
    pub items: Vec<XDataItem>,
}

/// Represents a piece of extended data.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
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
    pub(crate) fn read_item(
        application_name: String,
        iter: &mut CodePairPutBack,
    ) -> DxfResult<XData> {
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
                iter.put_back(Ok(pair));
                break;
            }
            xdata.items.push(XDataItem::read_item(&pair, iter)?);
        }
        Ok(xdata)
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>, version: AcadVersion) {
        // not supported on < R2000
        if version >= AcadVersion::R2000 {
            pairs.push(CodePair::new_string(
                XDATA_APPLICATIONNAME,
                &self.application_name,
            ));
            for item in &self.items {
                item.add_code_pairs(pairs);
            }
        }
    }
}

impl XDataItem {
    fn read_item(pair: &CodePair, iter: &mut CodePairPutBack) -> DxfResult<XDataItem> {
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
    fn read_double(iter: &mut CodePairPutBack, expected_code: i32) -> DxfResult<f64> {
        match iter.next() {
            Some(Ok(ref pair)) if pair.code == expected_code => Ok(pair.assert_f64()?),
            Some(Ok(pair)) => Err(DxfError::UnexpectedCode(pair.code, pair.offset)),
            Some(Err(e)) => Err(e),
            None => Err(DxfError::UnexpectedEndOfInput),
        }
    }
    fn read_point(iter: &mut CodePairPutBack, first: f64, expected_code: i32) -> DxfResult<Point> {
        Ok(Point::new(
            first,
            XDataItem::read_double(iter, expected_code + 10)?,
            XDataItem::read_double(iter, expected_code + 20)?,
        ))
    }
    fn read_vector(
        iter: &mut CodePairPutBack,
        first: f64,
        expected_code: i32,
    ) -> DxfResult<Vector> {
        Ok(Vector::new(
            first,
            XDataItem::read_double(iter, expected_code + 10)?,
            XDataItem::read_double(iter, expected_code + 20)?,
        ))
    }
    pub(crate) fn add_code_pairs(&self, pairs: &mut Vec<CodePair>) {
        match self {
            XDataItem::Str(ref s) => {
                pairs.push(CodePair::new_string(XDATA_STRING, s));
            }
            XDataItem::ControlGroup(ref items) => {
                pairs.push(CodePair::new_str(XDATA_CONTROLGROUP, "{"));
                for item in &items[..] {
                    item.add_code_pairs(pairs);
                }
                pairs.push(CodePair::new_str(XDATA_CONTROLGROUP, "}"));
            }
            XDataItem::LayerName(ref l) => {
                pairs.push(CodePair::new_string(XDATA_LAYER, l));
            }
            XDataItem::BinaryData(ref data) => {
                let mut line = String::new();
                for b in data {
                    line.push_str(&format!("{:02X}", b));
                }
                pairs.push(CodePair::new_string(XDATA_BINARYDATA, &line));
            }
            XDataItem::Handle(h) => {
                pairs.push(CodePair::new_string(XDATA_HANDLE, &h.as_string()));
            }
            XDataItem::ThreeReals(x, y, z) => {
                pairs.push(CodePair::new_f64(XDATA_THREEREALS, *x));
                pairs.push(CodePair::new_f64(XDATA_THREEREALS + 10, *y));
                pairs.push(CodePair::new_f64(XDATA_THREEREALS + 20, *z));
            }
            XDataItem::WorldSpacePosition(ref p) => {
                pairs.push(CodePair::new_f64(XDATA_WORLDSPACEPOSITION, p.x));
                pairs.push(CodePair::new_f64(XDATA_WORLDSPACEPOSITION + 10, p.y));
                pairs.push(CodePair::new_f64(XDATA_WORLDSPACEPOSITION + 20, p.z));
            }
            XDataItem::WorldSpaceDisplacement(ref p) => {
                pairs.push(CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT, p.x));
                pairs.push(CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT + 10, p.y));
                pairs.push(CodePair::new_f64(XDATA_WORLDSPACEDISPLACEMENT + 20, p.z));
            }
            XDataItem::WorldDirection(ref v) => {
                pairs.push(CodePair::new_f64(XDATA_WORLDDIRECTION, v.x));
                pairs.push(CodePair::new_f64(XDATA_WORLDDIRECTION + 10, v.y));
                pairs.push(CodePair::new_f64(XDATA_WORLDDIRECTION + 20, v.z));
            }
            XDataItem::Real(f) => {
                pairs.push(CodePair::new_f64(XDATA_REAL, *f));
            }
            XDataItem::Distance(f) => {
                pairs.push(CodePair::new_f64(XDATA_DISTANCE, *f));
            }
            XDataItem::ScaleFactor(f) => {
                pairs.push(CodePair::new_f64(XDATA_SCALEFACTOR, *f));
            }
            XDataItem::Integer(i) => {
                pairs.push(CodePair::new_i16(XDATA_INTEGER, *i));
            }
            XDataItem::Long(i) => {
                pairs.push(CodePair::new_i32(XDATA_LONG, *i));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::code_pair::CodePair;
    use crate::enums::AcadVersion;
    use crate::helper_functions::tests::*;
    use crate::objects::*;
    use crate::x_data::{XData, XDataItem};
    use crate::{Drawing, Point, Vector};

    fn read_x_data_items(values: Vec<CodePair>) -> Vec<XDataItem> {
        let mut pairs = vec![
            CodePair::new_str(0, "ACAD_PROXY_OBJECT"),
            CodePair::new_str(1001, "TEST_APPLICATION_NAME"),
        ];
        for pair in values {
            pairs.push(pair);
        }
        let drawing = from_section("OBJECTS", pairs);
        let objects = drawing.objects().collect::<Vec<_>>();
        assert_eq!(1, objects.len());
        assert_eq!(1, objects[0].common.x_data.len());
        let x_data = objects[0].common.x_data[0].clone();
        assert_eq!("TEST_APPLICATION_NAME", x_data.application_name);
        x_data.items
    }

    fn read_x_data_item(values: Vec<CodePair>) -> XDataItem {
        let items = read_x_data_items(values);
        assert_eq!(1, items.len());
        items[0].clone()
    }

    fn write_x_data_item(item: XDataItem) -> Vec<CodePair> {
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
        drawing.code_pairs().unwrap()
    }

    #[test]
    fn read_3_reals() {
        let item = read_x_data_item(vec![
            CodePair::new_f64(1010, 1.0),
            CodePair::new_f64(1020, 2.0),
            CodePair::new_f64(1030, 3.0),
        ]);
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
        let item = read_x_data_item(vec![
            CodePair::new_f64(1011, 1.0),
            CodePair::new_f64(1021, 2.0),
            CodePair::new_f64(1031, 3.0),
        ]);
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
        let item = read_x_data_item(vec![
            CodePair::new_f64(1012, 1.0),
            CodePair::new_f64(1022, 2.0),
            CodePair::new_f64(1032, 3.0),
        ]);
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
        let item = read_x_data_item(vec![
            CodePair::new_f64(1013, 1.0),
            CodePair::new_f64(1023, 2.0),
            CodePair::new_f64(1033, 3.0),
        ]);
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
            CodePair::new_str(1001, "TEST_APPLICATION_NAME"),
            CodePair::new_f64(1010, 1.0),
            CodePair::new_f64(1020, 2.0),
            CodePair::new_f64(1030, 3.0),
            CodePair::new_str(0, "ENDSEC"),
        ];
        assert_vec_contains(&actual, &expected);
    }

    #[test]
    fn write_world_space_position() {
        let actual = write_x_data_item(XDataItem::WorldSpacePosition(Point::new(1.0, 2.0, 3.0)));
        let expected = vec![
            CodePair::new_str(1001, "TEST_APPLICATION_NAME"),
            CodePair::new_f64(1011, 1.0),
            CodePair::new_f64(1021, 2.0),
            CodePair::new_f64(1031, 3.0),
            CodePair::new_str(0, "ENDSEC"),
        ];
        assert_vec_contains(&actual, &expected);
    }

    #[test]
    fn write_world_space_displacement() {
        let actual =
            write_x_data_item(XDataItem::WorldSpaceDisplacement(Point::new(1.0, 2.0, 3.0)));
        let expected = vec![
            CodePair::new_str(1001, "TEST_APPLICATION_NAME"),
            CodePair::new_f64(1012, 1.0),
            CodePair::new_f64(1022, 2.0),
            CodePair::new_f64(1032, 3.0),
            CodePair::new_str(0, "ENDSEC"),
        ];
        assert_vec_contains(&actual, &expected);
    }

    #[test]
    fn write_world_direction() {
        let actual = write_x_data_item(XDataItem::WorldDirection(Vector::new(1.0, 2.0, 3.0)));
        let expected = vec![
            CodePair::new_str(1001, "TEST_APPLICATION_NAME"),
            CodePair::new_f64(1013, 1.0),
            CodePair::new_f64(1023, 2.0),
            CodePair::new_f64(1033, 3.0),
            CodePair::new_str(0, "ENDSEC"),
        ];
        assert_vec_contains(&actual, &expected);
    }
}
