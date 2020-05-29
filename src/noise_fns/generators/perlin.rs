use crate::math::interpolate::s_curve5;
use crate::{
    gradient,
    math::{self, Vector2, Vector4},
    noisefield::{NoiseField, NoiseField2D, NoiseField3D},
    permutationtable::PermutationTable,
    NoiseFieldFn, NoiseFn, Seedable,
};
use rayon::prelude::*;
use vek::Vec3;

/// Noise function that outputs 2/3/4-dimensional Perlin noise.
#[derive(Clone, Copy, Debug)]
pub struct Perlin {
    seed: u32,
    perm_table: PermutationTable,
}

impl Perlin {
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new() -> Self {
        Self {
            seed: Self::DEFAULT_SEED,
            perm_table: PermutationTable::new(Self::DEFAULT_SEED),
        }
    }

    pub fn perlin_2d_surflet(&self, point: Vector2<f64>) -> f64 {
        const SCALE_FACTOR: f64 = 3.160_493_827_160_493_7;

        #[inline(always)]
        fn surflet(perm_table: &PermutationTable, corner: [isize; 2], distance: [f64; 2]) -> f64 {
            let attn = 1.0 - math::dot2(distance, distance);
            if attn > 0.0 {
                attn.powi(4) * math::dot2(distance, gradient::get2(perm_table.get2(corner)))
            } else {
                0.0
            }
        }

        let floored = math::map2(point, f64::floor);
        let near_corner = math::to_isize2(floored);
        let far_corner = math::add2(near_corner, math::one2());
        let near_distance = math::sub2(point, floored);
        let far_distance = math::sub2(near_distance, math::one2());

        let f00 = surflet(
            &self.perm_table,
            [near_corner[0], near_corner[1]],
            [near_distance[0], near_distance[1]],
        );
        let f10 = surflet(
            &self.perm_table,
            [far_corner[0], near_corner[1]],
            [far_distance[0], near_distance[1]],
        );
        let f01 = surflet(
            &self.perm_table,
            [near_corner[0], far_corner[1]],
            [near_distance[0], far_distance[1]],
        );
        let f11 = surflet(
            &self.perm_table,
            [far_corner[0], far_corner[1]],
            [far_distance[0], far_distance[1]],
        );

        // Multiply by arbitrary value to scale to -1..1
        math::clamp((f00 + f10 + f01 + f11) * SCALE_FACTOR, -1.0, 1.0)
    }

    #[inline]
    pub fn perlin_2d(&self, x: f64, y: f64) -> f64 {
        // Unscaled range of linearly interpolated perlin noise should be (-sqrt(N/4), sqrt(N/4)),
        // where N is the dimension of the noise. Need to invert this value and multiply the
        // unscaled result by the value to get a scaled range of (-1, 1).
        // 1/sqrt(N/4), N=2 -> 1/sqrt(1/2) -> sqrt(2)
        let scale_factor: f64 = (2.0_f64).sqrt();

        #[inline(always)]
        fn gradient_dot_v(perm: usize, point: [f64; 2]) -> f64 {
            let x = point[0];
            let y = point[1];

            match perm & 0b11 {
                0 => x + y,  // ( 1,  1)
                1 => -x + y, // (-1,  1)
                2 => x - y,  // ( 1, -1)
                3 => -x - y, // (-1, -1)
                _ => unreachable!(),
            }
        }

        let floored = [x.floor(), y.floor()];
        let near_corner = [floored[0] as isize, floored[1] as isize];
        let far_corner = [near_corner[0] + 1, near_corner[1] + 1];
        let near_distance = [x - floored[0], y - floored[1]];
        let far_distance = [near_distance[0] - 1., near_distance[1] - 1.];

        let u = s_curve5(near_distance[0]);
        let v = s_curve5(near_distance[1]);

        let g00 = gradient_dot_v(self.perm_table.get2(near_corner), near_distance);
        let g10 = gradient_dot_v(
            self.perm_table.get2([far_corner[0], near_corner[1]]),
            [far_distance[0], near_distance[1]],
        );
        let g01 = gradient_dot_v(
            self.perm_table.get2([near_corner[0], far_corner[1]]),
            [near_distance[0], far_distance[1]],
        );
        let g11 = gradient_dot_v(self.perm_table.get2(far_corner), far_distance);

        let k0 = g00;
        let k1 = g10 - g00;
        let k2 = g01 - g00;
        let k3 = g00 + g11 - g10 - g01;

        let unscaled_result = k0 + k1 * u + k2 * v + k3 * u * v;

        let scaled_result = unscaled_result * scale_factor;

        // At this point, we should be really close to the (-1, 1) range, but some float errors
        // could have accumulated, so let's just clamp the results to (-1, 1) to cut off any
        // outliers and return it.
        math::clamp(scaled_result, -1.0, 1.0)
    }

    pub fn perlin_2d_autovec(&self, x: &[f64], y: &[f64]) -> Vec<f64> {
        x.iter()
            .zip(y.iter())
            .map(|(&x, &y)| self.perlin_2d(x, y))
            .collect()
    }

    pub fn perlin_2d_autovec_pariter(&self, x: &[f64], y: &[f64]) -> Vec<f64> {
        x.par_iter()
            .zip(y.par_iter())
            .map(|(&x, &y)| self.perlin_2d(x, y))
            .collect()
    }

    pub fn perlin_3d_surflet(&self, point: Vec3<f64>) -> f64 {
        const SCALE_FACTOR: f64 = 3.889_855_325_553_107_4;

        #[inline(always)]
        fn surflet(perm_table: &PermutationTable, corner: [isize; 3], distance: [f64; 3]) -> f64 {
            let attn = 1.0 - math::dot3(distance, distance);
            if attn > 0.0 {
                attn.powi(4) * math::dot3(distance, gradient::get3(perm_table.get3(corner)))
            } else {
                0.0
            }
        }

        let floored = point.map(|a| a.floor());
        let near_corner = floored.map(|a| a as isize);
        let far_corner = near_corner + Vec3::one();
        let near_distance = point - floored;
        let far_distance = near_distance - Vec3::one();

        let f000 = surflet(
            &self.perm_table,
            [near_corner.x, near_corner.y, near_corner.z],
            [near_distance.x, near_distance.y, near_distance.x],
        );
        let f100 = surflet(
            &self.perm_table,
            [far_corner.x, near_corner.y, near_corner.z],
            [far_distance.x, near_distance.y, near_distance.z],
        );
        let f010 = surflet(
            &self.perm_table,
            [near_corner.x, far_corner.y, near_corner.z],
            [near_distance.x, far_distance.y, near_distance.z],
        );
        let f110 = surflet(
            &self.perm_table,
            [far_corner.x, far_corner.y, near_corner.z],
            [far_distance.x, far_distance.y, near_distance.z],
        );
        let f001 = surflet(
            &self.perm_table,
            [near_corner.x, near_corner.y, far_corner.z],
            [near_distance.x, near_distance.y, far_distance.z],
        );
        let f101 = surflet(
            &self.perm_table,
            [far_corner.x, near_corner.y, far_corner.z],
            [far_distance.x, near_distance.y, far_distance.z],
        );
        let f011 = surflet(
            &self.perm_table,
            [near_corner.x, far_corner.y, far_corner.z],
            [near_distance.x, far_distance.y, far_distance.z],
        );
        let f111 = surflet(
            &self.perm_table,
            [far_corner.x, far_corner.y, far_corner.z],
            [far_distance.x, far_distance.y, far_distance.z],
        );

        // Multiply by arbitrary value to scale to -1..1
        math::clamp(
            (f000 + f100 + f010 + f110 + f001 + f101 + f011 + f111) * SCALE_FACTOR,
            -1.0,
            1.0,
        )
    }

    #[inline]
    pub fn perlin_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        // Unscaled range of linearly interpolated perlin noise should be (-sqrt(N/4), sqrt(N/4)).
        // Need to invert this value and multiply the unscaled result by the value to get a scaled
        // range of (-1, 1).
        // 1/sqrt(N/4), N=3 -> 1/sqrt(3/4) -> 2/sqrt(3)
        let scale_factor: f64 = 2.0_f64 / ((3.0_f64).sqrt());

        #[inline(always)]
        #[cfg_attr(rustfmt, rustfmt_skip)]
        fn gradient_dot_v(perm: usize, x: f64, y: f64, z: f64) -> f64 {
            match perm & 0b1111 {
                0 =>  x + y, // ( 1,  1,  0)
                1 => -x + y, // (-1,  1,  0)
                2 =>  x - y, // ( 1, -1,  0)
                3 => -x - y, // (-1, -1,  0)
                4 =>  x + z, // ( 1,  0,  1)
                5 => -x + z, // (-1,  0,  1)
                6 =>  x - z, // ( 1,  0, -1)
                7 => -x - z, // (-1,  0, -1)
                8 =>  y + z, // ( 0,  1,  1)
                9 => -y + z, // ( 0, -1,  1)
                10 =>  y - z, // ( 0,  1, -1)
                11 => -y - z, // ( 0, -1, -1)
                12 =>  x + y, // ( 1,  1,  0) - Repeated
                13 => -x + y, // (-1,  1,  0) - Repeated
                14 => -y + z, // ( 0, -1,  1) - Repeated
                15 => -y - z, // ( 0, -1, -1) - Repeated
                _ => unreachable!(),
            }
        }

        let floored = [x.floor(), y.floor(), z.floor()];
        let near_corner = [
            floored[0] as isize,
            floored[1] as isize,
            floored[2] as isize,
        ];
        let far_corner = [near_corner[0] + 1, near_corner[1] + 1, near_corner[2] + 1];
        let near_distance = [x - floored[0], y - floored[1], z - floored[2]];
        let far_distance = [
            near_distance[0] - 1.,
            near_distance[1] - 1.,
            near_distance[2] - 1.,
        ];

        let u = s_curve5(near_distance[0]);
        let v = s_curve5(near_distance[1]);
        let w = s_curve5(near_distance[2]);

        let g000 = gradient_dot_v(
            self.perm_table.get3(near_corner),
            near_distance[0],
            near_distance[1],
            near_distance[2],
        );
        let g100 = gradient_dot_v(
            self.perm_table
                .get3([far_corner[0], near_corner[1], near_corner[2]]),
            far_distance[0],
            near_distance[1],
            near_distance[2],
        );
        let g010 = gradient_dot_v(
            self.perm_table
                .get3([near_corner[0], far_corner[1], near_corner[2]]),
            near_distance[0],
            far_distance[1],
            near_distance[2],
        );
        let g110 = gradient_dot_v(
            self.perm_table
                .get3([far_corner[0], far_corner[1], near_corner[2]]),
            far_distance[0],
            far_distance[1],
            near_distance[2],
        );
        let g001 = gradient_dot_v(
            self.perm_table
                .get3([near_corner[0], near_corner[1], far_corner[2]]),
            near_distance[0],
            near_distance[1],
            far_distance[2],
        );
        let g101 = gradient_dot_v(
            self.perm_table
                .get3([far_corner[0], near_corner[1], far_corner[2]]),
            far_distance[0],
            near_distance[1],
            far_distance[2],
        );
        let g011 = gradient_dot_v(
            self.perm_table
                .get3([near_corner[0], far_corner[1], far_corner[2]]),
            near_distance[0],
            far_distance[1],
            far_distance[2],
        );
        let g111 = gradient_dot_v(
            self.perm_table.get3(far_corner),
            far_distance[0],
            far_distance[1],
            far_distance[2],
        );

        let k0 = g000;
        let k1 = g100 - g000;
        let k2 = g010 - g000;
        let k3 = g001 - g000;
        let k4 = g000 + g110 - g100 - g010;
        let k5 = g000 + g101 - g100 - g001;
        let k6 = g000 + g011 - g010 - g001;
        let k7 = g100 + g010 + g001 + g111 - g000 - g110 - g101 - g011;

        let unscaled_result =
            k0 + k1 * u + k2 * v + k3 * w + k4 * u * v + k5 * u * w + k6 * v * w + k7 * u * v * w;

        let scaled_result = unscaled_result * scale_factor;

        // At this point, we should be really damn close to the (-1, 1) range, but some float errors
        // could have accumulated, so let's just clamp the results to (-1, 1) to cut off any
        // outliers and return it.
        math::clamp(scaled_result, -1.0, 1.0)
    }

    pub fn perlin_3d_autovec(&self, x: &[f64], y: &[f64], z: &[f64]) -> Vec<f64> {
        x.iter()
            .zip(y.iter())
            .zip(z.iter())
            .map(|((&x, &y), &z)| self.perlin_3d(x, y, z))
            .collect()
    }

    pub fn perlin_3d_autovec_pariter(&self, x: &[f64], y: &[f64], z: &[f64]) -> Vec<f64> {
        x.par_iter()
            .zip(y.par_iter())
            .zip(z.par_iter())
            .map(|((&x, &y), &z)| self.perlin_3d(x, y, z))
            .collect()
    }

    pub fn perlin_4d(&self, point: Vector4<f64>) -> f64 {
        const SCALE_FACTOR: f64 = 4.424_369_240_215_691;

        #[inline(always)]
        fn surflet(perm_table: &PermutationTable, corner: [isize; 4], distance: [f64; 4]) -> f64 {
            let attn = 1.0 - math::dot4(distance, distance);
            if attn > 0.0 {
                attn.powi(4) * math::dot4(distance, gradient::get4(perm_table.get4(corner)))
            } else {
                0.0
            }
        }

        let floored = math::map4(point, f64::floor);
        let near_corner = math::to_isize4(floored);
        let far_corner = math::add4(near_corner, math::one4());
        let near_distance = math::sub4(point, floored);
        let far_distance = math::sub4(near_distance, math::one4());

        let f0000 = surflet(
            &self.perm_table,
            [
                near_corner[0],
                near_corner[1],
                near_corner[2],
                near_corner[3],
            ],
            [
                near_distance[0],
                near_distance[1],
                near_distance[2],
                near_distance[3],
            ],
        );
        let f1000 = surflet(
            &self.perm_table,
            [
                far_corner[0],
                near_corner[1],
                near_corner[2],
                near_corner[3],
            ],
            [
                far_distance[0],
                near_distance[1],
                near_distance[2],
                near_distance[3],
            ],
        );
        let f0100 = surflet(
            &self.perm_table,
            [
                near_corner[0],
                far_corner[1],
                near_corner[2],
                near_corner[3],
            ],
            [
                near_distance[0],
                far_distance[1],
                near_distance[2],
                near_distance[3],
            ],
        );
        let f1100 = surflet(
            &self.perm_table,
            [far_corner[0], far_corner[1], near_corner[2], near_corner[3]],
            [
                far_distance[0],
                far_distance[1],
                near_distance[2],
                near_distance[3],
            ],
        );
        let f0010 = surflet(
            &self.perm_table,
            [
                near_corner[0],
                near_corner[1],
                far_corner[2],
                near_corner[3],
            ],
            [
                near_distance[0],
                near_distance[1],
                far_distance[2],
                near_distance[3],
            ],
        );
        let f1010 = surflet(
            &self.perm_table,
            [far_corner[0], near_corner[1], far_corner[2], near_corner[3]],
            [
                far_distance[0],
                near_distance[1],
                far_distance[2],
                near_distance[3],
            ],
        );
        let f0110 = surflet(
            &self.perm_table,
            [near_corner[0], far_corner[1], far_corner[2], near_corner[3]],
            [
                near_distance[0],
                far_distance[1],
                far_distance[2],
                near_distance[3],
            ],
        );
        let f1110 = surflet(
            &self.perm_table,
            [far_corner[0], far_corner[1], far_corner[2], near_corner[3]],
            [
                far_distance[0],
                far_distance[1],
                far_distance[2],
                near_distance[3],
            ],
        );
        let f0001 = surflet(
            &self.perm_table,
            [
                near_corner[0],
                near_corner[1],
                near_corner[2],
                far_corner[3],
            ],
            [
                near_distance[0],
                near_distance[1],
                near_distance[2],
                far_distance[3],
            ],
        );
        let f1001 = surflet(
            &self.perm_table,
            [far_corner[0], near_corner[1], near_corner[2], far_corner[3]],
            [
                far_distance[0],
                near_distance[1],
                near_distance[2],
                far_distance[3],
            ],
        );
        let f0101 = surflet(
            &self.perm_table,
            [near_corner[0], far_corner[1], near_corner[2], far_corner[3]],
            [
                near_distance[0],
                far_distance[1],
                near_distance[2],
                far_distance[3],
            ],
        );
        let f1101 = surflet(
            &self.perm_table,
            [far_corner[0], far_corner[1], near_corner[2], far_corner[3]],
            [
                far_distance[0],
                far_distance[1],
                near_distance[2],
                far_distance[3],
            ],
        );
        let f0011 = surflet(
            &self.perm_table,
            [near_corner[0], near_corner[1], far_corner[2], far_corner[3]],
            [
                near_distance[0],
                near_distance[1],
                far_distance[2],
                far_distance[3],
            ],
        );
        let f1011 = surflet(
            &self.perm_table,
            [far_corner[0], near_corner[1], far_corner[2], far_corner[3]],
            [
                far_distance[0],
                near_distance[1],
                far_distance[2],
                far_distance[3],
            ],
        );
        let f0111 = surflet(
            &self.perm_table,
            [near_corner[0], far_corner[1], far_corner[2], far_corner[3]],
            [
                near_distance[0],
                far_distance[1],
                far_distance[2],
                far_distance[3],
            ],
        );
        let f1111 = surflet(
            &self.perm_table,
            [far_corner[0], far_corner[1], far_corner[2], far_corner[3]],
            [
                far_distance[0],
                far_distance[1],
                far_distance[2],
                far_distance[3],
            ],
        );

        // Multiply by arbitrary value to scale to -1..1
        math::clamp(
            (f0000
                + f1000
                + f0100
                + f1100
                + f0010
                + f1010
                + f0110
                + f1110
                + f0001
                + f1001
                + f0101
                + f1101
                + f0011
                + f1011
                + f0111
                + f1111)
                * SCALE_FACTOR,
            -1.0,
            1.0,
        )
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

impl Seedable for Perlin {
    /// Sets the seed value for Perlin noise
    fn set_seed(self, seed: u32) -> Self {
        // If the new seed is the same as the current seed, just return self.
        if self.seed == seed {
            return self;
        }

        // Otherwise, regenerate the permutation table based on the new seed.
        Self {
            seed,
            perm_table: PermutationTable::new(seed),
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

/// 2-dimensional perlin noise
impl NoiseFn<[f64; 2]> for Perlin {
    fn get(&self, point: [f64; 2]) -> f64 {
        self.perlin_2d_surflet(point)
    }
}

/// 3-dimensional perlin noise
impl NoiseFn<[f64; 3]> for Perlin {
    fn get(&self, point: [f64; 3]) -> f64 {
        self.perlin_3d(point[0], point[1], point[2])
    }
}

/// 4-dimensional perlin noise
impl NoiseFn<[f64; 4]> for Perlin {
    fn get(&self, point: [f64; 4]) -> f64 {
        self.perlin_4d(point)
    }
}

impl NoiseFieldFn<NoiseField2D> for Perlin {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = field.clone();

        out.set_values(&self.perlin_2d_autovec(&field.x(), &field.y()));

        out
    }
}

impl NoiseFieldFn<NoiseField3D> for Perlin {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = field.clone();

        out.set_values(&self.perlin_3d_autovec(&field.x(), &field.y(), &field.z()));

        out
    }
}
