use crate::noisefield::{NoiseField, NoiseField2D, NoiseField3D};
use crate::{utils::noise_map::NoiseMap, NoiseFieldFn};
use vek::{Vec2, Vec3};

pub trait NoiseMapBuilder<'a, T>
where
    T: NoiseField,
{
    fn set_size(self, width: usize, height: usize) -> Self;

    fn set_source_module(self, source_module: &'a dyn NoiseFieldFn<T>) -> Self;

    fn size(&self) -> (usize, usize);

    fn build(&self) -> NoiseMap;
}

pub struct CylinderMapBuilder<'a> {
    angle_bounds: (f64, f64),
    height_bounds: (f64, f64),
    size: (usize, usize),
    source_module: &'a dyn NoiseFieldFn<NoiseField3D>,
}

impl<'a> CylinderMapBuilder<'a> {
    pub fn new(source_module: &'a dyn NoiseFieldFn<NoiseField3D>) -> Self {
        CylinderMapBuilder {
            angle_bounds: (-180.0, 180.0),
            height_bounds: (-10.0, 10.0),
            size: (1000, 1000),
            source_module,
        }
    }

    pub fn set_angle_bounds(self, lower_bound: f64, upper_bound: f64) -> Self {
        let angle_bounds = if lower_bound >= upper_bound {
            eprintln!(
                "lower bound {:?} is larger than upper bound {:?}, switching order",
                lower_bound, upper_bound
            );
            (upper_bound, lower_bound)
        } else {
            (lower_bound, upper_bound)
        };

        CylinderMapBuilder {
            angle_bounds,
            ..self
        }
    }

    pub fn set_height_bounds(self, lower_bound: f64, upper_bound: f64) -> Self {
        let height_bounds = if lower_bound >= upper_bound {
            eprintln!(
                "lower bound {:?} is larger than upper bound {:?}, switching order",
                lower_bound, upper_bound
            );
            (upper_bound, lower_bound)
        } else {
            (lower_bound, upper_bound)
        };

        CylinderMapBuilder {
            height_bounds,
            ..self
        }
    }

    pub fn angle_bounds(&self) -> (f64, f64) {
        self.angle_bounds
    }

    pub fn height_bounds(&self) -> (f64, f64) {
        self.height_bounds
    }
}

impl<'a> NoiseMapBuilder<'a, NoiseField3D> for CylinderMapBuilder<'a> {
    fn set_size(self, width: usize, height: usize) -> Self {
        CylinderMapBuilder {
            size: (width, height),
            ..self
        }
    }

    fn set_source_module(self, source_module: &'a dyn NoiseFieldFn<NoiseField3D>) -> Self {
        CylinderMapBuilder {
            source_module,
            ..self
        }
    }

    fn size(&self) -> (usize, usize) {
        self.size
    }

    fn build(&self) -> NoiseMap {
        let (width, height) = self.size;

        let mut result_map = NoiseMap::new(width, height);
        let mut field = NoiseField3D::new(width, height, 1);

        let (width, height) = self.size;

        let angle_extent = self.angle_bounds.1 - self.angle_bounds.0;
        let height_extent = self.height_bounds.1 - self.height_bounds.0;

        let x_step = angle_extent / width as f64;
        let y_step = height_extent / height as f64;

        for y in 0..height {
            let current_height = self.height_bounds.0 + y_step * y as f64;

            for x in 0..width {
                let current_angle = self.angle_bounds.0 + x_step * x as f64;

                let point_x = current_angle.to_radians().cos();
                let point_z = current_angle.to_radians().sin();

                field.set_coord_at_point(
                    Vec3 { x, y, z: 0 },
                    Vec3::new(point_x, current_height, point_z),
                );
            }
        }

        field = self.source_module.process_field(&field);
        let grid_size = field.size();

        for x in 0..grid_size[0] {
            for y in 0..grid_size[1] {
                result_map.set_value(x, y, field.value_at_point(Vec3 { x, y, z: 0 }));
            }
        }

        result_map
    }
}

pub struct PlaneMapBuilder<'a> {
    is_seamless: bool,
    x_bounds: (f64, f64),
    y_bounds: (f64, f64),
    size: (usize, usize),
    source_module: &'a dyn NoiseFieldFn<NoiseField2D>,
}

impl<'a> PlaneMapBuilder<'a> {
    pub fn new(source_module: &'a dyn NoiseFieldFn<NoiseField2D>) -> Self {
        PlaneMapBuilder {
            is_seamless: false,
            x_bounds: (-4.0, 4.0),
            y_bounds: (-4.0, 4.0),
            size: (1000, 1000),
            source_module,
        }
    }

    pub fn set_is_seamless(self, is_seamless: bool) -> Self {
        PlaneMapBuilder {
            is_seamless,
            ..self
        }
    }

    pub fn set_x_bounds(self, lower_x_bound: f64, upper_x_bound: f64) -> Self {
        PlaneMapBuilder {
            x_bounds: (lower_x_bound, upper_x_bound),
            ..self
        }
    }

    pub fn set_y_bounds(self, lower_y_bound: f64, upper_y_bound: f64) -> Self {
        PlaneMapBuilder {
            y_bounds: (lower_y_bound, upper_y_bound),
            ..self
        }
    }

    pub fn x_bounds(&self) -> (f64, f64) {
        self.x_bounds
    }

    pub fn y_bounds(&self) -> (f64, f64) {
        self.y_bounds
    }
}

impl<'a> NoiseMapBuilder<'a, NoiseField2D> for PlaneMapBuilder<'a> {
    fn set_size(self, width: usize, height: usize) -> Self {
        PlaneMapBuilder {
            size: (width, height),
            ..self
        }
    }

    fn set_source_module(self, source_module: &'a dyn NoiseFieldFn<NoiseField2D>) -> Self {
        PlaneMapBuilder {
            source_module,
            ..self
        }
    }

    fn size(&self) -> (usize, usize) {
        self.size
    }

    fn build(&self) -> NoiseMap {
        let (width, height) = self.size;

        let mut result_map = NoiseMap::new(width, height);
        let mut field = NoiseField2D::new(width, height);

        let x_extent = self.x_bounds.1 - self.x_bounds.0;
        let y_extent = self.y_bounds.1 - self.y_bounds.0;

        let x_step = x_extent / width as f64;
        let y_step = y_extent / height as f64;

        for y in 0..height {
            let current_y = self.y_bounds.0 + y_step * y as f64;

            for x in 0..width {
                let current_x = self.x_bounds.0 + x_step * x as f64;

                field.set_coord_at_point(Vec2 { x, y }, Vec2::new(current_x, current_y));

                // let final_value = if self.is_seamless {
                //     let sw_value = self.source_module.get([current_x, current_y, 0.0]);
                //     let se_value = self
                //         .source_module
                //         .get([current_x + x_extent, current_y, 0.0]);
                //     let nw_value = self
                //         .source_module
                //         .get([current_x, current_y + y_extent, 0.0]);
                //     let ne_value =
                //         self.source_module
                //             .get([current_x + x_extent, current_y + y_extent, 0.0]);
                //
                //     let x_blend = 1.0 - ((current_x - self.x_bounds.0) / x_extent);
                //     let y_blend = 1.0 - ((current_y - self.y_bounds.0) / y_extent);
                //
                //     let y0 = interpolate::linear(sw_value, se_value, x_blend);
                //     let y1 = interpolate::linear(nw_value, ne_value, x_blend);
                //
                //     interpolate::linear(y0, y1, y_blend)
                // } else {
                //     self.source_module.get([current_x, current_y, 0.0]);
                // };

                // result_map.set_value(x, y, final_value);
            }
        }

        field = self.source_module.process_field(&field);
        let grid_size = field.size();

        for x in 0..grid_size[0] {
            for y in 0..grid_size[1] {
                result_map.set_value(x, y, field.value_at_point(Vec2::new(x, y)));
            }
        }

        result_map
    }
}

pub struct SphereMapBuilder<'a> {
    latitude_bounds: (f64, f64),
    longitude_bounds: (f64, f64),
    radius: f64,
    size: (usize, usize),
    source_module: &'a dyn NoiseFieldFn<NoiseField3D>,
}

impl<'a> SphereMapBuilder<'a> {
    pub fn new(source_module: &'a dyn NoiseFieldFn<NoiseField3D>) -> Self {
        SphereMapBuilder {
            latitude_bounds: (-90.0, 90.0),
            longitude_bounds: (-180.0, 180.0),
            radius: 10.0,
            size: (1000, 1000),
            source_module,
        }
    }

    pub fn set_latitude_bounds(self, min_lat_bound: f64, max_lat_bound: f64) -> Self {
        SphereMapBuilder {
            latitude_bounds: (min_lat_bound, max_lat_bound),
            ..self
        }
    }

    pub fn set_longitude_bounds(self, min_lon_bound: f64, max_lon_bound: f64) -> Self {
        SphereMapBuilder {
            longitude_bounds: (min_lon_bound, max_lon_bound),
            ..self
        }
    }

    pub fn set_bounds(
        self,
        min_lat_bound: f64,
        max_lat_bound: f64,
        min_lon_bound: f64,
        max_lon_bound: f64,
    ) -> Self {
        SphereMapBuilder {
            latitude_bounds: (min_lat_bound, max_lat_bound),
            longitude_bounds: (min_lon_bound, max_lon_bound),
            ..self
        }
    }

    pub fn set_radius(self, radius: f64) -> Self {
        SphereMapBuilder { radius, ..self }
    }

    pub fn latitude_bounds(&self) -> (f64, f64) {
        self.latitude_bounds
    }

    pub fn longitude_bounds(&self) -> (f64, f64) {
        self.longitude_bounds
    }
}

impl<'a> NoiseMapBuilder<'a, NoiseField3D> for SphereMapBuilder<'a> {
    fn set_size(self, width: usize, height: usize) -> Self {
        SphereMapBuilder {
            size: (width, height),
            ..self
        }
    }

    fn set_source_module(self, source_module: &'a dyn NoiseFieldFn<NoiseField3D>) -> Self {
        SphereMapBuilder {
            source_module,
            ..self
        }
    }

    fn size(&self) -> (usize, usize) {
        self.size
    }

    fn build(&self) -> NoiseMap {
        let (width, height) = self.size;

        let mut result_map = NoiseMap::new(width, height);
        let mut field = NoiseField3D::new(width, height, 1);

        let lon_extent = self.longitude_bounds.1 - self.longitude_bounds.0;
        let lat_extent = self.latitude_bounds.1 - self.latitude_bounds.0;

        let x_step = lon_extent / width as f64;
        let y_step = lat_extent / height as f64;

        for y in 0..height {
            let current_lat = self.latitude_bounds.0 + y_step * y as f64;

            for x in 0..width {
                let current_lon = self.longitude_bounds.0 + x_step * x as f64;

                let point = lat_lon_to_xyz(current_lat, current_lon, self.radius);

                field.set_coord_at_point(Vec3 { x, y, z: 0 }, Vec3::from(point));
            }
        }

        field = self.source_module.process_field(&field);
        let grid_size = field.size();

        for x in 0..grid_size[0] {
            for y in 0..grid_size[1] {
                result_map.set_value(x, y, field.value_at_point(Vec3 { x, y, z: 0 }));
            }
        }

        result_map
    }
}

fn lat_lon_to_xyz(lat: f64, lon: f64, radius: f64) -> [f64; 3] {
    let r = lat.to_radians().cos() * radius;
    let x = r * lon.to_radians().cos();
    let y = lat.to_radians().sin() * radius;
    let z = r * lon.to_radians().sin();

    [x, y, z]
}
