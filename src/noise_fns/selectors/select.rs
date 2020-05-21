use crate::{
    math::interpolate,
    noisefield::{NoiseField2D, NoiseField3D},
    NoiseFieldFn,
};

/// Noise function that outputs the value selected from one of two source
/// functions chosen by the output value from a control function.
pub struct Select<'a, T> {
    /// Outputs a value.
    pub source1: &'a dyn NoiseFieldFn<T>,

    /// Outputs a value.
    pub source2: &'a dyn NoiseFieldFn<T>,

    /// Determines the value to select. If the output value from
    /// the control function is within a range of values know as the _selection
    /// range_, this noise function outputs the value from `source2`.
    /// Otherwise, this noise function outputs the value from `source1`.
    pub control: &'a dyn NoiseFieldFn<T>,

    /// Bounds of the selection range. Default is 0.0 to 1.0.
    pub bounds: (f64, f64),

    /// Edge falloff value. Default is 0.0.
    pub falloff: f64,
}

impl<'a, T> Select<'a, T> {
    pub fn new(
        source1: &'a dyn NoiseFieldFn<T>,
        source2: &'a dyn NoiseFieldFn<T>,
        control: &'a dyn NoiseFieldFn<T>,
    ) -> Self {
        Select {
            source1,
            source2,
            control,
            bounds: (0.0, 1.0),
            falloff: 0.0,
        }
    }

    pub fn set_bounds(self, lower_bound: f64, upper_bound: f64) -> Self {
        Select {
            bounds: (lower_bound, upper_bound),
            ..self
        }
    }

    pub fn set_falloff(self, falloff: f64) -> Self {
        Select { falloff, ..self }
    }
}

// impl<'a, T> NoiseFn<T> for Select<'a, T>
// where
//     T: Copy,
// {
//     fn get(&self, point: T) -> f64 {
//         let control_value = self.control.get(point);
//         let (lower, upper) = self.bounds;
//
//         if self.falloff > 0.0 {
//             match () {
//                 _ if control_value < (lower - self.falloff) => self.source1.get(point),
//                 _ if control_value < (lower + self.falloff) => {
//                     let lower_curve = lower - self.falloff;
//                     let upper_curve = lower + self.falloff;
//                     let alpha = interpolate::s_curve3(
//                         (control_value - lower_curve) / (upper_curve - lower_curve),
//                     );
//
//                     interpolate::linear(self.source1.get(point), self.source2.get(point), alpha)
//                 }
//                 _ if control_value < (upper - self.falloff) => self.source2.get(point),
//                 _ if control_value < (upper + self.falloff) => {
//                     let lower_curve = upper - self.falloff;
//                     let upper_curve = upper + self.falloff;
//                     let alpha = interpolate::s_curve3(
//                         (control_value - lower_curve) / (upper_curve - lower_curve),
//                     );
//
//                     interpolate::linear(self.source2.get(point), self.source1.get(point), alpha)
//                 }
//                 _ => self.source1.get(point),
//             }
//         } else if control_value < lower || control_value > upper {
//             self.source1.get(point)
//         } else {
//             self.source2.get(point)
//         }
//     }
// }

impl<'a> NoiseFieldFn<NoiseField2D> for Select<'a, NoiseField2D> {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let control = self.control.process_field(field);
        let source1 = self.source1.process_field(field);
        let source2 = self.source2.process_field(field);
        let mut out = field.clone();

        let (lower_bound, upper_bound) = self.bounds;

        out.values = control
            .values()
            .iter()
            .zip(source1.values().iter().zip(source2.values().iter()))
            .map(|(control_value, (lower, upper))| {
                if self.falloff > 0.0 {
                    match () {
                        _ if *control_value < (lower_bound - self.falloff) => *lower,
                        _ if *control_value < (lower_bound + self.falloff) => {
                            let lower_curve = lower_bound - self.falloff;
                            let upper_curve = lower_bound + self.falloff;
                            let alpha = interpolate::s_curve3(
                                (control_value - lower_curve) / (upper_curve - lower_curve),
                            );

                            interpolate::linear(*lower, *upper, alpha)
                        }
                        _ if *control_value < (upper_bound - self.falloff) => *upper,
                        _ if *control_value < (upper_bound + self.falloff) => {
                            let lower_curve = upper_bound - self.falloff;
                            let upper_curve = upper_bound + self.falloff;
                            let alpha = interpolate::s_curve3(
                                (control_value - lower_curve) / (upper_curve - lower_curve),
                            );

                            interpolate::linear(*upper, *lower, alpha)
                        }
                        _ => *lower,
                    }
                } else if *control_value > lower_bound && *control_value < upper_bound {
                    *upper
                } else {
                    *lower
                }
            })
            .collect();

        out
    }
}

impl<'a> NoiseFieldFn<NoiseField3D> for Select<'a, NoiseField3D> {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let control = self.control.process_field(field);
        let source1 = self.source1.process_field(field);
        let source2 = self.source2.process_field(field);
        let mut out = field.clone();

        let (lower_bound, upper_bound) = self.bounds;

        out.values = control
            .values()
            .iter()
            .zip(source1.values().iter().zip(source2.values().iter()))
            .map(|(control_value, (lower, upper))| {
                if self.falloff > 0.0 {
                    match () {
                        _ if *control_value < (lower_bound - self.falloff) => *lower,
                        _ if *control_value < (lower_bound + self.falloff) => {
                            let lower_curve = lower_bound - self.falloff;
                            let upper_curve = lower_bound + self.falloff;
                            let alpha = interpolate::s_curve3(
                                (control_value - lower_curve) / (upper_curve - lower_curve),
                            );

                            interpolate::linear(*lower, *upper, alpha)
                        }
                        _ if *control_value < (upper_bound - self.falloff) => *upper,
                        _ if *control_value < (upper_bound + self.falloff) => {
                            let lower_curve = upper_bound - self.falloff;
                            let upper_curve = upper_bound + self.falloff;
                            let alpha = interpolate::s_curve3(
                                (control_value - lower_curve) / (upper_curve - lower_curve),
                            );

                            interpolate::linear(*upper, *lower, alpha)
                        }
                        _ => *lower,
                    }
                } else if *control_value > lower_bound && *control_value < upper_bound {
                    *upper
                } else {
                    *lower
                }
            })
            .collect();

        out
    }
}
