use crate::vectors::*;

pub trait Matrix: Copy + Sized {
    type VectorType: Vector<Scalar = f32>;
    const ROWS: usize;
    const COLS: usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Matrix4 {
    pub m: [[f32; 4]; 4],
}

impl Matrix4 {
    pub const fn identity() -> Self {
        let mut m = Self { m: [[0.0; 4]; 4] };
        let mut i = 0;
        while i < 4 {
            m.m[i][i] = 1.0;
            i += 1;
        }
        m
    }
}

impl Matrix for Matrix4 {
    type VectorType = Vector4<f32>;
    const ROWS: usize = 4;
    const COLS: usize = 4;
}

impl std::ops::Mul for Matrix4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut result = Self { m: [[0.0; 4]; 4] };

        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = self.m[i][0] * other.m[0][j]
                    + self.m[i][1] * other.m[1][j]
                    + self.m[i][2] * other.m[2][j]
                    + self.m[i][3] * other.m[3][j];
            }
        }

        result
    }
}

impl std::ops::Mul<Vector4<f32>> for Matrix4 {
    type Output = Vector4<f32>;

    fn mul(self, vector: Vector4<f32>) -> Vector4<f32> {
        Vector4 {
            x: self.m[0][0] * vector.x
                + self.m[0][1] * vector.y
                + self.m[0][2] * vector.z
                + self.m[0][3] * vector.w,
            y: self.m[1][0] * vector.x
                + self.m[1][1] * vector.y
                + self.m[1][2] * vector.z
                + self.m[1][3] * vector.w,
            z: self.m[2][0] * vector.x
                + self.m[2][1] * vector.y
                + self.m[2][2] * vector.z
                + self.m[2][3] * vector.w,
            w: self.m[3][0] * vector.x
                + self.m[3][1] * vector.y
                + self.m[3][2] * vector.z
                + self.m[3][3] * vector.w,
        }
    }
}

impl std::ops::Mul<Matrix4> for Vector4<f32> {
    type Output = Vector4<f32>;

    fn mul(self, matrix: Matrix4) -> Vector4<f32> {
        Vector4 {
            x: self.x * matrix.m[0][0]
                + self.y * matrix.m[1][0]
                + self.z * matrix.m[2][0]
                + self.w * matrix.m[3][0],
            y: self.x * matrix.m[0][1]
                + self.y * matrix.m[1][1]
                + self.z * matrix.m[2][1]
                + self.w * matrix.m[3][1],
            z: self.x * matrix.m[0][2]
                + self.y * matrix.m[1][2]
                + self.z * matrix.m[2][2]
                + self.w * matrix.m[3][2],
            w: self.x * matrix.m[0][3]
                + self.y * matrix.m[1][3]
                + self.z * matrix.m[2][3]
                + self.w * matrix.m[3][3],
        }
    }
}

impl std::ops::Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &f32 {
        &self.m[index.0][index.1]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix4 {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut f32 {
        &mut self.m[index.0][index.1]
    }
}

impl std::ops::Index<usize> for Matrix4 {
    type Output = <Matrix4 as Matrix>::VectorType;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.m[index].as_ptr().cast::<Self::Output>() }
    }
}
