use crate::{Handle, Point};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum DataTableValue {
    Boolean(bool),
    Integer(i32),
    Double(f64),
    Str(String),
    Point2D(Point),
    Point3D(Point),
    Handle(Handle),
}
