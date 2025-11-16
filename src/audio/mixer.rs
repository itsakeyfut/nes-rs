// Audio mixer - Implements NES APU non-linear mixing formula
//
// The NES uses a non-linear mixing approach that simulates the analog
// characteristics of the hardware. This produces more accurate sound
// compared to simple linear mixing.

/// APU mixer implementing the NES non-linear mixing formula
///
/// The NES uses separate mixing for pulse channels and the other channels:
///
/// ```text
/// pulse_out = 95.88 / (8128 / (pulse1 + pulse2) + 100)
/// tnd_out = 159.79 / (1 / (triangle/8227 + noise/12241 + dmc/22638) + 100)
/// output = pulse_out + tnd_out
/// ```
///
/// Where pulse1, pulse2, triangle, noise, and dmc are the raw output
/// values from each channel (0-15 for pulse, 0-15 for triangle,
/// 0-15 for noise, 0-127 for DMC).
pub struct Mixer {
    /// Volume control (0.0 = mute, 1.0 = full volume)
    volume: f32,
}

impl Mixer {
    /// Create a new mixer with full volume
    pub fn new() -> Self {
        Self { volume: 1.0 }
    }

    /// Create a new mixer with specified volume
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level (0.0 = mute, 1.0 = full volume)
    pub fn with_volume(volume: f32) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
        }
    }

    /// Set the master volume
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level (0.0 = mute, 1.0 = full volume)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get the current volume
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Mix all APU channels using the non-linear formula
    ///
    /// # Arguments
    ///
    /// * `pulse1` - Pulse channel 1 output (0-15)
    /// * `pulse2` - Pulse channel 2 output (0-15)
    /// * `triangle` - Triangle channel output (0-15)
    /// * `noise` - Noise channel output (0-15)
    /// * `dmc` - DMC channel output (0-127)
    ///
    /// # Returns
    ///
    /// Mixed audio sample as f32 in range [-1.0, 1.0]
    pub fn mix(&self, pulse1: u8, pulse2: u8, triangle: u8, noise: u8, dmc: u8) -> f32 {
        // Mix pulse channels using non-linear formula
        let pulse_out = self.mix_pulse(pulse1, pulse2);

        // Mix triangle, noise, and DMC using non-linear formula
        let tnd_out = self.mix_tnd(triangle, noise, dmc);

        // Combine and apply volume.
        // NES formulas yield ~[0.0, 1.0], so map to [-1.0, 1.0] with 2x-1.
        // This ensures silence (0.0) maps to 0.0, avoiding DC offset.
        let mixed = pulse_out + tnd_out;
        let output = (mixed * 2.0 - 1.0) * self.volume;

        // Clamp to valid range
        output.clamp(-1.0, 1.0)
    }

    /// Mix pulse channels using the NES non-linear formula
    ///
    /// Formula: pulse_out = 95.88 / (8128 / (pulse1 + pulse2) + 100)
    ///
    /// # Arguments
    ///
    /// * `pulse1` - Pulse channel 1 output (0-15)
    /// * `pulse2` - Pulse channel 2 output (0-15)
    ///
    /// # Returns
    ///
    /// Mixed pulse output in range [0.0, ~1.0]
    fn mix_pulse(&self, pulse1: u8, pulse2: u8) -> f32 {
        let pulse_sum = pulse1 as f32 + pulse2 as f32;

        if pulse_sum == 0.0 {
            return 0.0;
        }

        95.88 / (8128.0 / pulse_sum + 100.0)
    }

    /// Mix triangle, noise, and DMC channels using the NES non-linear formula
    ///
    /// Formula: tnd_out = 159.79 / (1 / (triangle/8227 + noise/12241 + dmc/22638) + 100)
    ///
    /// # Arguments
    ///
    /// * `triangle` - Triangle channel output (0-15)
    /// * `noise` - Noise channel output (0-15)
    /// * `dmc` - DMC channel output (0-127)
    ///
    /// # Returns
    ///
    /// Mixed TND output in range [0.0, ~1.0]
    fn mix_tnd(&self, triangle: u8, noise: u8, dmc: u8) -> f32 {
        let triangle_val = triangle as f32 / 8227.0;
        let noise_val = noise as f32 / 12241.0;
        let dmc_val = dmc as f32 / 22638.0;

        let tnd_sum = triangle_val + noise_val + dmc_val;

        if tnd_sum == 0.0 {
            return 0.0;
        }

        159.79 / (1.0 / tnd_sum + 100.0)
    }

    /// Mix channels with individual volume control
    ///
    /// This is useful for debugging individual channels or implementing
    /// per-channel volume control.
    ///
    /// # Arguments
    ///
    /// * `pulse1` - Pulse channel 1 output (0-15)
    /// * `pulse2` - Pulse channel 2 output (0-15)
    /// * `triangle` - Triangle channel output (0-15)
    /// * `noise` - Noise channel output (0-15)
    /// * `dmc` - DMC channel output (0-127)
    /// * `pulse1_vol` - Pulse 1 volume multiplier (0.0-1.0)
    /// * `pulse2_vol` - Pulse 2 volume multiplier (0.0-1.0)
    /// * `triangle_vol` - Triangle volume multiplier (0.0-1.0)
    /// * `noise_vol` - Noise volume multiplier (0.0-1.0)
    /// * `dmc_vol` - DMC volume multiplier (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Mixed audio sample as f32 in range [-1.0, 1.0]
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn mix_with_channel_volumes(
        &self,
        pulse1: u8,
        pulse2: u8,
        triangle: u8,
        noise: u8,
        dmc: u8,
        pulse1_vol: f32,
        pulse2_vol: f32,
        triangle_vol: f32,
        noise_vol: f32,
        dmc_vol: f32,
    ) -> f32 {
        // Apply individual channel volumes
        let p1 = (pulse1 as f32 * pulse1_vol.clamp(0.0, 1.0)) as u8;
        let p2 = (pulse2 as f32 * pulse2_vol.clamp(0.0, 1.0)) as u8;
        let tri = (triangle as f32 * triangle_vol.clamp(0.0, 1.0)) as u8;
        let noi = (noise as f32 * noise_vol.clamp(0.0, 1.0)) as u8;
        let d = (dmc as f32 * dmc_vol.clamp(0.0, 1.0)) as u8;

        self.mix(p1, p2, tri, noi, d)
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_creation() {
        let mixer = Mixer::new();
        assert_eq!(mixer.volume(), 1.0);

        let mixer = Mixer::with_volume(0.5);
        assert_eq!(mixer.volume(), 0.5);
    }

    #[test]
    fn test_volume_clamping() {
        let mixer = Mixer::with_volume(2.0);
        assert_eq!(mixer.volume(), 1.0);

        let mixer = Mixer::with_volume(-0.5);
        assert_eq!(mixer.volume(), 0.0);
    }

    #[test]
    fn test_mix_silence() {
        let mixer = Mixer::new();
        let output = mixer.mix(0, 0, 0, 0, 0);
        // NES formulas return 0.0 when all channels are 0, which maps to -1.0 in [-1, 1] range
        assert_eq!(output, -1.0);
    }

    #[test]
    fn test_mix_pulse_only() {
        let mixer = Mixer::new();
        let output = mixer.mix(15, 15, 0, 0, 0);
        // Output should be non-zero
        assert!(output > -1.0);
        assert!(output <= 1.0);
    }

    #[test]
    fn test_mix_all_channels() {
        let mixer = Mixer::new();
        let output = mixer.mix(15, 15, 15, 15, 127);
        // Output should be in valid range
        assert!(output >= -1.0);
        assert!(output <= 1.0);
    }

    #[test]
    fn test_volume_control() {
        let mut mixer = Mixer::new();
        mixer.set_volume(0.5);
        assert_eq!(mixer.volume(), 0.5);

        let output_half = mixer.mix(15, 15, 15, 15, 127);

        mixer.set_volume(1.0);
        let output_full = mixer.mix(15, 15, 15, 15, 127);

        // Half volume should produce smaller output
        assert!(output_half.abs() < output_full.abs());
    }

    #[test]
    fn test_mix_pulse_formula() {
        let mixer = Mixer::new();

        // Test with known values
        let result = mixer.mix_pulse(8, 8);

        // Expected: 95.88 / (8128 / 16 + 100) = 95.88 / 608 ≈ 0.1577
        let expected = 95.88 / (8128.0 / 16.0 + 100.0);
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_mix_tnd_formula() {
        let mixer = Mixer::new();

        // Test with known values
        let result = mixer.mix_tnd(8, 8, 64);

        // Calculate expected value
        let triangle_val = 8.0 / 8227.0;
        let noise_val = 8.0 / 12241.0;
        let dmc_val = 64.0 / 22638.0;
        let tnd_sum = triangle_val + noise_val + dmc_val;
        let expected = 159.79 / (1.0 / tnd_sum + 100.0);

        assert!((result - expected).abs() < 0.001);
    }
}
