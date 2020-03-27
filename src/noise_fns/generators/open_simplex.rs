//! Note that this is NOT Ken Perlin's simplex noise, as that is patent encumbered.
//! Instead, these functions use the `OpenSimplex` algorithm, as detailed here:
//! <http://uniblock.tumblr.com/post/97868843242/noise>

use crate::{
    gradient, math,
    noise_fns::{NoiseFn, Seedable},
    permutationtable::PermutationTable,
};
use std::ops::Add;

/// Noise function that outputs 2/3/4-dimensional Open Simplex noise.
#[derive(Clone, Copy, Debug)]
pub struct OpenSimplex {
    seed: u32,
    perm_table: PermutationTable,
}

impl OpenSimplex {
    const DEFAULT_SEED: u32 = 0;

    const STRETCH_CONSTANT_2D: f64 = -0.211_324_865_405_187; //(1/sqrt(2+1)-1)/2;
    const SQUISH_CONSTANT_2D: f64 = 0.366_025_403_784_439; //(sqrt(2+1)-1)/2;
    const STRETCH_CONSTANT_3D: f64 = -1.0 / 6.0; //(1/Math.sqrt(3+1)-1)/3;
    const SQUISH_CONSTANT_3D: f64 = 1.0 / 3.0; //(Math.sqrt(3+1)-1)/3;
    const STRETCH_CONSTANT_4D: f64 = -0.138_196_601_125_011; //(Math.sqrt(4+1)-1)/4;
    const SQUISH_CONSTANT_4D: f64 = 0.309_016_994_374_947; //(Math.sqrt(4+1)-1)/4;

    const NORM_CONSTANT_2D: f64 = 1.0 / 14.0;
    const NORM_CONSTANT_3D: f64 = 1.0 / 14.0;
    const NORM_CONSTANT_4D: f64 = 1.0 / 6.869_909_007_095_662_5;

    pub fn new() -> Self {
        Self {
            seed: Self::DEFAULT_SEED,
            perm_table: PermutationTable::new(Self::DEFAULT_SEED),
        }
    }
}

impl Default for OpenSimplex {
    fn default() -> Self {
        Self::new()
    }
}

impl Seedable for OpenSimplex {
    /// Sets the seed value for Open Simplex noise
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

/// 2-dimensional [`OpenSimplex` Noise](http://uniblock.tumblr.com/post/97868843242/noise)
///
/// This is a slower but higher quality form of gradient noise than `Perlin` 2D.
impl NoiseFn<[f64; 2]> for OpenSimplex {
    fn get(&self, point: [f64; 2]) -> f64 {
        fn gradient(perm_table: &PermutationTable, vertex: [f64; 2], position: [f64; 2]) -> f64 {
            let attn = 2.0 - math::dot2(position, position);
            if attn > 0.0 {
                let index = perm_table.get2(math::to_isize2(vertex));
                let vec = gradient::get2(index);
                attn.powi(4) * math::dot2(position, vec)
            } else {
                0.0
            }
        }

        let zero = 0.0;
        let one = 1.0;
        let two = 2.0;
        let stretch_constant = Self::STRETCH_CONSTANT_2D;
        let squish_constant = Self::SQUISH_CONSTANT_2D;

        // Place input coordinates onto grid.
        let stretch_offset = math::fold2(point, Add::add) * stretch_constant;
        let stretched = math::map2(point, |v| v + stretch_offset);

        // Floor to get grid coordinates of rhombus (stretched square) cell origin.
        let stretched_floor = math::map2(stretched, f64::floor);

        // Skew out to get actual coordinates of rhombus origin. We'll need these later.
        let squish_offset = math::fold2(stretched_floor, Add::add) * squish_constant;
        let skewed_floor = math::map2(stretched_floor, |v| v + squish_offset);

        // Compute grid coordinates relative to rhombus origin.
        let rel_coords = math::sub2(stretched, stretched_floor);

        // Sum those together to get a value that determines which region we're in.
        let region_sum = math::fold2(rel_coords, Add::add);

        // Positions relative to origin point (0, 0).
        let pos0 = math::sub2(point, skewed_floor);

        let mut value = zero;

        let mut vertex;
        let mut dpos;

        let t0 = squish_constant;
        let t1 = squish_constant + one;
        let t2 = squish_constant + t1;
        let t3 = squish_constant + squish_constant;
        let t4 = one + t2;

        // Contribution (1, 0)
        vertex = math::add2(stretched_floor, [one, zero]);
        dpos = math::sub2(pos0, [t1, t0]);
        value = value + gradient(&self.perm_table, vertex, dpos);

        // Contribution (0, 1)
        vertex = math::add2(stretched_floor, [zero, one]);
        dpos = math::sub2(pos0, [t0, t1]);
        value = value + gradient(&self.perm_table, vertex, dpos);

        //              ( 0,  2)
        //                  |   \
        //                  |  F  \
        //                  |       \
        // (-1,  1) --- ( 0,  1) --- ( 1,  1)
        //         \        |   \        |   \
        //           \  E   |  A  \  B   |  C  \
        //             \    |       \    |       \
        //              ( 0,  0) --- ( 1,  0) --- ( 2,  0)
        //                      \        |
        //                        \  D   |
        //                          \    |
        //                           ( 1, -1)

        let ext_vertex;
        let ext_dpos;

        // See the graph for an intuitive explanation; the sum of `x` and `y` is
        // only greater than `1` if we're in Region B.
        if region_sum < one {
            // In region A
            // Contribution (0, 0)
            vertex = math::add2(stretched_floor, [zero, zero]);
            dpos = math::sub2(pos0, [zero, zero]);

            // Surflet radius is larger than one simplex, add contribution from extra vertex
            let center_dist = one - region_sum;
            // If closer to either edge that doesn't border region B
            if center_dist > rel_coords[0] || center_dist > rel_coords[1] {
                if rel_coords[0] > rel_coords[1] {
                    // Nearest contributing surflets are from region D
                    // Contribution (1, -1)
                    ext_vertex = math::add2(stretched_floor, [one, -one]);
                    ext_dpos = math::sub2(pos0, [one, -one]);
                } else {
                    // Nearest contributing surflets are from region E
                    // Contribution (-1, 1)
                    ext_vertex = math::add2(stretched_floor, [-one, one]);
                    ext_dpos = math::sub2(pos0, [-one, one]);
                }
            } else {
                // Nearest contributing surflets are from region B
                // Contribution (1, 1)
                ext_vertex = math::add2(stretched_floor, [one, one]);
                ext_dpos = math::sub2(pos0, [t2, t2]);
            }
        } else {
            // In region B
            // Contribution (1, 1)
            vertex = math::add2(stretched_floor, [one, one]);
            // We are moving across the diagonal `/`, so we'll need to add by the
            // squish constant
            dpos = math::sub2(pos0, [t2, t2]);

            // Surflet radius is larger than one simplex, add contribution from extra vertex
            let center_dist = two - region_sum;
            // If closer to either edge that doesn't border region A
            if center_dist < rel_coords[0] || center_dist < rel_coords[1] {
                if rel_coords[0] > rel_coords[1] {
                    // Nearest contributing surflets are from region C
                    // Contribution (2, 0)
                    ext_vertex = math::add2(stretched_floor, [two, zero]);
                    ext_dpos = math::sub2(pos0, [t4, t3]);
                } else {
                    // Nearest contributing surflets are from region F
                    // Contribution (0, 2)
                    ext_vertex = math::add2(stretched_floor, [zero, two]);
                    ext_dpos = math::sub2(pos0, [t3, t4]);
                }
            } else {
                // Nearest contributing surflets are from region A
                // Contribution (0, 0)
                ext_vertex = math::add2(stretched_floor, [zero, zero]);
                ext_dpos = math::sub2(pos0, [zero, zero]);
            }
        }

        // Point (0, 0) or (1, 1)
        value = value + gradient(&self.perm_table, vertex, dpos);

        // Neighboring simplex point
        value = value + gradient(&self.perm_table, ext_vertex, ext_dpos);

        value * Self::NORM_CONSTANT_2D
    }
}

/// 3-dimensional [`OpenSimplex` Noise](http://uniblock.tumblr.com/post/97868843242/noise)
///
/// This is a slower but higher quality form of gradient noise than `Perlin` 3D.
impl NoiseFn<[f64; 3]> for OpenSimplex {
    fn get(&self, point: [f64; 3]) -> f64 {
        fn gradient(perm_table: &PermutationTable, vertex: [f64; 3], position: [f64; 3]) -> f64 {
            let attn = 2.0 - math::dot3(position, position);
            if attn > 0.0 {
                let index = perm_table.get3(math::to_isize3(vertex));
                let vec = gradient::get3(index);
                attn.powi(4) * math::dot3(position, vec)
            } else {
                0.0
            }
        }

        let zero = 0.0;
        let one = 1.0;
        let two = 2.0;
        let three = 3.0;
        let stretch_constant = Self::STRETCH_CONSTANT_3D;
        let squish_constant = Self::SQUISH_CONSTANT_3D;

        // Place input coordinates on simplectic honeycomb.
        let stretch_offset = math::fold3(point, Add::add) * stretch_constant;
        let stretched = math::map3(point, |v| v + stretch_offset);

        // Floor to get simplectic honeycomb coordinates of rhombohedron
        // (stretched cube) super-cell origin.
        let stretched_floor = math::map3(stretched, f64::floor);

        // Skew out to get actual coordinates of rhombohedron origin. We'll need
        // these later.
        let squish_offset = math::fold3(stretched_floor, Add::add) * squish_constant;
        let skewed_floor = math::map3(stretched_floor, |v| v + squish_offset);

        // Compute simplectic honeycomb coordinates relative to rhombohedral origin.
        let rel_coords = math::sub3(stretched, stretched_floor);

        // Sum those together to get a value that determines which region we're in.
        let region_sum = math::fold3(rel_coords, Add::add);

        // Positions relative to origin point.
        let pos0 = math::sub3(point, skewed_floor);

        let mut value = zero;

        let mut vertex;
        let mut dpos;

        let mut ext0_vertex = stretched_floor;
        let mut ext0_dpos = pos0;
        let mut ext1_vertex = stretched_floor;
        let mut ext1_dpos = pos0;

        if region_sum <= one {
            // We're inside the tetrahedron (3-Simplex) at (0, 0, 0)
            let t0 = squish_constant;
            let t1 = squish_constant + one;

            // Contribution at (0, 0, 0)
            vertex = math::add3(stretched_floor, [zero, zero, zero]);
            dpos = math::sub3(pos0, [zero, zero, zero]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 0, 0)
            vertex = math::add3(stretched_floor, [one, zero, zero]);
            dpos = math::sub3(pos0, [t1, t0, t0]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 0)
            vertex = math::add3(stretched_floor, [zero, one, zero]);
            dpos = math::sub3(pos0, [t0, t1, t0]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 0, 1)
            vertex = math::add3(stretched_floor, [zero, zero, one]);
            dpos = math::sub3(pos0, [t0, t0, t1]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Surflet radius is slightly larger than 3-simplex, calculate contribution from the closest 2 non-shared vertices of the nearest neighboring 3-simplex
            let center_dist = one - region_sum;
            // Find the closest two points inside the current 3-simplex
            // 0x01 => (1, 0, 0)
            // 0x02 => (0, 1, 0)
            // 0x04 => (0, 0, 1)
            let (a_point, a_dist, b_point, b_dist) = {
                if rel_coords[0] >= rel_coords[1] && rel_coords[2] > rel_coords[1] {
                    (0x01, rel_coords[0], 0x04, rel_coords[2])
                } else if rel_coords[0] < rel_coords[1] && rel_coords[2] > rel_coords[0] {
                    (0x04, rel_coords[2], 0x02, rel_coords[1])
                } else {
                    (0x01, rel_coords[0], 0x02, rel_coords[1])
                }
            };
            // If closer to (0, 0, 0) than either of the other 2 closest points
            if center_dist > a_dist || center_dist > b_dist {
                // (0, 0, 0) is one of the two closest points
                // Other closest point determines ext0 and ext1:
                // (1, 0, 0) => ext0 = (1, -1, 0), ext1 = (1, 0, -1)
                // (0, 1, 0) => ext0 = (-1, 1, 0), ext1 = (0, 1, -1)
                // (0, 0, 1) => ext0 = (-1, 0, 1), ext1 = (0, -1, 1)

                // Determine the next closest point from a and b.
                let c_point = if a_dist < b_dist { b_point } else { a_point };

                if c_point & 0x01 == 0 {
                    // c_point is either (0, 1, 0) or (0, 0, 1)
                    ext0_vertex[0] = ext0_vertex[0] - one;
                    ext0_dpos[0] = ext0_dpos[0] + one;
                } else {
                    // c_point is (1, 0, 0)
                    ext0_vertex[0] = ext0_vertex[0] + one;
                    ext1_vertex[0] = ext1_vertex[0] + one;
                    ext0_dpos[0] = ext0_dpos[0] - one;
                    ext1_dpos[0] = ext1_dpos[0] - one;
                }

                if c_point & 0x02 == 0 {
                    // c_point is either (1, 0, 0) or (0, 0, 1)
                    if c_point & 0x01 == 0 {
                        // c_point is (0, 0, 1)
                        ext1_vertex[1] = ext1_vertex[1] - one;
                        ext1_dpos[1] = ext1_dpos[1] + one;
                    } else {
                        // c_point is (1, 0, 0)
                        ext0_vertex[1] = ext0_vertex[1] - one;
                        ext0_dpos[1] = ext0_dpos[1] + one;
                    }
                } else {
                    // c_point is (0, 1, 0)
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext1_vertex[1] = ext1_vertex[1] + one;
                    ext0_dpos[1] = ext0_dpos[1] - one;
                    ext1_dpos[1] = ext1_dpos[1] - one;
                }

                if c_point & 0x04 == 0 {
                    // c_point is either (1, 0, 0) or (0, 1, 0)
                    ext1_vertex[2] = ext1_vertex[2] - one;
                    ext1_dpos[2] = ext1_dpos[2] + one;
                } else {
                    // c_point is (0, 0, 1)
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext1_vertex[2] = ext1_vertex[2] + one;
                    ext0_dpos[2] = ext0_dpos[2] - one;
                    ext1_dpos[2] = ext1_dpos[2] - one;
                }
            } else {
                // a and b are the closest points
                let c_point = a_point | b_point;

                // a and b determine ext0 and ext1:
                // (1, 0, 0), (0, 1, 0) => ext0 = (1, 1, 0), ext1 = (1, 1, -1)
                // (1, 0, 0), (0, 0, 1) => ext0 = (1, 0, 1), ext1 = (1, -1, 1)
                // (0, 1, 0), (0, 0, 1) => ext0 = (0, 1, 1), ext1 = (-1, 1, 1)

                let t0 = squish_constant + squish_constant;
                let t1 = one + t0;
                let t2 = one - squish_constant;
                let t3 = one + squish_constant;

                if c_point & 0x01 == 0 {
                    // Nearest points are (0, 1, 0) and (0, 0, 1)
                    ext1_vertex[0] = ext1_vertex[0] - one;
                    ext0_dpos[0] = ext0_dpos[0] - t0;
                    ext1_dpos[0] = ext1_dpos[0] + t2;
                } else {
                    // (1, 0, 0) is a closest point
                    ext0_vertex[0] = ext0_vertex[0] + one;
                    ext1_vertex[0] = ext1_vertex[0] + one;
                    ext0_dpos[0] = ext0_dpos[0] - t1;
                    ext1_dpos[0] = ext1_dpos[0] - t3;
                }

                if c_point & 0x02 == 0 {
                    // Nearest points are (1, 0, 0) and (0, 0, 1)
                    ext1_vertex[1] = ext1_vertex[1] - one;
                    ext0_dpos[1] = ext0_dpos[1] - t0;
                    ext1_dpos[1] = ext1_dpos[1] + t2;
                } else {
                    // (0, 1, 0) is a closest point
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext1_vertex[1] = ext1_vertex[1] + one;
                    ext0_dpos[1] = ext0_dpos[1] - t1;
                    ext1_dpos[1] = ext1_dpos[1] - t3;
                }

                if c_point & 0x04 == 0 {
                    // Nearest points are (1, 0, 0) and (0, 1, 0)
                    ext1_vertex[2] = ext1_vertex[2] - one;
                    ext0_dpos[2] = ext0_dpos[2] - t0;
                    ext1_dpos[2] = ext1_dpos[2] + t2;
                } else {
                    // (0, 0, 1) is a closest point
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext1_vertex[2] = ext1_vertex[2] + one;
                    ext0_dpos[2] = ext0_dpos[2] - t1;
                    ext1_dpos[2] = ext1_dpos[2] - t3;
                }
            }
        } else if region_sum >= two {
            // We're inside the tetrahedron (3-Simplex) at (1, 1, 1)
            let t0 = two * squish_constant;
            let t1 = one + two * squish_constant;
            let t2 = t1 + squish_constant;

            // Contribution at (1, 1, 0)
            vertex = math::add3(stretched_floor, [one, one, zero]);
            dpos = math::sub3(pos0, [t1, t1, t0]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 0, 1)
            vertex = math::add3(stretched_floor, [one, zero, one]);
            dpos = math::sub3(pos0, [t1, t0, t1]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 1)
            vertex = math::add3(stretched_floor, [zero, one, one]);
            dpos = math::sub3(pos0, [t0, t1, t1]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 1, 1)
            vertex = math::add3(stretched_floor, [one, one, one]);
            dpos = math::sub3(pos0, [t2, t2, t2]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Surflet radius is slightly larger than 3-simplex, calculate contribution from the closest 2 non-shared vertices of the nearest neighboring 3-simplex
            let center_dist = three - region_sum;
            // Find the closest two points inside the current 3-simplex
            // 0x03 => (1, 1, 0)
            // 0x05 => (1, 0, 1)
            // 0x06 => (0, 1, 1)
            let (a_point, a_dist, b_point, b_dist) = {
                if rel_coords[0] <= rel_coords[1] && rel_coords[2] < rel_coords[1] {
                    (0x06, rel_coords[0], 0x03, rel_coords[2])
                } else if rel_coords[0] > rel_coords[1] && rel_coords[2] < rel_coords[0] {
                    (0x03, rel_coords[2], 0x05, rel_coords[1])
                } else {
                    (0x06, rel_coords[0], 0x05, rel_coords[1])
                }
            };
            // If closer to (1, 1, 1) than either of the other 2 closest points
            if center_dist < a_dist || center_dist < b_dist {
                // (1, 1, 1) is one of the two closest points
                // Other closest point determines ext0 and ext1:
                // (1, 1, 0) => ext0 = (2, 1, 0), ext1 = (1, 2, 0)
                // (1, 0, 1) => ext0 = (2, 0, 1), ext1 = (1, 0, 2)
                // (0, 1, 1) => ext0 = (0, 2, 1), ext1 = (0, 1, 2)

                // Determine the next closest point from a and b.
                let c_point = if b_dist < a_dist { b_point } else { a_point };

                let t0 = squish_constant + squish_constant + squish_constant;
                let t1 = one + t0;
                let t2 = two + t0;

                if c_point & 0x01 != 0 {
                    // c_point is either (1, 1, 0) or (1, 0, 1)
                    ext0_vertex[0] = ext0_vertex[0] + two;
                    ext1_vertex[0] = ext1_vertex[0] + one;
                    ext0_dpos[0] = ext0_dpos[0] - t2;
                    ext1_dpos[0] = ext1_dpos[0] - t1;
                } else {
                    // c_point is (0, 1, 1)
                    ext0_dpos[0] = ext0_dpos[0] - t0;
                    ext1_dpos[0] = ext1_dpos[0] - t0;
                }

                if c_point & 0x02 != 0 {
                    // c_point is either (1, 1, 0) or (0, 1, 1)
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext1_vertex[1] = ext1_vertex[1] + one;
                    ext0_dpos[1] = ext0_dpos[1] - t1;
                    ext1_dpos[1] = ext1_dpos[1] - t1;
                    if c_point & 0x01 != 0 {
                        // c_point is (1, 1, 0)
                        ext1_vertex[1] = ext1_vertex[1] + one;
                        ext1_dpos[1] = ext1_dpos[1] - one;
                    } else {
                        // c_point is (0, 1, 1)
                        ext0_vertex[1] = ext0_vertex[1] + one;
                        ext0_dpos[1] = ext0_dpos[1] - one;
                    }
                } else {
                    // c_point is (1, 0, 1)
                    ext0_dpos[1] = ext0_dpos[1] - t0;
                    ext1_dpos[1] = ext1_dpos[1] - t0;
                }

                if c_point & 0x04 != 0 {
                    // c_point is either (1, 0, 1) or (0, 1, 1)
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext1_vertex[2] = ext1_vertex[2] + two;
                    ext0_dpos[2] = ext0_dpos[2] - t1;
                    ext1_dpos[2] = ext1_dpos[2] - t2;
                } else {
                    // c_point is (1, 1, 0)
                    ext0_dpos[2] = ext0_dpos[2] - t0;
                    ext1_dpos[2] = ext1_dpos[2] - t0;
                }
            } else {
                // a and b determine ext0 and ext1:
                // (1, 1, 0), (1, 0, 1) => ext0 = (1, 0, 0), ext1 = (2, 0, 0)
                // (1, 1, 0), (0, 1, 1) => ext0 = (0, 1, 0), ext1 = (0, 2, 0)
                // (1, 0, 1), (0, 1, 1) => ext0 = (0, 0, 1), ext1 = (0, 0, 2)

                // a and b are the closest points
                let c_point = a_point & b_point;

                let t0 = squish_constant;
                let t1 = one + t1;
                let t2 = squish_constant + squish_constant;
                let t3 = two + t2;

                if c_point & 0x01 != 0 {
                    // a, b are (1, 1, 0), (1, 0, 1)
                    ext0_vertex[0] = ext0_vertex[0] + one;
                    ext1_vertex[0] = ext1_vertex[0] + two;
                    ext0_dpos[0] = ext0_dpos[0] - t1;
                    ext1_dpos[0] = ext1_dpos[0] - t3;
                } else {
                    ext0_dpos[0] = ext0_dpos[0] - t0;
                    ext1_dpos[0] = ext1_dpos[0] - t2;
                }

                if c_point & 0x02 != 0 {
                    // a, b are (1, 1, 0), (0, 1, 1)
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext1_vertex[1] = ext1_vertex[1] + two;
                    ext0_dpos[1] = ext0_dpos[1] - t1;
                    ext1_dpos[1] = ext1_dpos[1] - t3;
                } else {
                    ext0_dpos[1] = ext0_dpos[1] - t0;
                    ext1_dpos[1] = ext1_dpos[1] - t2;
                }

                if c_point & 0x04 != 0 {
                    // a, b are (1, 0, 1), (0, 1, 1)
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext1_vertex[2] = ext1_vertex[2] + two;
                    ext0_dpos[2] = ext0_dpos[2] - t1;
                    ext1_dpos[2] = ext1_dpos[2] - t3;
                } else {
                    ext0_dpos[2] = ext0_dpos[2] - t0;
                    ext1_dpos[2] = ext1_dpos[2] - t2;
                }
            }
        } else {
            // We're inside the octahedron (Rectified 3-Simplex) inbetween.
            let t0 = squish_constant;
            let t1 = one + squish_constant;
            let t2 = two * squish_constant;
            let t3 = one + two * squish_constant;

            // Contribution at (1, 0, 0)
            vertex = math::add3(stretched_floor, [one, zero, zero]);
            dpos = math::sub3(pos0, [t1, t0, t0]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 0)
            vertex = math::add3(stretched_floor, [zero, one, zero]);
            dpos = math::sub3(pos0, [t0, t1, t0]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 0, 1)
            vertex = math::add3(stretched_floor, [zero, zero, one]);
            dpos = math::sub3(pos0, [t0, t0, t1]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 1, 0)
            vertex = math::add3(stretched_floor, [one, one, zero]);
            dpos = math::sub3(pos0, [t3, t3, t2]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 0, 1)
            vertex = math::add3(stretched_floor, [one, zero, one]);
            dpos = math::sub3(pos0, [t3, t2, t3]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 1)
            vertex = math::add3(stretched_floor, [zero, one, one]);
            dpos = math::sub3(pos0, [t2, t3, t3]);
            value = value + gradient(&self.perm_table, vertex, dpos);

            // Surflet radius is slightly larger than 3-simplex, calculate contribution from the closest 2 non-shared vertices of the nearest neighboring 3-simplex

            // Find the closest two points inside the current 3-simplex
            // 0x01 => (1, 0, 0)
            // 0x02 => (0, 1, 0)
            // 0x03 => (1, 1, 0)
            // 0x02 => (0, 0, 1)
            // 0x05 => (1, 0, 1)
            // 0x06 => (0, 1, 1)

            let (a_point, _a_dist, a_is_further, b_point, _b_dist, b_is_further) = {
                // Pick closest of (0,0,1) and (1,1,0) for point a
                let a_temp = rel_coords[0] + rel_coords[1];
                let (mut a_point, mut a_dist, mut a_is_further) = if a_temp > one {
                    (0x03, a_temp - one, true)
                } else {
                    (0x04, one - a_temp, false)
                };

                // Pick closest of (0,1,0) and (1,0,1) for point b
                let b_temp = rel_coords[0] + rel_coords[2];
                let (mut b_point, mut b_dist, mut b_is_further) = if b_temp > one {
                    (0x05, b_temp - one, true)
                } else {
                    (0x02, one - b_temp, false)
                };

                // If either of (1,0,0) and (0,1,1) are closer than a or b, replace the farther of those two
                let c_temp = rel_coords[1] + rel_coords[2];
                if c_temp > one {
                    let c_dist = c_temp - one;
                    if a_dist <= b_dist && a_dist < c_dist {
                        a_point = 0x06;
                        a_dist = c_dist;
                        a_is_further = true;
                    } else if a_dist > b_dist && b_dist < c_dist {
                        b_point = 0x06;
                        b_dist = c_dist;
                        b_is_further = true;
                    }
                } else {
                    let c_dist = one - c_temp;
                    if a_dist <= b_dist && a_dist < c_dist {
                        a_point = 0x01;
                        a_dist = c_dist;
                        a_is_further = false;
                    } else if a_dist > b_dist && b_dist < c_dist {
                        b_point = 0x01;
                        b_dist = c_dist;
                        b_is_further = false;
                    }
                }

                (a_point, a_dist, a_is_further, b_point, b_dist, b_is_further)
            };

            if a_is_further == b_is_further {
                // a and b are on the same side of the 3-simplex
                if a_is_further {
                    // Both points on the side of (1, 1, 1)

                    // (1, 1, 1) is ext0
                    // The common axis between a and b determines ext0:
                    // (1, 0, 0) => ext1 = (2, 0, 0)
                    // (0, 1, 0) => ext1 = (0, 2, 0)
                    // (0, 0, 1) => ext1 = (0, 0, 2)

                    let t0 = squish_constant + squish_constant;
                    let t1 = two + t0;
                    let t2 = one + squish_constant + t0;

                    // ext0 = (1, 1, 1)
                    ext0_vertex[0] = ext0_vertex[0] + one;
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext0_dpos[0] = ext0_dpos[0] - t2;
                    ext0_dpos[1] = ext0_dpos[1] - t2;
                    ext0_dpos[2] = ext0_dpos[2] - t2;

                    // ext1 is based on the common axis between a and b
                    let c_point = a_point & b_point;

                    if c_point & 0x01 != 0 {
                        // a and b share (1, 0, 0)
                        ext1_vertex[0] = ext1_vertex[0] + two;
                        ext1_dpos[0] = ext1_dpos[0] - t1;
                        ext1_dpos[1] = ext1_dpos[1] - t0;
                        ext1_dpos[2] = ext1_dpos[2] - t0;
                    } else if c_point & 0x02 != 0 {
                        // a and b share (0, 1, 0)
                        ext1_vertex[1] = ext1_vertex[1] + two;
                        ext1_dpos[0] = ext1_dpos[0] - t0;
                        ext1_dpos[1] = ext1_dpos[1] - t1;
                        ext1_dpos[2] = ext1_dpos[2] - t0;
                    } else {
                        // a and b share (0, 0, 1)
                        ext1_vertex[2] = ext1_vertex[2] + two;
                        ext1_dpos[0] = ext1_dpos[0] - t0;
                        ext1_dpos[1] = ext1_dpos[1] - t0;
                        ext1_dpos[2] = ext1_dpos[2] - t1;
                    }
                } else {
                    // Both points on the side of (0, 0, 0)

                    // (0, 0, 0) is ext0
                    // The axis that a and b lack determines ext0:
                    // (1, 0, 0) => ext1 = (-1, 1, 1)
                    // (0, 1, 0) => ext1 = (1, -1, 1)
                    // (0, 0, 1) => ext1 = (1, 1, -1)

                    let t0 = one + squish_constant;
                    let t1 = one - squish_constant;

                    // ext1 is based on the axis that a and b lack
                    let c_point = a_point | b_point;
                    if c_point & 0x01 == 0 {
                        // a and b lack (1, 0, 0)
                        ext1_vertex[0] = ext1_vertex[0] - one;
                        ext1_vertex[1] = ext1_vertex[1] + one;
                        ext1_vertex[2] = ext1_vertex[2] + one;
                        ext1_dpos[0] = ext1_dpos[0] + t1;
                        ext1_dpos[1] = ext1_dpos[1] - t0;
                        ext1_dpos[2] = ext1_dpos[2] - t0;
                    } else if c_point & 0x02 == 0 {
                        // a and b lack (0, 1, 0)
                        ext1_vertex[0] = ext1_vertex[0] + one;
                        ext1_vertex[1] = ext1_vertex[1] - one;
                        ext1_vertex[2] = ext1_vertex[2] + one;
                        ext1_dpos[0] = ext1_dpos[0] - t0;
                        ext1_dpos[1] = ext1_dpos[1] + t1;
                        ext1_dpos[2] = ext1_dpos[2] - t0;
                    } else {
                        // a and b lack (0, 0, 1)
                        ext1_vertex[0] = ext1_vertex[0] + one;
                        ext1_vertex[1] = ext1_vertex[1] + one;
                        ext1_vertex[2] = ext1_vertex[2] - one;
                        ext1_dpos[0] = ext1_dpos[0] - t0;
                        ext1_dpos[1] = ext1_dpos[1] - t0;
                        ext1_dpos[2] = ext1_dpos[2] + t1;
                    }
                }
            } else {
                // One point on the side of (0, 0, 0), one on the side of (1, 1, 1)

                // The axis that the point on the side of (1, 1, 1) lacks determines ext0:
                // (0, x, x) => ext0 = (-1, 1, 1)
                // (x, 0, x) => ext0 = (1, -1, 1)
                // (x, x, 0) => ext0 = (1, 1, -1)
                // The axis that the point on the side of (0, 0, 0) has determines ext1:
                // (1, 0, 0) => ext1 = (2, 0, 0)
                // (0, 1, 0) => ext1 = (0, 2, 0)
                // (0, 0, 1) => ext1 = (0, 0, 2)

                let t0 = one + squish_constant;
                let t1 = one - squish_constant;
                let t2 = squish_constant + squish_constant;

                // c_point0 takes the point on the side of (1, 1, 1)
                // c_point1 takes the point on the side of (0, 0, 0)
                let (c_point0, c_point1) = if a_is_further {
                    (a_point, b_point)
                } else {
                    (b_point, a_point)
                };

                // ext0 is (-1, 1, 1), (1, -1, 1), or (1, 1, -1)
                if c_point0 & 0x01 == 0 {
                    ext0_vertex[0] = ext0_vertex[0] - one;
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext0_dpos[0] = ext0_dpos[0] + t1;
                    ext0_dpos[1] = ext0_dpos[1] - t0;
                    ext0_dpos[2] = ext0_dpos[2] - t0;
                } else if c_point0 & 0x02 == 0 {
                    ext0_vertex[0] = ext0_vertex[0] + one;
                    ext0_vertex[1] = ext0_vertex[1] - one;
                    ext0_vertex[2] = ext0_vertex[2] + one;
                    ext0_dpos[0] = ext0_dpos[0] - t0;
                    ext0_dpos[1] = ext0_dpos[1] + t1;
                    ext0_dpos[2] = ext0_dpos[2] - t0;
                } else {
                    ext0_vertex[0] = ext0_vertex[0] + one;
                    ext0_vertex[1] = ext0_vertex[1] + one;
                    ext0_vertex[2] = ext0_vertex[2] - one;
                    ext0_dpos[0] = ext0_dpos[0] - t0;
                    ext0_dpos[1] = ext0_dpos[1] - t0;
                    ext0_dpos[2] = ext0_dpos[2] + t1;
                }

                // One contribution is a permutation of (0,0,2)
                ext1_dpos[0] = ext1_dpos[0] - t2;
                ext1_dpos[1] = ext1_dpos[1] - t2;
                ext1_dpos[2] = ext1_dpos[2] - t2;
                if c_point1 & 0x01 != 0 {
                    ext1_vertex[0] = ext1_vertex[0] + two;
                    ext1_dpos[0] = ext1_dpos[0] - two;
                } else if c_point1 & 0x02 != 0 {
                    ext1_vertex[1] = ext1_vertex[1] + two;
                    ext1_dpos[1] = ext1_dpos[1] - two;
                } else {
                    ext1_vertex[2] = ext1_vertex[2] + two;
                    ext1_dpos[2] = ext1_dpos[2] - two;
                }
            }
        }

        // Contribution at ext0
        value = value + gradient(&self.perm_table, ext0_vertex, ext0_dpos);

        // Contribution at ext1
        value = value + gradient(&self.perm_table, ext1_vertex, ext1_dpos);

        value * Self::NORM_CONSTANT_3D
    }
}

/// 4-dimensional [`OpenSimplex` Noise](http://uniblock.tumblr.com/post/97868843242/noise)
///
/// This is a slower but higher quality form of gradient noise than `Perlin` 4D.
impl NoiseFn<[f64; 4]> for OpenSimplex {
    fn get(&self, point: [f64; 4]) -> f64 {
        #[inline(always)]
        fn gradient(perm_table: &PermutationTable, vertex: [f64; 4], pos: [f64; 4]) -> f64 {
            let attn = 2.0 - math::dot4(pos, pos);
            if attn > 0.0 {
                let index = perm_table.get4(math::to_isize4(vertex));
                let vec = gradient::get4(index);
                attn.powi(4) * math::dot4(pos, vec)
            } else {
                0.0
            }
        }

        // Place input coordinates on simplectic h1.0ycomb.
        let stretch_offset = math::fold4(point, Add::add) * Self::STRETCH_CONSTANT_4D;
        let stretched = math::map4(point, |v| v + stretch_offset);

        // Floor to get simplectic h1.0ycomb coordinates of rhombo-hypercube
        // super-cell origin.
        let stretched_floor = math::map4(stretched, f64::floor);

        // Skew out to get actual coordinates of stretched rhombo-hypercube origin.
        // We'll need these later.
        let squish_offset = math::fold4(stretched_floor, Add::add) * Self::SQUISH_CONSTANT_4D;
        let skewed_floor = math::map4(stretched_floor, |v| v + squish_offset);

        // Compute simplectic h1.0ycomb coordinates relative to rhombo-hypercube
        // origin.
        let rel_coords = math::sub4(stretched, stretched_floor);

        // Sum those together to get a value that determines which region
        // we're in.
        let region_sum = math::fold4(rel_coords, Add::add);

        // Position relative to origin point.
        let mut pos0 = math::sub4(point, skewed_floor);

        let mut value = 0.0;
        if region_sum <= 1.0 {
            // We're inside the pentachoron (4-Simplex) at (0, 0, 0, 0)

            // Contribution at (0, 0, 0, 0)
            value += gradient(&self.perm_table, stretched_floor, pos0);

            // Contribution at (1, 0, 0, 0)
            let pos1;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 0.0, 0.0]);
                pos1 = math::sub4(
                    pos0,
                    [
                        1.0 + Self::SQUISH_CONSTANT_4D,
                        Self::SQUISH_CONSTANT_4D,
                        Self::SQUISH_CONSTANT_4D,
                        Self::SQUISH_CONSTANT_4D,
                    ],
                );
                value += gradient(&self.perm_table, vertex, pos1);
            }

            // Contribution at (0, 1, 0, 0)
            let pos2;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 0.0, 0.0]);
                pos2 = [pos1[0] + 1.0, pos1[1] - 1.0, pos1[2], pos1[3]];
                value += gradient(&self.perm_table, vertex, pos2);
            }

            // Contribution at (0, 0, 1, 0)
            let pos3;
            {
                let vertex = math::add4(stretched_floor, [0.0, 0.0, 1.0, 0.0]);
                pos3 = [pos2[0], pos1[1], pos1[2] - 1.0, pos1[3]];
                value += gradient(&self.perm_table, vertex, pos3);
            }

            // Contribution at (0, 0, 0, 1)
            let pos4;
            {
                let vertex = math::add4(stretched_floor, [0.0, 0.0, 0.0, 1.0]);
                pos4 = [pos2[0], pos1[1], pos1[2], pos1[3] - 1.0];
                value += gradient(&self.perm_table, vertex, pos4);
            }
        } else if region_sum >= 3.0 {
            // We're inside the pentachoron (4-Simplex) at (1, 1, 1, 1)
            let squish_constant_3 = 3.0 * Self::SQUISH_CONSTANT_4D;

            // Contribution at (1, 1, 1, 0)
            let pos4;
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 1.0, 0.0]);
                pos4 = math::sub4(
                    pos0,
                    [
                        1.0 + squish_constant_3,
                        1.0 + squish_constant_3,
                        1.0 + squish_constant_3,
                        squish_constant_3,
                    ],
                );
                value += gradient(&self.perm_table, vertex, pos4);
            }

            // Contribution at (1, 1, 0, 1)
            let pos3;
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 0.0, 1.0]);
                pos3 = [pos4[0], pos4[1], pos4[2] + 1.0, pos4[3] - 1.0];
                value += gradient(&self.perm_table, vertex, pos3);
            }

            // Contribution at (1, 0, 1, 1)
            let pos2;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 1.0, 1.0]);
                pos2 = [pos4[0], pos4[1] + 1.0, pos4[2], pos3[3]];
                value += gradient(&self.perm_table, vertex, pos2);
            }

            // Contribution at (0, 1, 1, 1)
            let pos1;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 1.0, 1.0]);
                pos1 = [pos0[0] - squish_constant_3, pos4[1], pos4[2], pos3[3]];
                value += gradient(&self.perm_table, vertex, pos1);
            }

            // Contribution at (1, 1, 1, 1)
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 1.0, 1.0]);
                pos0[0] = pos4[0] - Self::SQUISH_CONSTANT_4D;
                pos0[1] = pos4[1] - Self::SQUISH_CONSTANT_4D;
                pos0[2] = pos4[2] - Self::SQUISH_CONSTANT_4D;
                pos0[3] = pos3[3] - Self::SQUISH_CONSTANT_4D;
                value += gradient(&self.perm_table, vertex, pos0);
            }
        } else if region_sum <= 2.0 {
            // We're inside the first dispentachoron (Rectified 4-Simplex)

            // Contribution at (1, 0, 0, 0)
            let pos1;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 0.0, 0.0]);
                pos1 = math::sub4(
                    pos0,
                    [
                        1.0 + Self::SQUISH_CONSTANT_4D,
                        Self::SQUISH_CONSTANT_4D,
                        Self::SQUISH_CONSTANT_4D,
                        Self::SQUISH_CONSTANT_4D,
                    ],
                );
                value += gradient(&self.perm_table, vertex, pos1);
            }

            // Contribution at (0, 1, 0, 0)
            let pos2;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 0.0, 0.0]);
                pos2 = [pos1[0] + 1.0, pos1[1] - 1.0, pos1[2], pos1[3]];
                value += gradient(&self.perm_table, vertex, pos2);
            }

            // Contribution at (0, 0, 1, 0)
            let pos3;
            {
                let vertex = math::add4(stretched_floor, [0.0, 0.0, 1.0, 0.0]);
                pos3 = [pos2[0], pos1[1], pos1[2] - 1.0, pos1[3]];
                value += gradient(&self.perm_table, vertex, pos3);
            }

            // Contribution at (0, 0, 0, 1)
            let pos4;
            {
                let vertex = math::add4(stretched_floor, [0.0, 0.0, 0.0, 1.0]);
                pos4 = [pos2[0], pos1[1], pos1[2], pos1[3] - 1.0];
                value += gradient(&self.perm_table, vertex, pos4);
            }

            // Contribution at (1, 1, 0, 0)
            let pos5;
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 0.0, 0.0]);
                pos5 = [
                    pos1[0] - Self::SQUISH_CONSTANT_4D,
                    pos2[1] - Self::SQUISH_CONSTANT_4D,
                    pos1[2] - Self::SQUISH_CONSTANT_4D,
                    pos1[3] - Self::SQUISH_CONSTANT_4D,
                ];
                value += gradient(&self.perm_table, vertex, pos5);
            }

            // Contribution at (1, 0, 1, 0)
            let pos6;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 1.0, 0.0]);
                pos6 = [pos5[0], pos5[1] + 1.0, pos5[2] - 1.0, pos5[3]];
                value += gradient(&self.perm_table, vertex, pos6);
            }

            // Contribution at (1, 0, 0, 1)
            let pos7;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 0.0, 1.0]);
                pos7 = [pos5[0], pos6[1], pos5[2], pos5[3] - 1.0];
                value += gradient(&self.perm_table, vertex, pos7);
            }

            // Contribution at (0, 1, 1, 0)
            let pos8;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 1.0, 0.0]);
                pos8 = [pos5[0] + 1.0, pos5[1], pos6[2], pos5[3]];
                value += gradient(&self.perm_table, vertex, pos8);
            }

            // Contribution at (0, 1, 0, 1)
            let pos9;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 0.0, 1.0]);
                pos9 = [pos8[0], pos5[1], pos5[2], pos7[3]];
                value += gradient(&self.perm_table, vertex, pos9);
            }

            // Contribution at (0, 0, 1, 1)
            let pos10;
            {
                let vertex = math::add4(stretched_floor, [0.0, 0.0, 1.0, 1.0]);
                pos10 = [pos8[0], pos6[1], pos6[2], pos7[3]];
                value += gradient(&self.perm_table, vertex, pos10);
            }
        } else {
            // We're inside the second dispentachoron (Rectified 4-Simplex)
            let squish_constant_3 = 3.0 * Self::SQUISH_CONSTANT_4D;

            // Contribution at (1, 1, 1, 0)
            let pos4;
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 1.0, 0.0]);
                pos4 = math::sub4(
                    pos0,
                    [
                        1.0 + squish_constant_3,
                        1.0 + squish_constant_3,
                        1.0 + squish_constant_3,
                        squish_constant_3,
                    ],
                );
                value += gradient(&self.perm_table, vertex, pos4);
            }

            // Contribution at (1, 1, 0, 1)
            let pos3;
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 0.0, 1.0]);
                pos3 = [pos4[0], pos4[1], pos4[2] + 1.0, pos4[3] - 1.0];
                value += gradient(&self.perm_table, vertex, pos3);
            }

            // Contribution at (1, 0, 1, 1)
            let pos2;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 1.0, 1.0]);
                pos2 = [pos4[0], pos4[1] + 1.0, pos4[2], pos3[3]];
                value += gradient(&self.perm_table, vertex, pos2);
            }

            // Contribution at (0, 1, 1, 1)
            let pos1;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 1.0, 1.0]);
                pos1 = [pos4[0] + 1.0, pos4[1], pos4[2], pos3[3]];
                value += gradient(&self.perm_table, vertex, pos1);
            }

            // Contribution at (1, 1, 0, 0)
            let pos5;
            {
                let vertex = math::add4(stretched_floor, [1.0, 1.0, 0.0, 0.0]);
                pos5 = [
                    pos4[0] + Self::SQUISH_CONSTANT_4D,
                    pos4[1] + Self::SQUISH_CONSTANT_4D,
                    pos3[2] + Self::SQUISH_CONSTANT_4D,
                    pos4[3] + Self::SQUISH_CONSTANT_4D,
                ];
                value += gradient(&self.perm_table, vertex, pos5);
            }

            // Contribution at (1, 0, 1, 0)
            let pos6;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 1.0, 0.0]);
                pos6 = [pos5[0], pos5[1] + 1.0, pos5[2] - 1.0, pos5[3]];
                value += gradient(&self.perm_table, vertex, pos6);
            }

            // Contribution at (1, 0, 0, 1)
            let pos7;
            {
                let vertex = math::add4(stretched_floor, [1.0, 0.0, 0.0, 1.0]);
                pos7 = [pos5[0], pos6[1], pos5[2], pos5[3] - 1.0];
                value += gradient(&self.perm_table, vertex, pos7);
            }

            // Contribution at (0, 1, 1, 0)
            let pos8;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 1.0, 0.0]);
                pos8 = [pos5[0] + 1.0, pos5[1], pos6[2], pos5[3]];
                value += gradient(&self.perm_table, vertex, pos8);
            }

            // Contribution at (0, 1, 0, 1)
            let pos9;
            {
                let vertex = math::add4(stretched_floor, [0.0, 1.0, 0.0, 1.0]);
                pos9 = [pos8[0], pos5[1], pos5[2], pos7[3]];
                value += gradient(&self.perm_table, vertex, pos9);
            }

            // Contribution at (0, 0, 1, 1)
            let pos10;
            {
                let vertex = math::add4(stretched_floor, [0.0, 0.0, 1.0, 1.0]);
                pos10 = [pos8[0], pos6[1], pos6[2], pos7[3]];
                value += gradient(&self.perm_table, vertex, pos10);
            }
        }

        value * Self::NORM_CONSTANT_4D
    }
}
