use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2 {
    pub data: [[f32; 2]; 2],
}

impl Mat2 {
    pub fn new(data: [[f32; 2]; 2]) -> Self {
        Self { data }
    }

    pub fn add(&self, other: &Self) -> Self {
        let mut result = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                result[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        Self { data: result }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let mut result = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                result[i][j] = self.data[i][j] - other.data[i][j];
            }
        }
        Self { data: result }
    }

    pub fn scale(&self, scalar: f32) -> Self {
        let mut result = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                result[i][j] = self.data[i][j] * scalar;
            }
        }
        Self { data: result }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        Self { data: result }
    }

    pub fn transpose(&self) -> Self {
        let mut result = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                result[i][j] = self.data[j][i];
            }
        }
        Self { data: result }
    }

    pub fn determinant(&self) -> f32 {
        self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            None
        } else {
            let inv_det = 1.0 / det;
            Some(Self {
                data: [
                    [self.data[1][1] * inv_det, -self.data[0][1] * inv_det],
                    [-self.data[1][0] * inv_det, self.data[0][0] * inv_det],
                ],
            })
        }
    }
}

impl Add for Mat2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add(&other)
    }
}

impl Sub for Mat2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.subtract(&other)
    }
}

impl Mul<f32> for Mat2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        self.scale(scalar)
    }
}

impl Mul for Mat2 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    pub data: [[f32; 3]; 3],
}

impl Mat3 {
    pub fn new(data: [[f32; 3]; 3]) -> Self {
        Self { data }
    }

    pub fn add(&self, other: &Self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        Self { data: result }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.data[i][j] - other.data[i][j];
            }
        }
        Self { data: result }
    }

    pub fn scale(&self, scalar: f32) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.data[i][j] * scalar;
            }
        }
        Self { data: result }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        Self { data: result }
    }

    pub fn transpose(&self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.data[j][i];
            }
        }
        Self { data: result }
    }

    pub fn determinant(&self) -> f32 {
        self.data[0][0] * (self.data[1][1] * self.data[2][2] - self.data[1][2] * self.data[2][1])
            - self.data[0][1]
                * (self.data[1][0] * self.data[2][2] - self.data[1][2] * self.data[2][0])
            + self.data[0][2]
                * (self.data[1][0] * self.data[2][1] - self.data[1][1] * self.data[2][0])
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            None
        } else {
            let inv_det = 1.0 / det;
            let mut result = [[0.0; 3]; 3];

            result[0][0] =
                (self.data[1][1] * self.data[2][2] - self.data[1][2] * self.data[2][1]) * inv_det;
            result[0][1] =
                (self.data[0][2] * self.data[2][1] - self.data[0][1] * self.data[2][2]) * inv_det;
            result[0][2] =
                (self.data[0][1] * self.data[1][2] - self.data[0][2] * self.data[1][1]) * inv_det;
            result[1][0] =
                (self.data[1][2] * self.data[2][0] - self.data[1][0] * self.data[2][2]) * inv_det;
            result[1][1] =
                (self.data[0][0] * self.data[2][2] - self.data[0][2] * self.data[2][0]) * inv_det;
            result[1][2] =
                (self.data[0][2] * self.data[1][0] - self.data[0][0] * self.data[1][2]) * inv_det;
            result[2][0] =
                (self.data[1][0] * self.data[2][1] - self.data[1][1] * self.data[2][0]) * inv_det;
            result[2][1] =
                (self.data[0][1] * self.data[2][0] - self.data[0][0] * self.data[2][1]) * inv_det;
            result[2][2] =
                (self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]) * inv_det;

            Some(Self { data: result })
        }
    }
}

impl Add for Mat3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add(&other)
    }
}

impl Sub for Mat3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.subtract(&other)
    }
}

impl Mul<f32> for Mat3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        self.scale(scalar)
    }
}

impl Mul for Mat3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub data: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn new(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }

    pub fn add(&self, other: &Self) -> Self {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        Self { data: result }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.data[i][j] - other.data[i][j];
            }
        }
        Self { data: result }
    }

    pub fn scale(&self, scalar: f32) -> Self {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.data[i][j] * scalar;
            }
        }
        Self { data: result }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        Self { data: result }
    }

    pub fn transpose(&self) -> Self {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.data[j][i];
            }
        }
        Self { data: result }
    }

    pub fn determinant(&self) -> f32 {
        // Laplace expansion along the first row
        self.data[0][0] * Mat3::from_submatrix(self, 1, 1).determinant()
            - self.data[0][1] * Mat3::from_submatrix(self, 1, 2).determinant()
            + self.data[0][2] * Mat3::from_submatrix(self, 1, 3).determinant()
            - self.data[0][3] * Mat3::from_submatrix(self, 1, 4).determinant()
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det == 0.0 {
            None
        } else {
            let inv_det = 1.0 / det;
            let mut result = [[0.0; 4]; 4];

            for i in 0..4 {
                for j in 0..4 {
                    result[j][i] = Mat3::from_submatrix(self, i + 1, j + 1).determinant()
                        * inv_det
                        * if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                }
            }

            Some(Self { data: result })
        }
    }
}

// Helper function to extract a 3x3 submatrix from a 4x4 matrix
impl Mat3 {
    fn from_submatrix(mat4: &Mat4, row: usize, col: usize) -> Self {
        let mut data = [[0.0; 3]; 3];
        let mut sub_row = 0;
        for i in 0..4 {
            if i + 1 == row {
                continue;
            }
            let mut sub_col = 0;
            for j in 0..4 {
                if j + 1 == col {
                    continue;
                }
                data[sub_row][sub_col] = mat4.data[i][j];
                sub_col += 1;
            }
            sub_row += 1;
        }
        Self { data }
    }
}

impl Add for Mat4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add(&other)
    }
}

impl Sub for Mat4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.subtract(&other)
    }
}

impl Mul<f32> for Mat4 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        self.scale(scalar)
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
}
