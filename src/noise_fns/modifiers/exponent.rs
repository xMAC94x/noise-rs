use crate::{
    math::scale_shift,
    noisefield::{NoiseField2D, NoiseField3D},
    NoiseFieldFn, NoiseFn,
};

/// Noise function that maps the output value from the source function onto an
/// exponential curve.
///
/// Because most noise functions will output values that range from -1.0 to 1.0,
/// this noise function first normalizes the output value (the range becomes 0.0
/// to 1.0), maps that value onto an exponential curve, then rescales that
/// value back to the original range.
pub struct Exponent<'a, T> {
    /// Outputs a value.
    pub source: &'a dyn NoiseFieldFn<T>,

    /// Exponent to apply to the output value from the source function. Default
    /// is 1.0.
    pub exponent: f64,
}

impl<'a, T> Exponent<'a, T> {
    pub fn new(source: &'a dyn NoiseFieldFn<T>) -> Self {
        Self {
            source,
            exponent: 1.0,
        }
    }

    pub fn set_exponent(self, exponent: f64) -> Self {
        Self { exponent, ..self }
    }
}

// impl<'a, T> NoiseFn<T> for Exponent<'a, T> {
//     fn get(&self, point: T) -> f64 {
//         let mut value = self.source.get(point);
//         value = (value + 1.0) / 2.0;
//         value = value.abs();
//         value = value.powf(self.exponent);
//         scale_shift(value, 2.0)
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Exponent<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = self.source.process_field(field);

        out.values = out
            .values()
            .iter()
            .map(|value| {
                let mut value = *value;
                value = (value + 1.0) / 2.0;
                value = value.abs();
                value = value.powf(self.exponent);
                scale_shift(value, 2.0)
            })
            .collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Exponent<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = self.source.process_field(field);

        out.values = out
            .values()
            .iter()
            .map(|value| {
                let mut value = *value;
                value = (value + 1.0) / 2.0;
                value = value.abs();
                value = value.powf(self.exponent);
                scale_shift(value, 2.0)
            })
            .collect();

        out
    }
}
