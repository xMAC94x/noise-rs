use crate::math::{self, scale_shift};
use crate::noise_fns::{MultiFractal, NoiseFn, Perlin, Seedable};
use crate::noisefield::{NoiseField2D, NoiseField3D};
use crate::NoiseFieldFn;
use rayon::prelude::*;

/// Noise function that outputs ridged-multifractal noise.
///
/// This noise function, heavily based on the fBm-noise function, generates
/// ridged-multifractal noise. Ridged-multifractal noise is generated in much
/// the same way as fBm noise, except the output of each octave is modified by
/// an absolute-value function. Modifying the octave values in this way
/// produces ridge-like formations.
///
/// The values output from this function will usually range from -1.0 to 1.0 with
/// default values for the parameters, but there are no guarantees that all
/// output values will exist within this range. If the parameters are modified
/// from their defaults, then the output will need to be scaled to remain in
/// the [-1,1] range.
///
/// Ridged-multifractal noise is often used to generate craggy mountainous
/// terrain or marble-like textures.
#[derive(Clone, Debug)]
pub struct RidgedMulti {
    /// Total number of frequency octaves to generate the noise with.
    ///
    /// The number of octaves control the _amount of detail_ in the noise
    /// function. Adding more octaves increases the detail, with the drawback
    /// of increasing the calculation time.
    pub octaves: usize,

    /// The number of cycles per unit length that the noise function outputs.
    pub frequency: f64,

    /// A multiplier that determines how quickly the frequency increases for
    /// each successive octave in the noise function.
    ///
    /// The frequency of each successive octave is equal to the product of the
    /// previous octave's frequency and the lacunarity value.
    ///
    /// A lacunarity of 2.0 results in the frequency doubling every octave. For
    /// almost all cases, 2.0 is a good value to use.
    pub lacunarity: f64,

    /// A multiplier that determines how quickly the amplitudes diminish for
    /// each successive octave in the noise function.
    ///
    /// The amplitude of each successive octave is equal to the product of the
    /// previous octave's amplitude and the persistence value. Increasing the
    /// persistence produces "rougher" noise.
    // pub persistence: f64,

    /// The attenuation to apply to the weight on each octave. This reduces
    /// the strength of each successive octave, making their respective
    /// ridges smaller. The default attenuation is 2.0, making each octave
    /// half the height of the previous.
    pub gain: f64,

    seed: u32,
    sources: Vec<Perlin>,

    spectral_weights: Vec<f64>,
}

impl RidgedMulti {
    pub const DEFAULT_SEED: u32 = 0;
    pub const DEFAULT_OCTAVES: usize = 6;
    pub const DEFAULT_FREQUENCY: f64 = 1.0;
    pub const DEFAULT_LACUNARITY: f64 = 2.17;
    pub const DEFAULT_GAIN: f64 = 2.0;
    pub const MAX_OCTAVES: usize = 32;

    pub fn new() -> Self {
        let mut out = Self {
            seed: Self::DEFAULT_SEED,
            octaves: Self::DEFAULT_OCTAVES,
            frequency: Self::DEFAULT_FREQUENCY,
            lacunarity: Self::DEFAULT_LACUNARITY,
            gain: Self::DEFAULT_GAIN,
            sources: super::build_sources(Self::DEFAULT_SEED, Self::DEFAULT_OCTAVES),
            spectral_weights: vec![0.0; Self::DEFAULT_OCTAVES],
        };

        out.calc_spectral_weights();

        out
    }

    pub fn set_gain(self, gain: f64) -> Self {
        Self { gain, ..self }
    }

    fn calc_spectral_weights(&mut self) {
        let h: f64 = 1.0;
        let mut frequency = self.frequency;

        for i in 0..self.octaves {
            self.spectral_weights[i] = frequency.powf(-h);
            frequency *= self.lacunarity;
        }
    }
}

impl Default for RidgedMulti {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiFractal for RidgedMulti {
    fn set_octaves(self, mut octaves: usize) -> Self {
        if self.octaves == octaves {
            return self;
        }

        octaves = math::clamp(octaves, 1, Self::MAX_OCTAVES);
        let mut out = Self {
            octaves,
            sources: super::build_sources(self.seed, octaves),
            spectral_weights: vec![0.0; octaves],
            ..self
        };

        out.calc_spectral_weights();

        out
    }

    fn set_frequency(self, frequency: f64) -> Self {
        let mut out = Self { frequency, ..self };

        out.calc_spectral_weights();

        out
    }

    fn set_lacunarity(self, lacunarity: f64) -> Self {
        let mut out = Self { lacunarity, ..self };
        out.calc_spectral_weights();
        out
    }

    fn set_persistence(self, _persistence: f64) -> Self {
        unimplemented!()
    }
}

impl Seedable for RidgedMulti {
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        Self {
            seed,
            sources: super::build_sources(seed, self.octaves),
            ..self
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

/// 2-dimensional `RidgedMulti` noise
impl NoiseFn<[f64; 2]> for RidgedMulti {
    fn get(&self, mut point: [f64; 2]) -> f64 {
        let mut result = 0.0;
        let mut weight = 1.0;

        point = math::mul2(point, self.frequency);

        for x in 0..self.octaves {
            // Get the value.
            let mut signal = self.sources[x].get(point);

            // Make the ridges.
            signal = signal.abs();
            signal = 1.0 - signal;

            // Square the signal to increase the sharpness of the ridges.
            signal *= signal;

            // Apply the weighting from the previous octave to the signal.
            // Larger values have higher weights, producing sharp points along
            // the ridges.
            signal *= weight;

            // Weight successive contributions by the previous signal.
            weight = signal * self.gain;

            // Clamp the weight to [0,1] to prevent the result from diverging.
            weight = math::clamp(weight, 0.0, 1.0);

            // Scale the amplitude appropriately for this frequency.
            // signal *= self.persistence.powi(x as i32);

            // Add the signal to the result.
            result += signal;

            // Increase the frequency.
            point = math::mul2(point, self.lacunarity);
        }

        // Scale and shift the result into the [-1,1] range
        let scale = 2.0 - 0.5_f64.powi(self.octaves as i32 - 1);
        scale_shift(result, 2.0 / scale)
    }
}

/// 3-dimensional `RidgedMulti` noise
impl NoiseFn<[f64; 3]> for RidgedMulti {
    fn get(&self, mut point: [f64; 3]) -> f64 {
        let mut result = 0.0;
        let mut weight = 1.0;

        point = math::mul3(point, self.frequency);

        for x in 0..self.octaves {
            // Get the value.
            let mut signal = self.sources[x].get(point);

            // Make the ridges.
            signal = signal.abs();
            signal = 1.0 - signal;

            // Square the signal to increase the sharpness of the ridges.
            signal *= signal;

            // Apply the weighting from the previous octave to the signal.
            // Larger values have higher weights, producing sharp points along
            // the ridges.
            signal *= weight;

            // Weight successive contributions by the previous signal.
            weight = signal * self.gain;

            // Clamp the weight to [0,1] to prevent the result from diverging.
            weight = math::clamp(weight, 0.0, 1.0);

            // Scale the amplitude appropriately for this frequency.
            // signal *= self.persistence.powi(x as i32);

            // Add the signal to the result.
            result += signal;

            // Increase the frequency.
            point = math::mul3(point, self.lacunarity);
        }

        // Scale and shift the result into the [-1,1] range
        let scale = 2.0 - 0.5_f64.powi(self.octaves as i32 - 1);
        scale_shift(result, 2.0 / scale)
    }
}

/// 4-dimensional `RidgedMulti` noise
impl NoiseFn<[f64; 4]> for RidgedMulti {
    fn get(&self, mut point: [f64; 4]) -> f64 {
        let mut result = 0.0;
        let mut weight = 1.0;

        point = math::mul4(point, self.frequency);

        for x in 0..self.octaves {
            // Get the value.
            let mut signal = self.sources[x].get(point);

            // Make the ridges.
            signal = signal.abs();
            signal = 1.0 - signal;

            // Square the signal to increase the sharpness of the ridges.
            signal *= signal;

            // Apply the weighting from the previous octave to the signal.
            // Larger values have higher weights, producing sharp points along
            // the ridges.
            signal *= weight;

            // Weight successive contributions by the previous signal.
            weight = signal * self.gain;

            // Clamp the weight to [0,1] to prevent the result from diverging.
            weight = math::clamp(weight, 0.0, 1.0);

            // Scale the amplitude appropriately for this frequency.
            // signal *= self.persistence.powi(x as i32);

            // Add the signal to the result.
            result += signal;

            // Increase the frequency.
            point = math::mul4(point, self.lacunarity);
        }

        // Scale and shift the result into the [-1,1] range
        let scale = 2.0 - 0.5_f64.powi(self.octaves as i32 - 1);
        scale_shift(result, 2.0 / scale)
    }
}

impl NoiseFieldFn<NoiseField2D> for RidgedMulti {
    fn process_field(&self, field: &NoiseField2D) -> NoiseField2D {
        let mut out = field.clone();

        let fields: Vec<NoiseField2D> = self
            .sources
            .iter()
            .enumerate()
            .map(|(index, source)| {
                source.process_field(&out.scale_coordinates(self.lacunarity.powi(index as i32)))
            })
            .collect();

        // let spectral_weights = calc_spectral_weights(self.octaves, self.lacunarity);

        out.values = field
            .coordinates()
            .par_iter()
            .enumerate()
            .map(|(index, point)| {
                let offset = 1.0;

                let mut point = math::mul2(*point, self.frequency);

                // Do first octave
                let mut signal = fields[0].value_at_index(index);

                // Invert and translate (note that offset should be ~1.0)
                signal = signal.abs();
                signal = offset - signal;

                // Square the signal, to increase the sharpness of ridges
                signal *= signal;

                // Assign the initial values
                let mut result = signal;
                let mut weight;

                for current_octave in 1..self.octaves {
                    // Increase the frequency.
                    point = math::mul2(point, self.lacunarity);

                    // Weight successive contributions by the previous signal.
                    // weight = signal / self.attenuation;
                    weight = signal * self.gain;

                    // Clamp the weight to [0,1] to prevent the result from diverging.
                    weight = math::clamp(weight, 0.0, 1.0);

                    // Get the value.
                    let mut signal = fields[current_octave].value_at_index(index);

                    // Make the ridges.
                    signal = signal.abs();
                    signal = 1.0 - signal;

                    // Square the signal to increase the sharpness of the ridges.
                    signal *= signal;

                    // Apply the weighting from the previous octave to the signal.
                    // Larger values have higher weights, producing sharp points along
                    // the ridges.
                    signal *= weight;

                    // Add the signal to the output value.
                    result += signal * self.spectral_weights[current_octave];
                }

                // Scale and shift the result into the [-1,1] range
                let scale = 2.0 - 0.5_f64.powi(self.octaves as i32 - 1);
                scale_shift(result, 2.0 / scale)
            })
            .collect();

        out
    }
}

impl NoiseFieldFn<NoiseField3D> for RidgedMulti {
    fn process_field(&self, field: &NoiseField3D) -> NoiseField3D {
        let mut out = field.clone();

        let fields: Vec<NoiseField3D> = self
            .sources
            .iter()
            .enumerate()
            .map(|(index, source)| {
                source.process_field(&out.scale_coordinates(self.lacunarity.powi(index as i32)))
            })
            .collect();

        out.values = field
            .coordinates()
            .par_iter()
            .enumerate()
            .map(|(index, point)| {
                let offset = 1.0;

                let mut point = math::mul3(*point, self.frequency);

                // Do first octave
                let mut signal = fields[0].value_at_index(index);

                // Invert and translate (note that offset should be ~1.0)
                signal = signal.abs();
                signal = offset - signal;

                // Square the signal, to increase the sharpness of ridges
                signal *= signal;

                // Assign the initial values
                let mut result = signal;
                let mut weight;

                for current_octave in 1..self.octaves {
                    // Increase the frequency.
                    point = math::mul3(point, self.lacunarity);

                    // Weight successive contributions by the previous signal.
                    // weight = signal / self.attenuation;
                    weight = signal * self.gain;

                    // Clamp the weight to [0,1] to prevent the result from diverging.
                    weight = math::clamp(weight, 0.0, 1.0);

                    // Get the value.
                    let mut signal = fields[current_octave].value_at_index(index);

                    // Make the ridges.
                    signal = signal.abs();
                    signal = 1.0 - signal;

                    // Square the signal to increase the sharpness of the ridges.
                    signal *= signal;

                    // Apply the weighting from the previous octave to the signal.
                    // Larger values have higher weights, producing sharp points along
                    // the ridges.
                    signal *= weight;

                    // Add the signal to the output value.
                    result += signal * self.spectral_weights[current_octave];
                }

                // Scale and shift the result into the [-1,1] range
                let scale = 2.0 - 0.5_f64.powi(self.octaves as i32 - 1);
                scale_shift(result, 2.0 / scale)
            })
            .collect();

        out
    }
}
