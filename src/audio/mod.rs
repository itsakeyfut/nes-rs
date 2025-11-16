// Audio module - NES APU audio output and mixing
//
// This module provides:
// - Non-linear APU mixing (accurate NES audio reproduction)
// - Sample rate conversion (NES ~1.79 MHz to 44.1/48 kHz)
// - Cross-platform audio output using cpal
// - Audio buffering and synchronization
//
// # Usage
//
// ```no_run
// use nes_rs::audio::{AudioSystem, AudioConfig};
// use nes_rs::apu::Apu;
//
// // Create audio system
// let audio_config = AudioConfig::new().with_sample_rate(48000);
// let mut audio_system = AudioSystem::new(audio_config).unwrap();
//
// // In emulator loop:
// let mut apu = Apu::new();
// // ... run APU ...
//
// // Get channel outputs
// let pulse1 = apu.pulse1_output();
// let pulse2 = apu.pulse2_output();
// let triangle = apu.triangle_output();
// let noise = apu.noise_output();
// let dmc = apu.dmc_output();
//
// // Process audio
// audio_system.process_apu_sample(pulse1, pulse2, triangle, noise, dmc);
// ```

pub mod mixer;
pub mod output;
pub mod resampler;

pub use mixer::Mixer;
pub use output::{AudioConfig, AudioOutput, AudioOutputBuilder};
pub use resampler::{sample_rates, AudioBuffer, Resampler};

use std::sync::{Arc, Mutex};

/// Complete audio system for NES emulation
///
/// Combines mixer, resampler, and output into a single easy-to-use interface.
pub struct AudioSystem {
    /// APU mixer
    mixer: Mixer,

    /// Sample rate resampler
    resampler: Arc<Mutex<Resampler>>,

    /// Audio output
    output: AudioOutput,

    /// Statistics
    samples_processed: u64,
    samples_output: u64,
}

impl AudioSystem {
    /// Create a new audio system
    ///
    /// # Arguments
    ///
    /// * `config` - Audio configuration
    ///
    /// # Returns
    ///
    /// Result containing the AudioSystem or an error message
    pub fn new(config: AudioConfig) -> Result<Self, String> {
        let mixer = Mixer::new();

        let resampler = if config.sample_rate == 44100 {
            Resampler::new_44_1_khz()
        } else if config.sample_rate == 48000 {
            Resampler::new_48_khz()
        } else {
            Resampler::new(sample_rates::NES_CPU_CLOCK, config.sample_rate as f64)
        };

        let output = AudioOutput::new(config)?;

        Ok(Self {
            mixer,
            resampler: Arc::new(Mutex::new(resampler)),
            output,
            samples_processed: 0,
            samples_output: 0,
        })
    }

    /// Create a new audio system with default configuration (48 kHz, mono)
    pub fn new_default() -> Result<Self, String> {
        Self::new(AudioConfig::new())
    }

    /// Process one APU sample (call this every APU clock)
    ///
    /// # Arguments
    ///
    /// * `pulse1` - Pulse channel 1 output (0-15)
    /// * `pulse2` - Pulse channel 2 output (0-15)
    /// * `triangle` - Triangle channel output (0-15)
    /// * `noise` - Noise channel output (0-15)
    /// * `dmc` - DMC channel output (0-127)
    pub fn process_apu_sample(&mut self, pulse1: u8, pulse2: u8, triangle: u8, noise: u8, dmc: u8) {
        // Mix the channels
        let mixed_sample = self.mixer.mix(pulse1, pulse2, triangle, noise, dmc);

        // Add to resampler
        let mut resampler = self.resampler.lock().unwrap();
        resampler.add_input_sample(mixed_sample);

        self.samples_processed += 1;

        // Check if output sample is ready
        while let Some(output_sample) = resampler.get_output_sample() {
            // Push to audio output buffer
            if !self.output.push_sample(output_sample) {
                // Buffer full - this shouldn't happen often
                // Could implement flow control here if needed
            }
            self.samples_output += 1;
        }
    }

    /// Set the master volume
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level (0.0 = mute, 1.0 = full volume)
    pub fn set_volume(&mut self, volume: f32) {
        self.mixer.set_volume(volume);
    }

    /// Get the current volume
    pub fn volume(&self) -> f32 {
        self.mixer.volume()
    }

    /// Get the number of samples in the output buffer
    pub fn buffer_len(&self) -> usize {
        self.output.buffer_len()
    }

    /// Get the output buffer capacity
    pub fn buffer_capacity(&self) -> usize {
        self.output.buffer_capacity()
    }

    /// Check if the buffer is nearly full
    pub fn is_buffer_nearly_full(&self) -> bool {
        self.output.is_buffer_nearly_full()
    }

    /// Clear the audio buffer
    pub fn clear_buffer(&self) {
        self.output.clear_buffer();
    }

    /// Get audio statistics
    pub fn stats(&self) -> AudioStats {
        AudioStats {
            samples_processed: self.samples_processed,
            samples_output: self.samples_output,
            buffer_len: self.output.buffer_len(),
            buffer_capacity: self.output.buffer_capacity(),
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.samples_processed = 0;
        self.samples_output = 0;
    }

    /// Pause audio playback
    pub fn pause(&self) -> Result<(), String> {
        self.output.pause()
    }

    /// Resume audio playback
    pub fn resume(&self) -> Result<(), String> {
        self.output.resume()
    }
}

/// Audio statistics
#[derive(Debug, Clone, Copy)]
pub struct AudioStats {
    /// Total APU samples processed
    pub samples_processed: u64,

    /// Total audio samples output
    pub samples_output: u64,

    /// Current buffer length
    pub buffer_len: usize,

    /// Buffer capacity
    pub buffer_capacity: usize,
}

impl AudioStats {
    /// Get buffer fullness as a percentage (0.0 - 1.0)
    pub fn buffer_fullness(&self) -> f32 {
        if self.buffer_capacity == 0 {
            0.0
        } else {
            self.buffer_len as f32 / self.buffer_capacity as f32
        }
    }

    /// Get the resampling ratio (output / input)
    pub fn resampling_ratio(&self) -> f64 {
        if self.samples_processed == 0 {
            0.0
        } else {
            self.samples_output as f64 / self.samples_processed as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_stats() {
        let stats = AudioStats {
            samples_processed: 1000,
            samples_output: 50,
            buffer_len: 25,
            buffer_capacity: 100,
        };

        assert_eq!(stats.buffer_fullness(), 0.25);
        assert_eq!(stats.resampling_ratio(), 0.05);
    }

    #[test]
    fn test_audio_stats_empty() {
        let stats = AudioStats {
            samples_processed: 0,
            samples_output: 0,
            buffer_len: 0,
            buffer_capacity: 100,
        };

        assert_eq!(stats.buffer_fullness(), 0.0);
        assert_eq!(stats.resampling_ratio(), 0.0);
    }

    // Note: Cannot test AudioSystem creation in unit tests as it requires audio hardware
    // Integration tests should be used for end-to-end testing
}
