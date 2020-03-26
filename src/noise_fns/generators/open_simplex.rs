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

        //                           ( 1, -1)
        //                          /    |
        //                        /  D   |
        //                      /        |
        //              ( 0,  0) --- ( 1,  0) --- ( 2,  0)
        //             /    |       /    |       /
        //           /  E   |  A  /  B   |  C  /
        //         /        |   /        |   /
        // (-1,  1) --- ( 0,  1) --- ( 1,  1)
        //                  |       /
        //                  |  F  /
        //                  |   /
        //              ( 0,  2)

        let ext_vertex;
        let ext_dpos;

        // See the graph for an intuitive explanation; the sum of `x` and `y` is
        // only greater than `1` if we're on Region B.
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

        // Place input coordinates on simplectic honeycomb.
        let stretch_offset = math::fold3(point, Add::add) * Self::STRETCH_CONSTANT_3D;
        let stretched = math::map3(point, |v| v + stretch_offset);

        // Floor to get simplectic honeycomb coordinates of rhombohedron (stretched cube) super-cell origin.
        let stretched_floor = math::map3(stretched, f64::floor);

        // Skew out to get actual coordinates of rhombohedron origin. We'll need these later.
        let squish_offset = math::fold3(stretched_floor, Add::add) * Self::SQUISH_CONSTANT_3D;
        let skewed_floor = math::map3(stretched_floor, |v| v + squish_offset);

        // Compute simplectic honeycomb coordinates relative to rhombohedral origin.
        let relative_coords = math::sub3(stretched, stretched_floor);

        // Sum those together to get a value that determines which region we're in.
        let region_sum = math::fold3(relative_coords, Add::add);

        // Positions relative to origin point.
        let position_delta = math::sub3(point, skewed_floor);

        let mut value = 0.0;

        let mut vertex;
        let mut dpos;

        let mut dx_ext0 = 0.0;
        let mut dy_ext0 = 0.0;
        let mut dz_ext0 = 0.0;
        let mut dx_ext1 = 0.0;
        let mut dy_ext1 = 0.0;
        let mut dz_ext1 = 0.0;
        let mut xsv_ext0 = 0.0;
        let mut ysv_ext0 = 0.0;
        let mut zsv_ext0 = 0.0;
        let mut xsv_ext1 = 0.0;
        let mut ysv_ext1 = 0.0;
        let mut zsv_ext1 = 0.0;

        if region_sum <= 1.0 {
            // Inside the tetrahedron (3-Simplex) at (0, 0, 0)

            // Determine which two of (0, 0, 1), (0, 1, 0), or (1, 0, 0) are closest
            let mut a_point = 0x01u8;
            let mut b_point = 0x02u8;
            let mut a_score = relative_coords[0];
            let mut b_score = relative_coords[1];

            if a_score >= b_score && relative_coords[2] > b_score {
                b_score = relative_coords[2];
                b_point = 0x04;
            } else if a_score < b_score && relative_coords[2] > a_score {
                a_score = relative_coords[2];
                a_point = 0x04;
            }

            // Now determine the two lattice points not part of the tetrahedron that may contribute.
            // This depends on the closest two tetrahedral vertices, including (0, 0, 0)
            let wins = 1.0 - region_sum;

            if wins > a_score || wins > b_score {
                // (0, 0, 0) is one of the closest tetrahedral vertices
                let c = if b_score > a_score {
                    b_point
                } else {
                    // The other vertex is closest
                    a_point
                };

                if (c & 0x01) == 0 {
                    xsv_ext0 = stretched_floor[0] - 1.0;
                    xsv_ext1 = stretched_floor[0];
                    dx_ext0 = position_delta[0] + 1.0;
                    dx_ext1 = position_delta[0];
                } else {
                    xsv_ext0 = stretched_floor[0] + 1.0;
                    xsv_ext1 = xsv_ext0;
                    dx_ext0 = position_delta[0] - 1.0;
                    dx_ext1 = dx_ext0;
                }

                if (c & 0x02) == 0 {
                    ysv_ext0 = stretched_floor[1];
                    ysv_ext1 = stretched_floor[1];
                    dy_ext0 = position_delta[1];
                    dy_ext1 = position_delta[1];

                    if (c & 0x01) == 0 {
                        ysv_ext1 -= 1.0;
                        dy_ext1 += 1.0;
                    } else {
                        ysv_ext0 -= 1.0;
                        dy_ext0 += 1.0;
                    }
                } else {
                    ysv_ext0 = stretched_floor[1] + 1.0;
                    ysv_ext1 = ysv_ext0;
                    dy_ext0 = position_delta[1] - 1.0;
                    dy_ext1 = dy_ext0;
                }

                if (c & 0x04) == 0 {
                    zsv_ext0 = stretched_floor[2];
                    zsv_ext1 = stretched_floor[2] - 1.0;
                    dz_ext0 = position_delta[2];
                    dz_ext1 = position_delta[2] + 1.0;
                } else {
                    zsv_ext0 = stretched_floor[2] + 1.0;
                    zsv_ext1 = zsv_ext0;
                    dz_ext0 = position_delta[2] - 1.0;
                    dz_ext1 = dz_ext0;
                }
            } else {
                // (0, 0, 0) is not one of the closest two tetrahedral vertices

                // The two extra vertices are determined by the closest two
                let c = (a_point | b_point) as u8;

                if (c & 0x01) == 0 {
                    xsv_ext0 = stretched_floor[0];
                    xsv_ext1 = stretched_floor[0] - 1.0;
                    dx_ext0 = position_delta[0] - 2.0 * Self::SQUISH_CONSTANT_3D;
                    dx_ext1 = position_delta[0] + 1.0 - Self::SQUISH_CONSTANT_3D;
                } else {
                    xsv_ext0 = stretched_floor[0] + 1.0;
                    xsv_ext1 = xsv_ext0;
                    dx_ext0 = position_delta[0] - 1.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                    dx_ext1 = position_delta[0] - 1.0 - Self::SQUISH_CONSTANT_3D;
                }

                if (c & 0x02) == 0 {
                    ysv_ext0 = stretched_floor[1];
                    ysv_ext1 = stretched_floor[1] - 1.0;
                    dy_ext0 = position_delta[1] - 2.0 * Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] + 1.0 - Self::SQUISH_CONSTANT_3D;
                } else {
                    ysv_ext0 = stretched_floor[1] + 1.0;
                    ysv_ext1 = ysv_ext0;
                    dy_ext0 = position_delta[1] - 1.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] - 1.0 - Self::SQUISH_CONSTANT_3D;
                }

                if (c & 0x04) == 0 {
                    zsv_ext0 = stretched_floor[2];
                    zsv_ext1 = stretched_floor[2] - 1.0;
                    dz_ext0 = position_delta[2] - 2.0 * Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] + 1.0 - Self::SQUISH_CONSTANT_3D;
                } else {
                    zsv_ext0 = stretched_floor[2] + 1.0;
                    zsv_ext1 = xsv_ext0;
                    dz_ext0 = position_delta[2] - 1.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] - 1.0 - Self::SQUISH_CONSTANT_3D;
                }
            }

            let t0 = Self::SQUISH_CONSTANT_3D;
            let t1 = Self::SQUISH_CONSTANT_3D + 1.0;

            // Contribution at (0, 0, 0)
            vertex = math::add3(stretched_floor, [0.0, 0.0, 0.0]);
            dpos = math::sub3(position_delta, [0.0, 0.0, 0.0]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 0, 0)
            vertex = math::add3(stretched_floor, [1.0, 0.0, 0.0]);
            dpos = math::sub3(position_delta, [t1, t0, t0]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 0)
            vertex = math::add3(stretched_floor, [0.0, 1.0, 0.0]);
            dpos = math::sub3(position_delta, [t0, t1, t0]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 0, 1)
            vertex = math::add3(stretched_floor, [0.0, 0.0, 1.0]);
            dpos = math::sub3(position_delta, [t0, t0, t1]);
            value += gradient(&self.perm_table, vertex, dpos);
        } else if region_sum >= 2.0 {
            // Inside the tetrahedron (3-Simplex) at (1, 1, 1)

            // Determine which two tetrahedral vertices are the closest, out of (1, 1, 0), (1, 0, 1), (0, 1, 1), but not (1, 1, 1)
            let mut a_point = 0x06u8;
            let mut b_point = 0x05u8;
            let mut a_score = relative_coords[0];
            let mut b_score = relative_coords[1];

            if a_score <= b_score && relative_coords[2] < b_score {
                b_score = relative_coords[2];
                b_point = 0x03;
            } else {
                a_score = relative_coords[2];
                a_point = 0x03;
            }

            // Now determine the two lattice points not part of the tetrahedron that may contribute.
            // This depends on the closest two tetrahedral vertices, including (1, 1, 1).
            let wins = 3.0 - region_sum;
            if wins < a_score || wins < b_score {
                // (1, 1, 1) is one of the two closest tetrahedral vertices.
                let c = if b_score < a_score {
                    b_point
                } else {
                    a_point
                };

                if (c & 0x01) != 0 {
                    xsv_ext0 = stretched_floor[0] + 2.0;
                    xsv_ext1 = stretched_floor[0] + 1.0;
                    dx_ext0 = position_delta[0] - 2.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dx_ext1 = position_delta[0] - 1.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                } else {
                    xsv_ext0 = stretched_floor[0];
                    xsv_ext1 = stretched_floor[0];
                    dx_ext0 = position_delta[0] - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dx_ext1 = dx_ext0;
                }

                if (c & 0x02) != 0 {
                    ysv_ext0 = stretched_floor[1] + 1.0;
                    ysv_ext1 = stretched_floor[1];
                    dy_ext0 = position_delta[1] - 1.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = dy_ext0;

                    if (c & 0x01) != 0 {
                        ysv_ext1 += 1.0;
                        dx_ext1 -= 1.0;
                    } else {
                        ysv_ext0 += 1.0;
                        dy_ext0 -= 1.0;
                    }
                } else {
                    ysv_ext0 = stretched_floor[1];
                    ysv_ext1 = stretched_floor[1];
                    dy_ext0 = position_delta[1] - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = dy_ext0;
                }

                if (c & 0x04) != 0 {
                    zsv_ext0 = stretched_floor[2] + 1.0;
                    zsv_ext1 = stretched_floor[2] + 2.0;
                    dz_ext0 = position_delta[2] - 1.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] - 2.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                } else {
                    zsv_ext0 = stretched_floor[2];
                    zsv_ext1 = stretched_floor[2];
                    dz_ext0 = position_delta[2] - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = dz_ext0;
                }
            } else {
                // (1, 1, 1) is not one of the closest two tetrahedral vertices.
                let c = (a_point & b_point) as u8;

                if (c & 0x01) != 0 {
                    xsv_ext0 = stretched_floor[0] + 1.0;
                    xsv_ext1 = stretched_floor[0] + 2.0;
                    dx_ext0 = position_delta[0] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dx_ext1 = position_delta[0] - 2.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                } else {
                    xsv_ext0 = stretched_floor[0];
                    xsv_ext1 = stretched_floor[0];
                    dx_ext0 = position_delta[0] - Self::SQUISH_CONSTANT_3D;
                    dx_ext1 = position_delta[0] - 2.0 * Self::SQUISH_CONSTANT_3D;
                }

                if (c & 0x02) != 0 {
                    ysv_ext0 = stretched_floor[1] + 1.0;
                    ysv_ext1 = stretched_floor[1] + 2.0;
                    dy_ext0 = position_delta[1] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] - 2.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                } else {
                    ysv_ext0 = stretched_floor[1];
                    ysv_ext1 = stretched_floor[1];
                    dy_ext0 = position_delta[1] - Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] - 2.0 * Self::SQUISH_CONSTANT_3D;
                }

                if (c & 0x04) != 0 {
                    zsv_ext0 = stretched_floor[2] + 1.0;
                    zsv_ext1 = stretched_floor[2] + 2.0;
                    dz_ext0 = position_delta[2] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] - 2.0 - Self::SQUISH_CONSTANT_3D;
                } else {
                    zsv_ext0 = stretched_floor[2];
                    zsv_ext1 = stretched_floor[2];
                    dz_ext0 = position_delta[2] - Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] - 2.0 * Self::SQUISH_CONSTANT_3D;
                }
            }

            let t0 = 2.0 * Self::SQUISH_CONSTANT_3D;
            let t1 = 1.0 + 2.0 * Self::SQUISH_CONSTANT_3D;
            let t2 = t1 + Self::SQUISH_CONSTANT_3D;

            // Contribution at (1, 1, 0)
            vertex = math::add3(stretched_floor, [1.0, 1.0, 0.0]);
            dpos = math::sub3(position_delta, [t1, t1, t0]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 0, 1)
            vertex = math::add3(stretched_floor, [1.0, 0.0, 1.0]);
            dpos = math::sub3(position_delta, [t1, t0, t1]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 1)
            vertex = math::add3(stretched_floor, [0.0, 1.0, 1.0]);
            dpos = math::sub3(position_delta, [t0, t1, t1]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 1, 1)
            vertex = math::add3(stretched_floor, [1.0, 1.0, 1.0]);
            dpos = math::sub3(position_delta, [t2, t2, t2]);
            value += gradient(&self.perm_table, vertex, dpos);
        } else {
            // We're inside the octahedron (Rectified 3-Simplex) inbetween.
            let a_score;
            let b_score;
            let mut a_point: u8;
            let mut b_point: u8;
            let mut a_is_further_side: bool;
            let mut b_is_further_side: bool;

            // Decide between points (0, 0, 1) and (1, 1, 0) as closest.
            let p1 = relative_coords[0] + relative_coords[1];
            if p1 > 1.0 {
                a_score = p1 - 1.0;
                a_point = 0x03;
                a_is_further_side = true;
            } else {
                a_score = 1.0 - p1;
                a_point = 0x04;
                a_is_further_side = false;
            }

            // Decide between points (0, 1, 0) and (1, 0, 1) as closest.
            let p2 = relative_coords[0] + relative_coords[2];
            if p2 > 1.0 {
                b_score = p2 - 1.0;
                b_point = 0x05;
                b_is_further_side = true;
            } else {
                b_score = 1.0 - p2;
                b_point = 0x02;
                b_is_further_side = false;
            }

            // The closest out of the two (1, 0, 0) and (0, 1, 1) will replace the furthest out of the two decided above, if closer.
            let p3 = relative_coords[1] + relative_coords[2];
            if p3 < 1.0 {
                let score = p3 - 1.0;
                if a_score <= b_score && a_score < score {
                    a_point = 0x06;
                    a_is_further_side = true;
                } else if a_score > b_score && b_score < score {
                    b_point = 0x06;
                    b_is_further_side = true;
                }
            } else {
                let score = 1.0 - p3;
                if a_score <= b_score && a_score < score {
                    a_point = 0x01;
                    a_is_further_side = false;
                } else if a_score > b_score && b_score < score {
                    b_point = 0x01;
                    b_is_further_side = false;
                }
            }

            // Where each of the two closest points are determines how the extra two vertices are calculated.
            if a_is_further_side == b_is_further_side {
                // Both closest points are on (1, 1, 1) side
                if a_is_further_side {
                    // One of the two extra points is (1, 1, 1)
                    dx_ext0 = position_delta[0] - 1.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dy_ext0 = position_delta[1] - 1.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                    dz_ext0 = position_delta[2] - 1.0 - 3.0 * Self::SQUISH_CONSTANT_3D;
                    xsv_ext0 = stretched_floor[0] + 1.0;
                    ysv_ext0 = stretched_floor[1] + 1.0;
                    zsv_ext0 = stretched_floor[2] + 1.0;

                    // Other extra point is based on the shared axis.
                    let c = (a_point & b_point) as u8;

                    if (c & 0x01) != 0 {
                        dx_ext1 = position_delta[0] - 2.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                        dy_ext1 = position_delta[1] - 2.0 * Self::SQUISH_CONSTANT_3D;
                        dz_ext1 = position_delta[2] - 2.0 * Self::SQUISH_CONSTANT_3D;
                        xsv_ext1 = stretched_floor[0] + 2.0;
                        ysv_ext1 = stretched_floor[1];
                        zsv_ext1 = stretched_floor[2];
                    } else if (c & 0x02) != 0 {
                        dx_ext1 = position_delta[0] - 2.0 * Self::SQUISH_CONSTANT_3D;
                        dy_ext1 = position_delta[1] - 2.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                        dz_ext1 = position_delta[2] - 2.0 * Self::SQUISH_CONSTANT_3D;
                        xsv_ext1 = stretched_floor[0];
                        ysv_ext1 = stretched_floor[1] + 2.0;
                        zsv_ext1 = stretched_floor[2];
                    } else {
                        dx_ext1 = position_delta[0] - 2.0 * Self::SQUISH_CONSTANT_3D;
                        dy_ext1 = position_delta[1] - 2.0 * Self::SQUISH_CONSTANT_3D;
                        dz_ext1 = position_delta[2] - 2.0 - 2.0 * Self::SQUISH_CONSTANT_3D;
                        xsv_ext1 = stretched_floor[0];
                        ysv_ext1 = stretched_floor[1];
                        zsv_ext1 = stretched_floor[2] + 2.0;
                    }
                } else {
                    // Both closest points are on the (0, 0, 0) side
                    // One of the two extra points is (0, 0, 0)
                    dx_ext0 = position_delta[0];
                    dy_ext0 = position_delta[1];
                    dz_ext0 = position_delta[2];
                    xsv_ext0 = stretched_floor[0];
                    ysv_ext0 = stretched_floor[1];
                    zsv_ext0 = stretched_floor[2];

                    let c = (a_point | b_point) as u8;

                    if (c & 0x01) != 0 {
                        dx_ext1 = position_delta[0] + 1.0 - Self::SQUISH_CONSTANT_3D;
                        dy_ext1 = position_delta[1] - 1.0 - Self::SQUISH_CONSTANT_3D;
                        dz_ext1 = position_delta[2] - 1.0 - Self::SQUISH_CONSTANT_3D;
                        xsv_ext1 = stretched_floor[0] - 1.0;
                        ysv_ext1 = stretched_floor[1] + 1.0;
                        zsv_ext1 = stretched_floor[2] + 1.0;
                    } else if (c & 0x02) != 0 {
                        dx_ext1 = position_delta[0] - 1.0 - Self::SQUISH_CONSTANT_3D;
                        dy_ext1 = position_delta[1] + 1.0 - Self::SQUISH_CONSTANT_3D;
                        dz_ext1 = position_delta[2] - 1.0 - Self::SQUISH_CONSTANT_3D;
                        xsv_ext1 = stretched_floor[0] + 1.0;
                        ysv_ext1 = stretched_floor[1] - 1.0;
                        zsv_ext1 = stretched_floor[2] + 1.0;
                    } else {
                        dx_ext1 = position_delta[0] - 1.0 - Self::SQUISH_CONSTANT_3D;
                        dy_ext1 = position_delta[1] - 1.0 - Self::SQUISH_CONSTANT_3D;
                        dz_ext1 = position_delta[2] + 1.0 - Self::SQUISH_CONSTANT_3D;
                        xsv_ext1 = stretched_floor[0] + 1.0;
                        ysv_ext1 = stretched_floor[1] + 1.0;
                        zsv_ext1 = stretched_floor[2] - 1.0;
                    }
                }
            } else {
                // One point on (0, 0, 0) side, one point on (1, 1, 1) side
                let c1;
                let c2;
                if a_is_further_side {
                    c1 = a_point;
                    c2 = b_point;
                } else {
                    c1 = b_point;
                    c2 = a_point;
                }

                // One contribution is a permutation of (1, 1, -1)
                if (c1 & 0x01) != 0 {
                    dx_ext1 = position_delta[0] + 1.0 - Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    xsv_ext1 = stretched_floor[0] - 1.0;
                    ysv_ext1 = stretched_floor[1] + 1.0;
                    zsv_ext1 = stretched_floor[2] + 1.0;
                } else if (c1 & 0x02) != 0 {
                    dx_ext1 = position_delta[0] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] + 1.0 - Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    xsv_ext1 = stretched_floor[0] + 1.0;
                    ysv_ext1 = stretched_floor[1] - 1.0;
                    zsv_ext1 = stretched_floor[2] + 1.0;
                } else {
                    dx_ext1 = position_delta[0] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dy_ext1 = position_delta[1] - 1.0 - Self::SQUISH_CONSTANT_3D;
                    dz_ext1 = position_delta[2] + 1.0 - Self::SQUISH_CONSTANT_3D;
                    xsv_ext1 = stretched_floor[0] + 1.0;
                    ysv_ext1 = stretched_floor[1] + 1.0;
                    zsv_ext1 = stretched_floor[2] - 1.0;
                }

                // The other contribution is a permutation of (0, 0, 2)
                dx_ext1 = position_delta[0] - 2.0 * Self::SQUISH_CONSTANT_3D;
                dy_ext1 = position_delta[1] - 2.0 * Self::SQUISH_CONSTANT_3D;
                dz_ext1 = position_delta[2] - 2.0 * Self::SQUISH_CONSTANT_3D;
                xsv_ext1 = stretched_floor[0];
                ysv_ext1 = stretched_floor[1];
                zsv_ext1 = stretched_floor[2];

                if (c2 & 0x01) != 0 {
                    dx_ext1 -= 2.0;
                    xsv_ext1 += 2.0;
                } else if(c2 & 0x02) != 0 {
                    dy_ext1 -= 2.0;
                    ysv_ext1 += 2.0;
                } else {
                    dz_ext1 -= 2.0;
                    zsv_ext1 += 2.0;
                }
            }

            let t0 = Self::SQUISH_CONSTANT_3D;
            let t1 = 1.0 + Self::SQUISH_CONSTANT_3D;
            let t2 = 2.0 * Self::SQUISH_CONSTANT_3D;
            let t3 = 1.0 + 2.0 * Self::SQUISH_CONSTANT_3D;

            // Contribution at (1, 0, 0)
            vertex = math::add3(stretched_floor, [1.0, 0.0, 0.0]);
            dpos = math::sub3(position_delta, [t1, t0, t0]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 0)
            vertex = math::add3(stretched_floor, [0.0, 1.0, 0.0]);
            dpos = math::sub3(position_delta, [t0, t1, t0]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 0, 1)
            vertex = math::add3(stretched_floor, [0.0, 0.0, 1.0]);
            dpos = math::sub3(position_delta, [t0, t0, t1]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 1, 0)
            vertex = math::add3(stretched_floor, [1.0, 1.0, 0.0]);
            dpos = math::sub3(position_delta, [t3, t3, t2]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (1, 0, 1)
            vertex = math::add3(stretched_floor, [1.0, 0.0, 1.0]);
            dpos = math::sub3(position_delta, [t3, t2, t3]);
            value += gradient(&self.perm_table, vertex, dpos);

            // Contribution at (0, 1, 1)
            vertex = math::add3(stretched_floor, [0.0, 1.0, 1.0]);
            dpos = math::sub3(position_delta, [t2, t3, t3]);
            value += gradient(&self.perm_table, vertex, dpos);
        }

        // First extra vertex
        value += gradient(&self.perm_table, [xsv_ext0, ysv_ext0, zsv_ext0], [dz_ext0, dy_ext0, dz_ext0]);

        // Second extra vertex
        value += gradient(&self.perm_table, [xsv_ext1, ysv_ext1, zsv_ext1], [dz_ext1, dy_ext1, dz_ext1]);

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
