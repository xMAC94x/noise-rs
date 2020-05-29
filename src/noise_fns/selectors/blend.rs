use crate::noisefield::{NoiseField, NoiseField2D, NoiseField3D};
use crate::{math::interpolate, NoiseFieldFn};

/// Noise function that outputs a weighted blend of the output values from two
/// source functions given the output value supplied by a control function.
///
/// This noise function uses linear interpolation to perform the blending
/// operation.
pub struct Blend<'a, T> {
    /// Outputs one of the values to blend.
    pub source1: &'a dyn NoiseFieldFn<T>,

    /// Outputs one of the values to blend.
    pub source2: &'a dyn NoiseFieldFn<T>,

    /// Determines the weight of the blending operation. Negative values weight
    /// the blend towards the output value from the `source1` function. Positive
    /// values weight the blend towards the output value from the `source2`
    /// function.
    pub control: &'a dyn NoiseFieldFn<T>,
}

impl<'a, T> Blend<'a, T> {
    pub fn new(
        source1: &'a dyn NoiseFieldFn<T>,
        source2: &'a dyn NoiseFieldFn<T>,
        control: &'a dyn NoiseFieldFn<T>,
    ) -> Self {
        Blend {
            source1,
            source2,
            control,
        }
    }
}

// impl<'a, T> NoiseFn<T> for Blend<'a, T>
// where
//     T: Copy,
// {
//     fn get(&self, point: T) -> f64 {
//         let lower = self.source1.get(point);
//         let upper = self.source2.get(point);
//         let control = self.control.get(point);
//
//         interpolate::linear(lower, upper, control)
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Blend<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let control = self.control.process_field(field);
        let lower = self.source1.process_field(field);
        let upper = self.source2.process_field(field);
        let mut out = field.clone();

        out.values = control
            .values()
            .iter()
            .zip(lower.values.iter().zip(upper.values.iter()))
            .map(|(control, (lower, upper))| interpolate::linear(*lower, *upper, *control))
            .collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Blend<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let control = self.control.process_field(field);
        let lower = self.source1.process_field(field);
        let upper = self.source2.process_field(field);
        let mut out = field.clone();

        out.values = control
            .values()
            .iter()
            .zip(lower.values.iter().zip(upper.values.iter()))
            .map(|(control, (lower, upper))| interpolate::linear(*lower, *upper, *control))
            .collect();

        out
    }
}
