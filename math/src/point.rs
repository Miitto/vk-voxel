pub use crate::arith_traits::*;
pub use crate::vectors::{Vector2, Vector3};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point2<T: Number> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point3<T: Number> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Number> Point2<T> {
    pub fn to_vector(self) -> Vector2<T> {
        Vector2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl<T: Number> Point3<T> {
    pub fn to_vector(self) -> Vector3<T> {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl<T: Number> std::ops::Sub for Point2<T> {
    type Output = Vector2<T>;

    fn sub(self, other: Self) -> Vector2<T> {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Number> std::ops::Sub for Point3<T> {
    type Output = Vector3<T>;

    fn sub(self, other: Self) -> Vector3<T> {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: Number> std::ops::Add<Vector2<T>> for Point2<T> {
    type Output = Self;

    fn add(self, vector: Vector2<T>) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }
}

impl<T: Number> std::ops::Add<Vector3<T>> for Point3<T> {
    type Output = Self;

    fn add(self, vector: Vector3<T>) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
            z: self.z + vector.z,
        }
    }
}

impl<T: Number> From<Vector2<T>> for Point2<T> {
    fn from(vector: Vector2<T>) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
        }
    }
}

impl<T: Number> From<Vector3<T>> for Point3<T> {
    fn from(vector: Vector3<T>) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
        }
    }
}

impl<T: Number> From<[T; 2]> for Point2<T> {
    fn from(a: [T; 2]) -> Self {
        Self { x: a[0], y: a[1] }
    }
}

impl<T: Number> From<[T; 3]> for Point3<T> {
    fn from(a: [T; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
}
