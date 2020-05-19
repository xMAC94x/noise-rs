use crate::math::{Vector2, Vector3};

const MAX_GRID_SIZE: u16 = 32_767;

#[derive(Copy, Clone, Debug)]
struct GridSize2D {
    width: usize,
    height: usize,
}

#[derive(Copy, Clone, Debug)]
struct GridSize3D {
    width: usize,
    height: usize,
    depth: usize,
}

pub trait NoiseField {}

// impl NoiseField for NoiseField1D {}
impl NoiseField for NoiseField2D {}
impl NoiseField for NoiseField3D {}
// impl NoiseField for NoiseField4D {}

// pub struct NoiseField1D {
//     x: Vec<f64>,
//
//     values: Vec<f64>,

#[derive(Clone, Debug)]
pub struct NoiseField2D {
    grid_size: GridSize2D,

    // field_size: (f64, f64),
    // field_origin: (f64, f64),
    coordinates: Vec<Vector2<f64>>,

    pub values: Vec<f64>,
}

impl NoiseField2D {
    // pub fn new(grid_size: (usize, usize), field_size: (f64, f64), field_origin: (f64, f64)) -> Self {
    pub fn new(grid_width: usize, grid_height: usize) -> Self {
        // let (grid_width, grid_height) = grid_size;

        // Check for invalid grid width or height.
        //TODO: Return an error here instead of panicking
        assert!(grid_width > 0);
        assert!(grid_height > 0);
        assert!(grid_width < MAX_GRID_SIZE as usize);
        assert!(grid_height < MAX_GRID_SIZE as usize);

        // let (field_width, field_height) = field_size;
        //
        // // Check for invalid field width or height
        // //TODO: Return an error here instead of panicking
        // assert!((0.0 - field_width).abs() < std::f64::EPSILON);
        // assert!((0.0 - field_height).abs() < std::f64::EPSILON);

        let grid_size = grid_width * grid_height;

        Self {
            grid_size: GridSize2D {
                width: grid_width,
                height: grid_height,
            },

            // field_size,
            // field_origin,
            coordinates: vec![[0.0; 2]; grid_size],

            values: vec![0.0; grid_size],
        }
    }

    pub fn grid_size(&self) -> (usize, usize) {
        (self.grid_size.width, self.grid_size.height)
    }

    pub fn coordinates(&self) -> &Vec<Vector2<f64>> {
        &self.coordinates
    }

    pub fn coord_at_point(&self, grid_point: Vector2<usize>) -> Vector2<f64> {
        let index = self.index(grid_point);

        self.coordinates[index]
    }

    pub fn set_coord_at_point(&mut self, grid_point: Vector2<usize>, coordinate: Vector2<f64>) {
        let index = self.index(grid_point);

        self.coordinates[index] = coordinate;
    }

    pub fn scale_coordinates(&self, scale: f64) -> Self {
        let mut out = self.clone();

        for i in 0..self.coordinates.len() {
            let [x, y] = self.coordinates[i];
            out.coordinates[i] = [x * scale, y * scale];
        }

        out
    }

    pub fn values(&self) -> &Vec<f64> {
        &self.values
    }

    pub fn value_at_point(&self, grid_point: Vector2<usize>) -> f64 {
        let index = self.index(grid_point);

        self.values[index]
    }

    pub fn value_at_index(&self, index: usize) -> f64 {
        self.values[index]
    }

    fn index(&self, grid_point: Vector2<usize>) -> usize {
        // Y
        // |
        // 2 | 6 7 8
        // 1 | 3 4 5
        // 0 | 0 1 2
        // --|------
        //   | 0 1 2 - X

        let [x, y] = grid_point;

        x + (self.grid_size.width * y)
    }

    pub fn initialize() -> Self {
        Self {
            grid_size: GridSize2D {
                width: 1,
                height: 1,
            },

            // field_size: (0.0, 0.0),
            // field_origin: (0.0, 0.0),
            coordinates: Vec::new(),

            values: Vec::new(),
        }
    }
}

// impl Default for NoiseField2D {
//     fn default() -> Self {
//         Self::initialize()
//     }
// }

#[derive(Clone, Debug)]
pub struct NoiseField3D {
    grid_size: GridSize3D,

    // field_size: (f64, f64),
    // field_origin: (f64, f64),
    pub coordinates: Vec<Vector3<f64>>,

    pub values: Vec<f64>,
}

impl NoiseField3D {
    // pub fn new(grid_size: (usize, usize), field_size: (f64, f64), field_origin: (f64, f64)) -> Self {
    pub fn new(grid_width: usize, grid_height: usize, grid_depth: usize) -> Self {
        // let (grid_width, grid_height) = grid_size;

        // Check for invalid grid width or height.
        //TODO: Return an error here instead of panicking
        assert!(grid_width > 0);
        assert!(grid_height > 0);
        assert!(grid_depth > 0);
        assert!(grid_width < MAX_GRID_SIZE as usize);
        assert!(grid_height < MAX_GRID_SIZE as usize);
        assert!(grid_depth < MAX_GRID_SIZE as usize);

        // let (field_width, field_height) = field_size;
        //
        // // Check for invalid field width or height
        // //TODO: Return an error here instead of panicking
        // assert!((0.0 - field_width).abs() < std::f64::EPSILON);
        // assert!((0.0 - field_height).abs() < std::f64::EPSILON);

        let grid_size = grid_width * grid_height * grid_depth;

        Self {
            grid_size: GridSize3D {
                width: grid_width,
                height: grid_height,
                depth: grid_depth,
            },

            // field_size,
            // field_origin,
            coordinates: vec![[0.0; 3]; grid_size],

            values: vec![0.0; grid_size],
        }
    }

    pub fn grid_size(&self) -> (usize, usize, usize) {
        (
            self.grid_size.width,
            self.grid_size.height,
            self.grid_size.depth,
        )
    }

    pub fn coordinates(&self) -> &Vec<Vector3<f64>> {
        &self.coordinates
    }

    pub fn coord_at_point(&self, grid_point: Vector3<usize>) -> Vector3<f64> {
        let index = self.index(grid_point);

        self.coordinates[index]
    }

    pub fn set_coord_at_point(&mut self, grid_point: Vector3<usize>, coordinate: Vector3<f64>) {
        let index = self.index(grid_point);

        self.coordinates[index] = coordinate;
    }

    pub fn scale_coordinates(&self, scale: f64) -> Self {
        let mut out = self.clone();

        for i in 0..self.coordinates.len() {
            let [x, y, z] = self.coordinates[i];
            out.coordinates[i] = [x * scale, y * scale, z * scale];
        }

        out
    }

    pub fn values(&self) -> &Vec<f64> {
        &self.values
    }

    pub fn value_at_point(&self, grid_point: Vector3<usize>) -> f64 {
        let index = self.index(grid_point);

        self.values[index]
    }

    pub fn value_at_index(&self, index: usize) -> f64 {
        self.values[index]
    }

    fn index(&self, grid_point: Vector3<usize>) -> usize {
        // Y
        // |
        // 2 | 6 7 8
        // 1 | 3 4 5
        // 0 | 0 1 2
        // --|------
        //   | 0 1 2 - X

        let [x, y, z] = grid_point;

        x + (y * self.grid_size.width) + (z * self.grid_size.width * self.grid_size.height)
    }

    pub fn initialize() -> Self {
        Self {
            grid_size: GridSize3D {
                width: 1,
                height: 1,
                depth: 1,
            },

            // field_size: (0.0, 0.0),
            // field_origin: (0.0, 0.0),
            coordinates: Vec::new(),

            values: Vec::new(),
        }
    }
}

// pub struct NoiseField4D {
//     x: Vec<f64>,
//     y: Vec<f64>,
//     z: Vec<f64>,
//     w: Vec<f64>,
//
//     values: Vec<f64>,
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_noisefield2d() {
        let noisefield = NoiseField2D::new(1, 1);

        assert_eq!(noisefield.grid_size.width, 1);
        assert_eq!(noisefield.grid_size.height, 1);
    }

    #[test]
    #[should_panic]
    fn create_too_small_noisefield2d_x() {
        let noisefield = NoiseField2D::new(0, 1);
    }

    #[test]
    #[should_panic]
    fn create_too_small_noisefield2d_y() {
        let noisefield = NoiseField2D::new(1, 0);
    }

    #[test]
    fn get_index() {
        let index = NoiseField2D::new(3, 3).index([1, 1]);

        assert_eq!(index, 4);
    }

    #[test]
    fn set_coord() {
        let grid_point = [1, 1];
        let coordinate = [0.1, 1.5];
        let mut noisefield = NoiseField2D::new(3, 3);
        noisefield.set_field_coord(grid_point, coordinate);
        let index = noisefield.index(grid_point);

        assert_eq!(coordinate, noisefield.coordinates[index]);
    }
}
