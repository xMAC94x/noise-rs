use crate::{
    noisefield::{NoiseField, NoiseField2D, NoiseField3D},
    NoiseFieldFn, NoiseFn,
};
use rayon::prelude::*;

/// Noise function that outputs the absolute value of the output value from the
/// source function.
pub struct Abs<'a, T> {
    /// Outputs a value.
    pub source: &'a dyn NoiseFieldFn<T>,
}

impl<'a, T> Abs<'a, T> {
    pub fn new(source: &'a dyn NoiseFieldFn<T>) -> Self {
        Self { source }
    }
}

// impl<'a, T> NoiseFn<T> for Abs<'a, T> {
//     fn get(&self, point: T) -> f64 {
//         (self.source.get(point)).abs()
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Abs<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = self.source.process_field(field);

        out.values = out.values().par_iter().map(|value| value.abs()).collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Abs<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = self.source.process_field(field);

        out.values = out.values().par_iter().map(|value| value.abs()).collect();

        out
    }
}
