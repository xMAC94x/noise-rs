use vek::{Vec2, Vec3};

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

pub trait NoiseField {
    fn values(&self) -> &Vec<f64>;

    fn set_values(&mut self, values: &[f64]);
}

// impl NoiseField for NoiseField1D {}
impl NoiseField for NoiseField2D {
    fn values(&self) -> &Vec<f64> {
        &self.values
    }

    fn set_values(&mut self, values: &[f64]) {
        self.values = Vec::from(values);
    }
}
impl NoiseField for NoiseField3D {
    fn values(&self) -> &Vec<f64> {
        &self.values
    }

    fn set_values(&mut self, values: &[f64]) {
        self.values = Vec::from(values);
    }
}
// impl NoiseField for NoiseField4D {}

// pub struct NoiseField1D {
//     x: Vec<f64>,
//
//     values: Vec<f64>,

#[derive(Clone, Debug)]
pub struct NoiseField2D {
    size: GridSize2D,

    // field_size: (f64, f64),
    // field_origin: (f64, f64),
    pub coordinates: Vec<Vec2<f64>>,

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
            size: GridSize2D {
                width: grid_width,
                height: grid_height,
            },

            // field_size,
            // field_origin,
            coordinates: vec![Vec2::zero(); grid_size],

            values: vec![0.0; grid_size],
        }
    }

    pub fn size(&self) -> [usize; 2] {
        [self.size.width, self.size.height]
    }

    pub fn coordinates(&self) -> &Vec<Vec2<f64>> {
        &self.coordinates
    }

    pub fn coord_at_point(&self, point: Vec2<usize>) -> Vec2<f64> {
        let index = self.index(point);

        self.coordinates[index]
    }

    pub fn set_coord_at_point(&mut self, point: Vec2<usize>, coordinate: Vec2<f64>) {
        let index = self.index(point);

        self.coordinates[index] = coordinate;
    }

    pub fn scale_coordinates(&self, scale: f64) -> Self {
        let out = self.clone();

        for &coordinate in &out.coordinates {
            coordinate.map(|x| x * scale);
        }

        out
    }

    pub fn value_at_point(&self, point: Vec2<usize>) -> f64 {
        let index = self.index(point);

        self.values[index]
    }

    pub fn value_at_index(&self, index: usize) -> f64 {
        self.values[index]
    }

    pub fn build_field(&mut self, x_bounds: (f64, f64), y_bounds: (f64, f64)) {
        let x_extent = x_bounds.1 - x_bounds.0;
        let y_extent = y_bounds.1 - y_bounds.0;

        let x_step = x_extent / self.size.width as f64;
        let y_step = y_extent / self.size.height as f64;

        for y in 0..self.size.height {
            let current_y = y_bounds.0 + y_step * y as f64;

            for x in 0..self.size.width {
                let current_x = x_bounds.0 + x_step * x as f64;

                self.set_coord_at_point(Vec2::new(x, y), Vec2::new(current_x, current_y));
            }
        }
    }

    fn index(&self, point: Vec2<usize>) -> usize {
        // Y
        // |
        // 2 | 6 7 8
        // 1 | 3 4 5
        // 0 | 0 1 2
        // --|------
        //   | 0 1 2 - X

        point.x + (self.size.width * point.y)
    }

    pub fn initialize() -> Self {
        Self {
            size: GridSize2D {
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
    size: GridSize3D,

    // field_size: (f64, f64),
    // field_origin: (f64, f64),
    pub coordinates: Vec<Vec3<f64>>,

    pub values: Vec<f64>,
}

impl NoiseField3D {
    // pub fn new(grid_size: (usize, usize), field_size: (f64, f64), field_origin: (f64, f64)) -> Self {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        // Check for invalid grid width or height.
        //TODO: Return an error here instead of panicking
        assert!(width > 0);
        assert!(height > 0);
        assert!(depth > 0);
        assert!(width < MAX_GRID_SIZE as usize);
        assert!(height < MAX_GRID_SIZE as usize);
        assert!(depth < MAX_GRID_SIZE as usize);

        // let (field_width, field_height) = field_size;
        //
        // // Check for invalid field width or height
        // //TODO: Return an error here instead of panicking
        // assert!((0.0 - field_width).abs() < std::f64::EPSILON);
        // assert!((0.0 - field_height).abs() < std::f64::EPSILON);

        let grid_size = width * height * depth;

        Self {
            size: GridSize3D {
                width,
                height,
                depth,
            },

            // field_size,
            // field_origin,
            coordinates: vec![Vec3::zero(); grid_size],

            values: vec![0.0; grid_size],
        }
    }

    pub fn size(&self) -> [usize; 3] {
        [self.size.width, self.size.height, self.size.depth]
    }

    pub fn coordinates(&self) -> &Vec<Vec3<f64>> {
        &self.coordinates
    }

    pub fn coord_at_point(&self, point: Vec3<usize>) -> Vec3<f64> {
        let index = self.index(point);

        self.coordinates[index]
    }

    pub fn set_coord_at_point(&mut self, point: Vec3<usize>, coordinate: Vec3<f64>) {
        let index = self.index(point);

        self.coordinates[index] = coordinate;
    }

    pub fn scale_coordinates(&self, scale: f64) -> Self {
        let mut out = self.clone();

        for &coordinate in &out.coordinates {
            coordinate.map(|x| x * scale);
        }

        out
    }

    pub fn value_at_point(&self, grid_point: Vec3<usize>) -> f64 {
        let index = self.index(grid_point);

        self.values[index]
    }

    pub fn value_at_index(&self, index: usize) -> f64 {
        self.values[index]
    }

    pub fn build_field(
        &mut self,
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        z_bounds: (f64, f64),
    ) {
        let x_extent = x_bounds.1 - x_bounds.0;
        let y_extent = y_bounds.1 - y_bounds.0;
        let z_extent = y_bounds.1 - y_bounds.0;

        let x_step = x_extent / self.size.width as f64;
        let y_step = y_extent / self.size.height as f64;
        let z_step = y_extent / self.size.depth as f64;

        for z in 0..self.size.depth {
            let current_z = z_bounds.0 + z_step * z as f64;

            for y in 0..self.size.height {
                let current_y = y_bounds.0 + y_step * y as f64;

                for x in 0..self.size.width {
                    let current_x = x_bounds.0 + x_step * x as f64;

                    self.set_coord_at_point(
                        Vec3 { x, y, z },
                        Vec3::new(current_x, current_y, current_z),
                    );
                }
            }
        }
    }

    fn index(&self, point: Vec3<usize>) -> usize {
        // Y
        // |
        // 2 | 6 7 8
        // 1 | 3 4 5
        // 0 | 0 1 2
        // --|------
        //   | 0 1 2 - X

        point.x + (point.y * self.size.width) + (point.z * self.size.width * self.size.height)
    }

    pub fn initialize() -> Self {
        Self {
            size: GridSize3D {
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

        assert_eq!(noisefield.size.width, 1);
        assert_eq!(noisefield.size.height, 1);
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
        let index = NoiseField2D::new(3, 3).index(Vec2::new(1, 1));

        assert_eq!(index, 4);
    }

    #[test]
    fn set_coord() {
        let grid_point = Vec2::new(1, 1);
        let coordinate = Vec2::new(0.1, 1.5);
        let mut noisefield = NoiseField2D::new(3, 3);
        noisefield.set_coord_at_point(grid_point, coordinate);
        let index = noisefield.index(grid_point);

        assert_eq!(coordinate, noisefield.coordinates[index]);
    }
}
