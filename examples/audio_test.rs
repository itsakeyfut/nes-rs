// Audio system test example
//
// This example demonstrates how to use the audio system with the APU.
// It generates a simple test tone to verify audio output is working.

use nes_rs::apu::Apu;
use nes_rs::audio::{AudioConfig, AudioSystem};
use nes_rs::MemoryMappedDevice;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NES Audio System Test");
    println!("====================\n");

    // Create audio system with 48 kHz sample rate
    let audio_config = AudioConfig::new()
        .with_sample_rate(48000)
        .with_channels(1)
        .with_buffer_duration(50);

    println!("Initializing audio system...");
    let mut audio_system = AudioSystem::new(audio_config)?;
    println!("Audio system initialized!\n");

    // Create APU instance
    let mut apu = Apu::new();

    // Enable pulse channel 1
    apu.write(0x4015, 0x01);

    // Configure pulse channel 1 for a 440 Hz tone (A4 note)
    // Duty cycle 50%, constant volume 15
    apu.write(0x4000, 0b10111111); // Duty 50%, constant volume, volume 15

    // Calculate timer value for 440 Hz
    // Formula: timer = (CPU_CLOCK / (16 * frequency)) - 1
    // CPU_CLOCK = 1789773 Hz, frequency = 440 Hz
    // timer = (1789773 / (16 * 440)) - 1 = 253
    let timer: u16 = 253;
    apu.write(0x4002, (timer & 0xFF) as u8); // Timer low
    apu.write(0x4003, ((timer >> 8) & 0x07) as u8); // Timer high + length counter load

    println!("Playing 440 Hz tone (A4 note) for 3 seconds...");
    println!("You should hear a pure tone from your speakers/headphones.\n");

    // Simulate APU running for 3 seconds
    let duration_ms = 3000;
    let apu_clock_rate = 1_789_773.0; // Hz
    let total_cycles = (apu_clock_rate * (duration_ms as f64 / 1000.0)) as u64;

    for cycle in 0..total_cycles {
        // Clock the APU
        apu.clock();

        // Get channel outputs
        let pulse1 = apu.pulse1_output();
        let pulse2 = apu.pulse2_output();
        let triangle = apu.triangle_output();
        let noise = apu.noise_output();
        let dmc = apu.dmc_output();

        // Process audio sample
        audio_system.process_apu_sample(pulse1, pulse2, triangle, noise, dmc);

        // Print progress every second
        if cycle % 1_789_773 == 0 {
            let second = cycle / 1_789_773;
            let stats = audio_system.stats();
            println!(
                "Second {}: Buffer {}% full, {} samples processed",
                second,
                (stats.buffer_fullness() * 100.0) as u32,
                stats.samples_processed
            );
        }

        // Add small delay to prevent buffer overflow
        // In a real emulator, this would be handled by video sync
        if cycle % 1000 == 0 && audio_system.is_buffer_nearly_full() {
            thread::sleep(Duration::from_micros(10));
        }
    }

    println!("\nAudio test completed!");
    println!("\nFinal statistics:");
    let stats = audio_system.stats();
    println!("  APU samples processed: {}", stats.samples_processed);
    println!("  Audio samples output: {}", stats.samples_output);
    println!("  Resampling ratio: {:.6}", stats.resampling_ratio());
    println!(
        "  Buffer utilization: {:.1}%",
        stats.buffer_fullness() * 100.0
    );

    // Let the audio finish playing
    println!("\nWaiting for audio buffer to drain...");
    while audio_system.buffer_len() > 0 {
        thread::sleep(Duration::from_millis(10));
    }

    println!("Done!");
    Ok(())
}
