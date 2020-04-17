use crate::{CodePair, DxfError, DxfResult};

/// Represents a simple vector in Cartesian space.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Vector {
    /// The X component of the vector.
    pub x: f64,
    /// The Y component of the vector.
    pub y: f64,
    /// The Z component of the vector.
    pub z: f64,
}

impl Vector {
    /// Creates a new `Vector` with the specified values.
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector { x, y, z }
    }
    /// Returns a new zero vector representing (0, 0, 0).
    pub fn zero() -> Vector {
        Vector::new(0.0, 0.0, 0.0)
    }
    /// Returns a new vector representing the X axis.
    pub fn x_axis() -> Vector {
        Vector::new(1.0, 0.0, 0.0)
    }
    /// Returns a new vector representing the Y axis.
    pub fn y_axis() -> Vector {
        Vector::new(0.0, 1.0, 0.0)
    }
    /// Returns a new vector representing the Z axis.
    pub fn z_axis() -> Vector {
        Vector::new(0.0, 0.0, 1.0)
    }
    pub(crate) fn set(&mut self, pair: &CodePair) -> DxfResult<()> {
        match pair.code {
            10 => self.x = pair.assert_f64()?,
            20 => self.y = pair.assert_f64()?,
            30 => self.z = pair.assert_f64()?,
            _ => {
                return Err(DxfError::UnexpectedCodePair(
                    pair.clone(),
                    String::from("expected code [10, 20, 30] for vector"),
                ))
            }
        }

        Ok(())
    }
}
