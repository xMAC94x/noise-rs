use crate::{
    noisefield::{NoiseField2D, NoiseField3D},
    NoiseFieldFn, NoiseFn,
};

/// Noise function that raises the output value from the first source function
/// to the power of the output value of the second source function.
pub struct Power<'a, T> {
    /// Outputs a value.
    pub source1: &'a dyn NoiseFieldFn<T>,

    /// Outputs a value.
    pub source2: &'a dyn NoiseFieldFn<T>,
}

impl<'a, T> Power<'a, T> {
    pub fn new(source1: &'a dyn NoiseFieldFn<T>, source2: &'a dyn NoiseFieldFn<T>) -> Self {
        Self { source1, source2 }
    }
}

// impl<'a, T> NoiseFn<T> for Power<'a, T>
// where
//     T: Copy,
// {
//     fn get(&self, point: T) -> f64 {
//         (self.source1.get(point)).powf(self.source2.get(point))
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Power<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = self.source1.process_field(field);
        let field2 = self.source2.process_field(field);

        out.values = out
            .values()
            .iter()
            .zip(field2.values().iter())
            .map(|(value1, value2)| value1.powf(*value2))
            .collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Power<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = self.source1.process_field(field);
        let field2 = self.source2.process_field(field);

        out.values = out
            .values()
            .iter()
            .zip(field2.values().iter())
            .map(|(value1, value2)| value1.powf(*value2))
            .collect();

        out
    }
}
