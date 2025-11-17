// Sprite and OAM Test Suite
//
// These tests validate sprite rendering, sprite hit detection,
// and sprite overflow behavior.
//
// Sprites are hardware-accelerated moving objects on the NES.

mod common;

use common::run_blargg_style_test;

/// Run a sprite test ROM and check the result
///
/// This is a thin wrapper around the common test runner that uses
/// the standard timeout value for sprite tests.
fn run_sprite_test(rom_path: &str) -> Result<(bool, String), String> {
    run_blargg_style_test(rom_path, 100_000_000)
}

// ============================================================================
// Sprite Hit Tests 2005.10.05
// ============================================================================

#[test]
#[ignore] // Run with: cargo test sprite_tests -- --ignored --nocapture
fn sprite_hit_basics() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/01.basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Basics:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_alignment() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/02.alignment.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Alignment:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_corners() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/03.corners.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Corners:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_flip() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/04.flip.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Flip:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_left_clip() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/05.left_clip.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Left Clip:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_right_edge() {
    let result =
        run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/06.right_edge.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Right Edge:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_screen_bottom() {
    let result =
        run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/07.screen_bottom.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Screen Bottom:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_double_height() {
    let result =
        run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/08.double_height.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Double Height:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_timing_basics() {
    let result =
        run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/09.timing_basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Timing Basics:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_timing_order() {
    let result =
        run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/10.timing_order.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Timing Order:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_hit_edge_timing() {
    let result =
        run_sprite_test("tests/nes-test-rom/sprite_hit_tests_2005.10.05/11.edge_timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Hit Edge Timing:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

// ============================================================================
// Sprite Overflow Tests
// ============================================================================

#[test]
#[ignore]
fn sprite_overflow_basics() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_overflow_tests/1.Basics.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Overflow Basics:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_overflow_details() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_overflow_tests/2.Details.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Overflow Details:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_overflow_timing() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_overflow_tests/3.Timing.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Overflow Timing:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_overflow_obscure() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_overflow_tests/4.Obscure.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Overflow Obscure:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}

#[test]
#[ignore]
fn sprite_overflow_emulator() {
    let result = run_sprite_test("tests/nes-test-rom/sprite_overflow_tests/5.Emulator.nes");

    match result {
        Ok((passed, message)) => {
            println!("\nSprite Overflow Emulator:");
            println!("{}", message);
            assert!(passed, "Test failed: {}", message);
        }
        Err(e) => {
            panic!("Test error: {}", e);
        }
    }
}
