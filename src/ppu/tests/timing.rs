//! PPU Timing Tests
//!
//! Tests for cycle-accurate PPU timing including:
//! - Cycle and scanline tracking
//! - Frame completion
//! - VBlank flag timing
//! - NMI generation
//! - Odd/even frame behavior

use super::*;

// Cycle-accurate timing tests
// ========================================

#[test]
fn test_ppu_cycle_tracking() {
    let mut ppu = Ppu::new();

    // Initial state
    assert_eq!(ppu.scanline(), 0, "PPU should start at scanline 0");
    assert_eq!(ppu.cycle(), 0, "PPU should start at cycle 0");
    assert_eq!(ppu.frame_count(), 0, "PPU should start at frame 0");

    // Execute one cycle
    ppu.step();
    assert_eq!(ppu.cycle(), 1, "Cycle should advance to 1");
    assert_eq!(ppu.scanline(), 0, "Scanline should remain 0");
}

#[test]
fn test_ppu_scanline_advancement() {
    let mut ppu = Ppu::new();

    // Execute a full scanline (341 cycles)
    for _ in 0..CYCLES_PER_SCANLINE {
        ppu.step();
    }

    assert_eq!(ppu.scanline(), 1, "Scanline should advance to 1");
    assert_eq!(ppu.cycle(), 0, "Cycle should reset to 0");
}

#[test]
fn test_ppu_frame_completion() {
    let mut ppu = Ppu::new();

    // Execute cycles until a frame completes
    let mut frame_complete = false;
    let mut cycles_executed = 0;

    // Execute one full frame (262 scanlines × 341 cycles = 89,342 cycles)
    while !frame_complete && cycles_executed < CYCLES_PER_FRAME + 1000 {
        frame_complete = ppu.step();
        cycles_executed += 1;
    }

    assert!(
        frame_complete,
        "A frame should complete after one full frame of cycles"
    );
    assert_eq!(ppu.scanline(), 0, "Scanline should reset to 0 after frame");
    assert_eq!(ppu.frame_count(), 1, "Frame counter should be 1");
}

#[test]
fn test_vblank_flag_set() {
    let mut ppu = Ppu::new();

    // Execute until scanline 241, cycle 1 (VBlank start)
    // Scanlines 0-240 are visible/post-render
    for _ in 0..=240 {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    // Now we're at scanline 241, cycle 0
    assert_eq!(ppu.scanline(), 241, "Should be at VBlank scanline");

    // Execute one more cycle to trigger VBlank flag
    ppu.step();

    // Check VBlank flag is set (bit 7 of PPUSTATUS)
    assert_ne!(
        ppu.ppustatus & 0x80,
        0,
        "VBlank flag should be set at scanline 241, cycle 1"
    );
}

#[test]
fn test_vblank_nmi_generation() {
    let mut ppu = Ppu::new();

    // Enable NMI on VBlank
    ppu.ppuctrl = 0x80; // Set bit 7 to enable NMI

    // Execute until scanline 241, cycle 1 (VBlank start)
    for _ in 0..=240 {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    // Execute one more cycle to trigger VBlank and NMI
    ppu.step();

    // Check NMI is pending
    assert!(
        ppu.nmi_pending(),
        "NMI should be pending after VBlank starts"
    );
}

#[test]
fn test_vblank_nmi_disabled() {
    let mut ppu = Ppu::new();

    // NMI is disabled by default (ppuctrl bit 7 = 0)
    assert_eq!(ppu.ppuctrl & 0x80, 0, "NMI should be disabled");

    // Execute until scanline 241, cycle 1 (VBlank start)
    for _ in 0..=240 {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    // Execute one more cycle to trigger VBlank
    ppu.step();

    // Check NMI is NOT pending
    assert!(
        !ppu.nmi_pending(),
        "NMI should not be pending when disabled"
    );
}

#[test]
fn test_prerender_scanline_clears_flags() {
    let mut ppu = Ppu::new();

    // Set VBlank and sprite flags
    ppu.ppustatus = 0xE0; // Set VBlank, Sprite 0 hit, Sprite overflow

    // Execute until pre-render scanline (261), cycle 1
    // We need to go through scanlines 0-260 first
    for _ in 0..261 {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    // Now we're at scanline 261, cycle 0
    assert_eq!(ppu.scanline(), 261, "Should be at pre-render scanline");

    // Execute one more cycle to trigger flag clearing
    ppu.step();

    // Check all flags are cleared
    assert_eq!(
        ppu.ppustatus & 0xE0,
        0,
        "VBlank, Sprite 0 hit, and Sprite overflow flags should be cleared"
    );
    assert!(
        !ppu.nmi_pending(),
        "NMI pending flag should be cleared at pre-render scanline"
    );
}

#[test]
fn test_nmi_clear() {
    let mut ppu = Ppu::new();

    // Set NMI pending
    ppu.nmi_pending = true;
    assert!(ppu.nmi_pending(), "NMI should be pending");

    // Clear NMI
    ppu.clear_nmi();
    assert!(!ppu.nmi_pending(), "NMI should be cleared");
}

#[test]
fn test_multiple_frames() {
    let mut ppu = Ppu::new();

    let mut frames_completed = 0;

    // Execute several frames
    for _ in 0..(CYCLES_PER_FRAME * 3) {
        if ppu.step() {
            frames_completed += 1;
        }
    }

    assert_eq!(
        frames_completed, 3,
        "Should complete 3 frames after 3× frame cycles"
    );
    assert_eq!(ppu.frame_count(), 3, "Frame counter should be 3");
}

#[test]
fn test_cycle_counts() {
    // Verify constants are correct
    assert_eq!(
        CYCLES_PER_SCANLINE, 341,
        "PPU should have 341 cycles per scanline"
    );
    assert_eq!(
        SCANLINES_PER_FRAME, 262,
        "PPU should have 262 scanlines per frame (NTSC)"
    );
    assert_eq!(
        CYCLES_PER_FRAME, 89342,
        "PPU should have 89,342 cycles per frame (341 × 262)"
    );
}

#[test]
fn test_scanline_types() {
    // Verify scanline constants
    assert_eq!(FIRST_VISIBLE_SCANLINE, 0, "First visible scanline is 0");
    assert_eq!(LAST_VISIBLE_SCANLINE, 239, "Last visible scanline is 239");
    assert_eq!(POSTRENDER_SCANLINE, 240, "Post-render scanline is 240");
    assert_eq!(FIRST_VBLANK_SCANLINE, 241, "First VBlank scanline is 241");
    assert_eq!(LAST_VBLANK_SCANLINE, 260, "Last VBlank scanline is 260");
    assert_eq!(PRERENDER_SCANLINE, 261, "Pre-render scanline is 261");
}

#[test]
fn test_rendering_enabled_check() {
    let mut ppu = Ppu::new();

    // Initially, rendering is disabled
    assert!(
        !ppu.is_rendering_enabled(),
        "Rendering should be disabled initially"
    );

    // Enable background rendering (bit 3)
    ppu.ppumask = 0x08;
    assert!(
        ppu.is_rendering_enabled(),
        "Rendering should be enabled with background"
    );

    // Disable background, enable sprites (bit 4)
    ppu.ppumask = 0x10;
    assert!(
        ppu.is_rendering_enabled(),
        "Rendering should be enabled with sprites"
    );

    // Enable both
    ppu.ppumask = 0x18;
    assert!(
        ppu.is_rendering_enabled(),
        "Rendering should be enabled with both"
    );

    // Disable both
    ppu.ppumask = 0x00;
    assert!(
        !ppu.is_rendering_enabled(),
        "Rendering should be disabled with neither"
    );
}

#[test]
fn test_odd_frame_skips_last_cycle() {
    let mut ppu = Ppu::new();

    // Enable rendering to trigger odd frame behavior
    ppu.ppumask = 0x18; // Enable background and sprites

    // Execute until pre-render scanline 261, cycle 339 on frame 1 (odd)
    // First complete frame 0 (even frame)
    while ppu.frame_count() < 1 {
        ppu.step();
    }

    assert_eq!(ppu.frame_count(), 1, "Should be on frame 1");
    assert_eq!(ppu.scanline(), 0, "Should be at scanline 0");
    assert_eq!(ppu.cycle(), 0, "Should be at cycle 0");

    // Now on frame 1 (odd), advance to scanline 261
    while ppu.scanline() < PRERENDER_SCANLINE {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    assert_eq!(
        ppu.scanline(),
        PRERENDER_SCANLINE,
        "Should be at pre-render scanline"
    );

    // Advance to cycle 339 (CYCLES_PER_SCANLINE - 2)
    // We need to advance 339 cycles (0 to 339)
    for _ in 0..339 {
        ppu.step();
    }

    assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
    assert_eq!(ppu.cycle(), 339);

    // Next step should skip cycle 340 and complete the frame
    let frame_complete = ppu.step();

    assert!(frame_complete, "Frame should complete");
    assert_eq!(ppu.frame_count(), 2, "Should advance to frame 2");
    assert_eq!(ppu.scanline(), 0, "Should wrap to scanline 0");
    assert_eq!(ppu.cycle(), 0, "Should reset to cycle 0");
}

#[test]
fn test_even_frame_does_not_skip_last_cycle() {
    let mut ppu = Ppu::new();

    // Enable rendering
    ppu.ppumask = 0x18;

    // Frame 0 is even, advance to scanline 261, cycle 339
    while ppu.scanline() < PRERENDER_SCANLINE {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    // Advance to cycle 339
    for _ in 0..339 {
        ppu.step();
    }

    assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
    assert_eq!(ppu.cycle(), 339);
    assert_eq!(ppu.frame_count(), 0, "Still on frame 0 (even)");

    // Next step should go to cycle 340 (not skip)
    let frame_complete = ppu.step();

    assert!(!frame_complete, "Frame should not complete yet");
    assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
    assert_eq!(
        ppu.cycle(),
        340,
        "Should advance to cycle 340 on even frame"
    );
    assert_eq!(ppu.frame_count(), 0, "Still on frame 0");

    // One more step completes the frame normally
    let frame_complete = ppu.step();
    assert!(frame_complete, "Frame should complete now");
    assert_eq!(ppu.frame_count(), 1);
    assert_eq!(ppu.scanline(), 0);
    assert_eq!(ppu.cycle(), 0);
}

#[test]
fn test_odd_frame_skip_only_when_rendering_enabled() {
    let mut ppu = Ppu::new();

    // Disable rendering
    ppu.ppumask = 0x00;

    // Complete frame 0
    while ppu.frame_count() < 1 {
        ppu.step();
    }

    assert_eq!(ppu.frame_count(), 1, "On odd frame");

    // Advance to scanline 261, cycle 339
    while ppu.scanline() < PRERENDER_SCANLINE {
        for _ in 0..CYCLES_PER_SCANLINE {
            ppu.step();
        }
    }

    for _ in 0..339 {
        ppu.step();
    }

    assert_eq!(ppu.scanline(), PRERENDER_SCANLINE);
    assert_eq!(ppu.cycle(), 339);

    // With rendering disabled, should NOT skip even on odd frame
    let frame_complete = ppu.step();

    assert!(!frame_complete, "Should not complete yet");
    assert_eq!(ppu.cycle(), 340, "Should advance to cycle 340");

    // One more step completes normally
    let frame_complete = ppu.step();
    assert!(frame_complete);
    assert_eq!(ppu.frame_count(), 2);
}
