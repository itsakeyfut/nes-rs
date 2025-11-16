// Audio resampler - Converts NES APU sample rate to standard audio rates
//
// The NES APU generates samples at the CPU clock rate (approximately 1.789773 MHz).
// Modern audio hardware expects samples at standard rates like 44.1 kHz or 48 kHz.
// This module handles the conversion using simple linear interpolation.

/// Sample rate constants
pub mod sample_rates {
    /// NES CPU clock rate (NTSC) in Hz
    /// This is the rate at which the APU generates samples
    pub const NES_CPU_CLOCK: f64 = 1_789_773.0;

    /// Standard audio sample rate: 44.1 kHz (CD quality)
    pub const AUDIO_44_1_KHZ: f64 = 44_100.0;

    /// Standard audio sample rate: 48 kHz (common for digital audio)
    pub const AUDIO_48_KHZ: f64 = 48_000.0;
}

/// Audio resampler using linear interpolation
///
/// Converts from NES APU sample rate (~1.79 MHz) to standard audio rates
/// (44.1 kHz or 48 kHz) using linear interpolation.
pub struct Resampler {
    /// Input sample rate (NES APU rate)
    input_rate: f64,

    /// Output sample rate (audio hardware rate)
    output_rate: f64,

    /// Current time position in the input stream
    time_position: f64,

    /// Previous sample for interpolation
    prev_sample: f32,

    /// Current sample for interpolation
    current_sample: f32,

    /// Time increment per output sample
    time_increment: f64,
}

impl Resampler {
    /// Create a new resampler
    ///
    /// # Arguments
    ///
    /// * `input_rate` - Input sample rate (NES APU rate, ~1.79 MHz)
    /// * `output_rate` - Output sample rate (44.1 kHz or 48 kHz)
    pub fn new(input_rate: f64, output_rate: f64) -> Self {
        Self {
            input_rate,
            output_rate,
            time_position: 0.0,
            prev_sample: 0.0,
            current_sample: 0.0,
            time_increment: input_rate / output_rate,
        }
    }

    /// Create a resampler for 44.1 kHz output
    pub fn new_44_1_khz() -> Self {
        Self::new(sample_rates::NES_CPU_CLOCK, sample_rates::AUDIO_44_1_KHZ)
    }

    /// Create a resampler for 48 kHz output
    pub fn new_48_khz() -> Self {
        Self::new(sample_rates::NES_CPU_CLOCK, sample_rates::AUDIO_48_KHZ)
    }

    /// Add an input sample from the APU
    ///
    /// Call this method every APU clock cycle with the current mixed output.
    ///
    /// # Arguments
    ///
    /// * `sample` - Input sample from the APU mixer
    pub fn add_input_sample(&mut self, sample: f32) {
        self.prev_sample = self.current_sample;
        self.current_sample = sample;
        self.time_position += 1.0;
    }

    /// Get the next output sample
    ///
    /// Returns None if no output sample is ready yet.
    /// Returns Some(sample) when an output sample is available.
    ///
    /// # Returns
    ///
    /// Optional f32 sample in range [-1.0, 1.0]
    pub fn get_output_sample(&mut self) -> Option<f32> {
        if self.time_position >= self.time_increment {
            // Calculate interpolation factor
            let frac = (self.time_position % self.time_increment) / self.time_increment;

            // Linear interpolation
            let sample = self.prev_sample + (self.current_sample - self.prev_sample) * frac as f32;

            // Reset time position for next output sample
            self.time_position -= self.time_increment;

            Some(sample)
        } else {
            None
        }
    }

    /// Reset the resampler state
    pub fn reset(&mut self) {
        self.time_position = 0.0;
        self.prev_sample = 0.0;
        self.current_sample = 0.0;
    }

    /// Get the input sample rate
    pub fn input_rate(&self) -> f64 {
        self.input_rate
    }

    /// Get the output sample rate
    pub fn output_rate(&self) -> f64 {
        self.output_rate
    }
}

/// Audio buffer for storing resampled audio samples
///
/// This is a ring buffer that can be used to store samples for audio output.
pub struct AudioBuffer {
    /// Internal ring buffer
    buffer: Vec<f32>,

    /// Read position
    read_pos: usize,

    /// Write position
    write_pos: usize,

    /// Number of samples in the buffer
    count: usize,
}

impl AudioBuffer {
    /// Create a new audio buffer
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of samples the buffer can hold
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    /// Create a buffer sized for approximately N milliseconds at the given sample rate
    ///
    /// # Arguments
    ///
    /// * `milliseconds` - Duration in milliseconds
    /// * `sample_rate` - Sample rate in Hz
    pub fn with_duration(milliseconds: u32, sample_rate: f64) -> Self {
        let capacity = ((milliseconds as f64 / 1000.0) * sample_rate) as usize;
        Self::new(capacity)
    }

    /// Push a sample into the buffer
    ///
    /// Returns true if successful, false if buffer is full.
    ///
    /// # Arguments
    ///
    /// * `sample` - Audio sample to push
    pub fn push(&mut self, sample: f32) -> bool {
        if self.count >= self.buffer.len() {
            return false; // Buffer full
        }

        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        self.count += 1;
        true
    }

    /// Pop a sample from the buffer
    ///
    /// Returns None if buffer is empty.
    pub fn pop(&mut self) -> Option<f32> {
        if self.count == 0 {
            return None; // Buffer empty
        }

        let sample = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % self.buffer.len();
        self.count -= 1;
        Some(sample)
    }

    /// Get the number of samples in the buffer
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.count >= self.buffer.len()
    }

    /// Get the buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resampler_creation() {
        let resampler = Resampler::new_44_1_khz();
        assert_eq!(resampler.input_rate(), sample_rates::NES_CPU_CLOCK);
        assert_eq!(resampler.output_rate(), sample_rates::AUDIO_44_1_KHZ);

        let resampler = Resampler::new_48_khz();
        assert_eq!(resampler.output_rate(), sample_rates::AUDIO_48_KHZ);
    }

    #[test]
    fn test_resampler_basic() {
        let mut resampler = Resampler::new(1000.0, 100.0); // 10:1 ratio for testing

        // Add 10 input samples
        for i in 0..10 {
            resampler.add_input_sample(i as f32 / 10.0);
        }

        // Should have 1 output sample ready
        let sample = resampler.get_output_sample();
        assert!(sample.is_some());
    }

    #[test]
    fn test_audio_buffer_basic() {
        let mut buffer = AudioBuffer::new(10);
        assert_eq!(buffer.capacity(), 10);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        // Push samples
        for i in 0..5 {
            assert!(buffer.push(i as f32));
        }

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
        assert!(!buffer.is_full());

        // Pop samples
        for i in 0..5 {
            assert_eq!(buffer.pop(), Some(i as f32));
        }

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_audio_buffer_overflow() {
        let mut buffer = AudioBuffer::new(3);

        // Fill buffer
        assert!(buffer.push(1.0));
        assert!(buffer.push(2.0));
        assert!(buffer.push(3.0));
        assert!(buffer.is_full());

        // Try to push one more (should fail)
        assert!(!buffer.push(4.0));
    }

    #[test]
    fn test_audio_buffer_underflow() {
        let mut buffer = AudioBuffer::new(3);
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_audio_buffer_wrap_around() {
        let mut buffer = AudioBuffer::new(3);

        // Fill and empty multiple times
        for _ in 0..10 {
            buffer.push(1.0);
            buffer.push(2.0);
            buffer.push(3.0);

            assert_eq!(buffer.pop(), Some(1.0));
            assert_eq!(buffer.pop(), Some(2.0));
            assert_eq!(buffer.pop(), Some(3.0));
        }

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_audio_buffer_clear() {
        let mut buffer = AudioBuffer::new(10);
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);

        assert_eq!(buffer.len(), 3);

        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_audio_buffer_with_duration() {
        let buffer = AudioBuffer::with_duration(100, 44100.0); // 100ms at 44.1kHz
        assert_eq!(buffer.capacity(), 4410);
    }
}
