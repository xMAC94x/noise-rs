use crate::{
    noisefield::{NoiseField2D, NoiseField3D},
    NoiseFieldFn, NoiseFn,
};
use rayon::prelude::*;

/// Noise function that negates the output value from the source function.
pub struct Negate<'a, T> {
    /// Outputs a value.
    pub source: &'a dyn NoiseFieldFn<T>,
}

impl<'a, T> Negate<'a, T> {
    pub fn new(source: &'a dyn NoiseFieldFn<T>) -> Self {
        Negate { source }
    }
}

// impl<'a, T> NoiseFn<T> for Negate<'a, T> {
//     fn get(&self, point: T) -> f64 {
//         -self.source.get(point)
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Negate<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = self.source.process_field(field);

        out.values = out.values().par_iter().map(|value| -*value).collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Negate<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = self.source.process_field(field);

        out.values = out.values().par_iter().map(|value| -*value).collect();

        out
    }
}
