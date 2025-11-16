// Audio output - Handles audio playback using cpal
//
// This module provides cross-platform audio output using the cpal library.
// It manages the audio device, stream, and callback for audio playback.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

use super::resampler::AudioBuffer;

/// Audio output configuration
#[derive(Clone)]
pub struct AudioConfig {
    /// Sample rate in Hz (44100 or 48000)
    pub sample_rate: u32,

    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,

    /// Buffer size in milliseconds (affects latency)
    pub buffer_duration_ms: u32,
}

impl AudioConfig {
    /// Create default audio configuration
    ///
    /// - Sample rate: 48 kHz
    /// - Channels: 1 (mono)
    /// - Buffer duration: 50 ms
    pub fn new() -> Self {
        Self {
            sample_rate: 48000,
            channels: 1,
            buffer_duration_ms: 50,
        }
    }

    /// Set the sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Set the number of channels (1 = mono, 2 = stereo)
    pub fn with_channels(mut self, channels: u16) -> Self {
        self.channels = channels;
        self
    }

    /// Set the buffer duration in milliseconds
    pub fn with_buffer_duration(mut self, duration_ms: u32) -> Self {
        self.buffer_duration_ms = duration_ms;
        self
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio output handle
///
/// Manages the audio device and stream for playback.
pub struct AudioOutput {
    /// Audio configuration
    config: AudioConfig,

    /// Audio device
    _device: Device,

    /// Audio stream
    stream: Stream,

    /// Shared audio buffer
    buffer: Arc<Mutex<AudioBuffer>>,
}

impl AudioOutput {
    /// Create a new audio output
    ///
    /// # Arguments
    ///
    /// * `config` - Audio configuration
    ///
    /// # Returns
    ///
    /// Result containing the AudioOutput or an error message
    pub fn new(config: AudioConfig) -> Result<Self, String> {
        // Get default audio host
        let host = cpal::default_host();

        // Get default output device
        let device = host
            .default_output_device()
            .ok_or("No output device available")?;

        println!("Audio device: {}", device.name().unwrap_or_default());

        // Create stream configuration
        let stream_config = StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create shared audio buffer
        let buffer_capacity =
            ((config.buffer_duration_ms as f64 / 1000.0) * config.sample_rate as f64) as usize;
        let buffer = Arc::new(Mutex::new(AudioBuffer::new(buffer_capacity)));

        // Clone buffer for the audio callback
        let buffer_clone = Arc::clone(&buffer);

        // Create audio stream with callback
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Fill the output buffer with samples from our buffer
                    let mut buf = buffer_clone.lock().unwrap();

                    for sample in data.iter_mut() {
                        *sample = buf.pop().unwrap_or(0.0);
                    }
                },
                move |err| {
                    eprintln!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build audio stream: {}", e))?;

        // Start the stream
        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))?;

        println!(
            "Audio output initialized: {} Hz, {} channel(s)",
            config.sample_rate, config.channels
        );

        Ok(Self {
            config,
            _device: device,
            stream,
            buffer,
        })
    }

    /// Push a sample to the audio buffer
    ///
    /// Returns true if successful, false if buffer is full.
    ///
    /// # Arguments
    ///
    /// * `sample` - Audio sample to push (f32 in range [-1.0, 1.0])
    pub fn push_sample(&self, sample: f32) -> bool {
        let mut buf = self.buffer.lock().unwrap();
        buf.push(sample)
    }

    /// Get the number of samples currently in the buffer
    pub fn buffer_len(&self) -> usize {
        let buf = self.buffer.lock().unwrap();
        buf.len()
    }

    /// Get the buffer capacity
    pub fn buffer_capacity(&self) -> usize {
        let buf = self.buffer.lock().unwrap();
        buf.capacity()
    }

    /// Check if the buffer is nearly full (> 90% capacity)
    ///
    /// This can be used to implement flow control.
    pub fn is_buffer_nearly_full(&self) -> bool {
        let buf = self.buffer.lock().unwrap();
        buf.len() > (buf.capacity() * 9 / 10)
    }

    /// Clear the audio buffer
    pub fn clear_buffer(&self) {
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
    }

    /// Get the audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Pause audio playback
    pub fn pause(&self) -> Result<(), String> {
        self.stream
            .pause()
            .map_err(|e| format!("Failed to pause audio: {}", e))
    }

    /// Resume audio playback
    pub fn resume(&self) -> Result<(), String> {
        self.stream
            .play()
            .map_err(|e| format!("Failed to resume audio: {}", e))
    }
}

/// Audio output builder for easier configuration
pub struct AudioOutputBuilder {
    config: AudioConfig,
}

impl AudioOutputBuilder {
    /// Create a new audio output builder with default configuration
    pub fn new() -> Self {
        Self {
            config: AudioConfig::new(),
        }
    }

    /// Set the sample rate
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.config.sample_rate = sample_rate;
        self
    }

    /// Set the number of channels
    pub fn channels(mut self, channels: u16) -> Self {
        self.config.channels = channels;
        self
    }

    /// Set the buffer duration
    pub fn buffer_duration(mut self, duration_ms: u32) -> Self {
        self.config.buffer_duration_ms = duration_ms;
        self
    }

    /// Build the audio output
    pub fn build(self) -> Result<AudioOutput, String> {
        AudioOutput::new(self.config)
    }
}

impl Default for AudioOutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_creation() {
        let config = AudioConfig::new();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.buffer_duration_ms, 50);
    }

    #[test]
    fn test_audio_config_builder() {
        let config = AudioConfig::new()
            .with_sample_rate(44100)
            .with_channels(2)
            .with_buffer_duration(100);

        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 2);
        assert_eq!(config.buffer_duration_ms, 100);
    }

    #[test]
    fn test_audio_output_builder() {
        let builder = AudioOutputBuilder::new()
            .sample_rate(44100)
            .channels(2)
            .buffer_duration(100);

        assert_eq!(builder.config.sample_rate, 44100);
        assert_eq!(builder.config.channels, 2);
        assert_eq!(builder.config.buffer_duration_ms, 100);
    }

    // Note: Cannot test actual audio output in unit tests as it requires audio hardware
    // Integration tests should be used for end-to-end audio testing
}
