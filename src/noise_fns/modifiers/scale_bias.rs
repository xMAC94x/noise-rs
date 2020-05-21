use crate::{
    noisefield::{NoiseField2D, NoiseField3D},
    NoiseFieldFn, NoiseFn,
};

/// Noise function that applies a scaling factor and a bias to the output value
/// from the source function.
///
/// The function retrieves the output value from the source function, multiplies
/// it with the scaling factor, adds the bias to it, then outputs the value.
pub struct ScaleBias<'a, T> {
    /// Outputs a value.
    pub source: &'a dyn NoiseFieldFn<T>,

    /// Scaling factor to apply to the output value from the source function.
    /// The default value is 1.0.
    pub scale: f64,

    /// Bias to apply to the scaled output value from the source function.
    /// The default value is 0.0.
    pub bias: f64,
}

impl<'a, T> ScaleBias<'a, T> {
    pub fn new(source: &'a dyn NoiseFieldFn<T>) -> Self {
        Self {
            source,
            scale: 1.0,
            bias: 0.0,
        }
    }

    pub fn set_scale(self, scale: f64) -> Self {
        Self { scale, ..self }
    }

    pub fn set_bias(self, bias: f64) -> Self {
        Self { bias, ..self }
    }
}

// impl<'a, T> NoiseFn<T> for ScaleBias<'a, T> {
//     #[cfg(not(target_os = "emscripten"))]
//     fn get(&self, point: T) -> f64 {
//         (self.source.get(point)).mul_add(self.scale, self.bias)
//     }
//
//     #[cfg(target_os = "emscripten")]
//     fn get(&self, point: T) -> f64 {
//         (self.source.get(point) * self.scale) + self.bias
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for ScaleBias<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = self.source.process_field(field);

        out.values = out
            .values()
            .iter()
            .map(|value| value.mul_add(self.scale, self.bias))
            .collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for ScaleBias<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = self.source.process_field(field);

        out.values = out
            .values()
            .iter()
            .map(|value| value.mul_add(self.scale, self.bias))
            .collect();

        out
    }
}
