use crate::arith_traits::*;

pub trait Vector:
    Copy
    + Sized
    + std::ops::Index<usize, Output = Self::Scalar>
    + std::ops::IndexMut<usize, Output = Self::Scalar>
{
    type Scalar: Number;
    const DIMENSION: usize;

    fn to_array(&self) -> [Self::Scalar; Self::DIMENSION] {
        unsafe { std::ptr::read(self as *const Self as *const [Self::Scalar; Self::DIMENSION]) }
    }
    fn as_slice(&self) -> &[Self::Scalar] {
        unsafe {
            std::slice::from_raw_parts(self as *const Self as *const Self::Scalar, Self::DIMENSION)
        }
    }
    fn as_mut_slice(&mut self) -> &mut [Self::Scalar] {
        unsafe {
            std::slice::from_raw_parts_mut(self as *mut Self as *mut Self::Scalar, Self::DIMENSION)
        }
    }

    fn dot(&self, other: &Self) -> Self::Scalar {
        let mut sum = Self::Scalar::default();
        for i in 0..Self::DIMENSION {
            sum += self[i] * other[i];
        }
        sum
    }
}

pub trait Crossable: Vector {
    fn cross(&self, other: &Self) -> Self;
}

pub trait HasMagnitude: Vector {
    fn length(&self) -> Self::Scalar;
    fn length_squared(&self) -> Self::Scalar {
        self.dot(self)
    }
}

pub trait Normalizable: HasMagnitude {
    fn normalize(&self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector2<T: Number> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3<T: Number> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector4<T: Number> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub type Int2 = Vector2<i32>;
pub type Int3 = Vector3<i32>;
pub type Int4 = Vector4<i32>;
pub type UInt2 = Vector2<u32>;
pub type UInt3 = Vector3<u32>;
pub type UInt4 = Vector4<u32>;
pub type Float2 = Vector2<f32>;
pub type Float3 = Vector3<f32>;
pub type Float4 = Vector4<f32>;
pub type Double2 = Vector2<f64>;
pub type Double3 = Vector3<f64>;
pub type Double4 = Vector4<f64>;

impl<T: Number> std::ops::Index<usize> for Vector2<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds for Vector2"),
        }
    }
}

impl<T: Number> std::ops::IndexMut<usize> for Vector2<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Index out of bounds for Vector2"),
        }
    }
}

impl<T: Number> std::ops::Index<usize> for Vector3<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds for Vector3"),
        }
    }
}

impl<T: Number> std::ops::IndexMut<usize> for Vector3<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds for Vector3"),
        }
    }
}

impl<T: Number> std::ops::Index<usize> for Vector4<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        assert!(index < 4, "Index out of bounds for Vector4");

        &self.as_slice()[index]
    }
}

impl<T: Number> std::ops::IndexMut<usize> for Vector4<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < 4, "Index out of bounds for Vector4");

        &mut self.as_mut_slice()[index]
    }
}

impl<T: Number> Vector for Vector2<T> {
    type Scalar = T;
    const DIMENSION: usize = 2;
}

impl<T: Number> Vector for Vector3<T> {
    type Scalar = T;
    const DIMENSION: usize = 3;
}

impl<T: Number> Crossable for Vector3<T> {
    fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl<T: Number> Vector for Vector4<T> {
    type Scalar = T;
    const DIMENSION: usize = 4;
}

impl<T: Number> std::ops::Add for Vector2<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Number> std::ops::Add for Vector3<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T: Number> std::ops::Add for Vector4<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl<T: Number> std::ops::Sub for Vector2<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Number> std::ops::Sub for Vector3<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: Number> std::ops::Sub for Vector4<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl<T: Number> std::ops::Mul<T> for Vector2<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: Number> std::ops::Mul<T> for Vector3<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl<T: Number> std::ops::Mul<T> for Vector4<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl<T: Number> std::ops::Div<T> for Vector2<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl<T: Number> std::ops::Div<T> for Vector3<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl<T: Number> std::ops::Div<T> for Vector4<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        }
    }
}

impl<V, S> HasMagnitude for V
where
    V: Vector<Scalar = S>,
    S: Number + SquareRoot,
{
    fn length(&self) -> S {
        self.length_squared().sqrt()
    }
}

impl<V, S> Normalizable for V
where
    V: Vector<Scalar = S> + HasMagnitude + std::ops::Div<S, Output = V> + Copy,
    S: Number + SquareRoot + PartialEq<f32>,
{
    fn normalize(&self) -> Self {
        let len = self.length();
        if len == 0.0 { *self } else { *self / len }
    }
}

impl<T: Number> From<[T; 2]> for Vector2<T> {
    fn from(a: [T; 2]) -> Self {
        Self { x: a[0], y: a[1] }
    }
}

impl<T: Number> From<[T; 3]> for Vector3<T> {
    fn from(a: [T; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
}

impl<T: Number> From<[T; 4]> for Vector4<T> {
    fn from(a: [T; 4]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
            w: a[3],
        }
    }
}
