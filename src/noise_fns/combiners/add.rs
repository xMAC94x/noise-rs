use crate::noisefield::{NoiseField, NoiseField2D, NoiseField3D};
use crate::{NoiseFieldFn, NoiseFn};

/// Noise function that outputs the sum of the two output values from two source
/// functions.
pub struct Add<'a, T> {
    /// Outputs a value.
    pub source1: &'a dyn NoiseFieldFn<T>,

    /// Outputs a value.
    pub source2: &'a dyn NoiseFieldFn<T>,
}

impl<'a, T> Add<'a, T> {
    pub fn new(source1: &'a dyn NoiseFieldFn<T>, source2: &'a dyn NoiseFieldFn<T>) -> Self {
        Self { source1, source2 }
    }
}

// impl<'a, T> NoiseFn<T> for Add<'a, T>
// where
//     T: Copy,
// {
//     fn get(&self, point: T) -> f64 {
//         self.source1.get(point) + self.source2.get(point)
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Add<'a, NoiseField2D>
// where
//     T: NoiseField,
{
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = self.source1.process_field(field);
        let field2 = self.source2.process_field(field);

        out.values = out
            .values()
            .iter()
            .zip(field2.values().iter())
            .map(|(value1, value2)| value1 + value2)
            .collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Add<'a, NoiseField3D>
// where
//     T: NoiseField,
{
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = self.source1.process_field(field);
        let field2 = self.source2.process_field(field);

        out.values = out
            .values()
            .iter()
            .zip(field2.values().iter())
            .map(|(value1, value2)| value1 + value2)
            .collect();

        out
    }
}
