# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a NES emulator written in Rust, featuring cycle-accurate emulation of the 6502 CPU, 2C02 PPU, and 2A03 APU. The project has achieved 100% pass rate on standard NES test ROMs (100/100 tests passing).

## Build and Development Commands

### Using the xtask Development Tool

This project uses a custom `xtask` tool for all development tasks. Access it via `cargo x`:

```bash
# Full CI pipeline (fmt, clippy, build, test)
cargo x ci

# Quick checks before commit (fmt, clippy)
cargo x check

# Format code
cargo x fmt
cargo x fmt --check  # Check without modifying

# Run clippy
cargo x clippy
cargo x clippy --fix  # Auto-fix issues

# Build
cargo x build
cargo x build --release

# Run tests
cargo x test                    # All tests
cargo x test --cpu              # CPU module tests only
cargo x test --ppu              # PPU module tests only
cargo x test --memory           # Memory module tests only
cargo x test --doc              # Doc tests
cargo x test --ignored          # Run ignored tests (ROM tests)

# Run benchmarks
cargo x bench

# Test with a ROM file
cargo x rom-test path/to/rom.nes
cargo x rom-test path/to/rom.nes -n 1000000  # Specify instruction count
cargo x rom-test path/to/rom.nes --release    # Use release build

# Pre-commit checks (fmt, clippy, test)
cargo x pre-commit

# Install git hooks
cargo x install-hooks
```

### Running the Emulator

```bash
# Run the main binary
cargo run --release

# Run examples
cargo run --example display_ppu_integration
cargo run --example emulator_features
cargo run --example debug_example
cargo run --example audio_test  # Requires 'audio' feature
```

### Feature Flags

- `audio`: Enable audio output (default: enabled, uses ALSA on Linux)
- Use `--no-default-features` in CI to avoid ALSA dependency issues

### Running ROM Test Suite

The project includes 100 comprehensive test ROMs covering CPU, PPU, APU, and sprite behavior:

```bash
# Run specific test
cargo test nestest_cpu_test -- --ignored --nocapture

# Run test categories
cargo test --test blargg_cpu_tests -- --ignored --nocapture
cargo test --test blargg_ppu_tests -- --ignored --nocapture
cargo test --test blargg_apu_tests -- --ignored --nocapture
cargo test --test sprite_tests -- --ignored --nocapture

# Run all tests with automation script
./tests/run_all_tests.sh
./tests/run_all_tests.sh --cpu
./tests/run_all_tests.sh --verbose
```

Note: ROM tests use `#[ignore]` to prevent running during normal `cargo test`. See `tests/README.md` for details.

## Architecture Overview

### Core Components

The emulator follows a hardware-accurate architecture with these main components:

1. **CPU (src/cpu/)** - 6502 processor emulation
   - Full instruction set implementation split across instruction modules
   - Cycle-accurate execution
   - Interrupt handling (NMI, IRQ, BRK)
   - Addressing modes in `addressing.rs`
   - Instruction execution in `execute.rs`
   - Opcode definitions in `opcodes.rs`
   - Instructions grouped by category in `instructions/` (arithmetic, logic, memory, etc.)

2. **PPU (src/ppu/)** - Picture Processing Unit (2C02)
   - Scanline-based, cycle-accurate rendering (341 cycles/scanline, 262 scanlines/frame)
   - 4-stage tile fetching pipeline
   - Shift register-based pixel rendering
   - Sprite evaluation (max 8 sprites per scanline)
   - Registers in `registers.rs`
   - Memory/VRAM in `memory.rs`
   - Rendering logic in `rendering.rs`
   - Constants in `constants.rs`

3. **APU (src/apu/)** - Audio Processing Unit (2A03)
   - 5 audio channels: 2 pulse, 1 triangle, 1 noise, 1 DMC
   - Frame counter with 4-step and 5-step modes
   - Channels in `channels/` (pulse, triangle, noise, dmc)
   - Shared components in `components/` (envelope, sweep, timers, counters)

4. **Bus (src/bus.rs)** - Memory bus connecting all components
   - Handles memory-mapped I/O for PPU, APU, controllers
   - RAM mirroring ($0000-$1FFF)
   - PPU register mirroring ($2000-$3FFF)
   - OAM DMA handling
   - Implements `MemoryMappedDevice` trait for component integration

5. **Cartridge (src/cartridge/)** - ROM loading and mapper system
   - iNES 1.0 format support
   - Multiple mapper implementations (0, 1, 2, 3, 4, 7, 9, 10, 11, 66)
   - Mirroring types (horizontal, vertical, four-screen, single-screen)
   - Each mapper in `mappers/` directory

6. **Emulator (src/emulator/)** - High-level coordination
   - Coordinates CPU, PPU, APU, and Bus
   - Configuration management
   - Save states (serialization/deserialization)
   - Recent ROMs list
   - Speed control (normal, fast-forward, slow-motion)
   - Screenshot capture

7. **Display (src/display/)** - Rendering and window management
   - Frame buffer (256×240 pixels)
   - NES color palette (52 colors)
   - Window scaling (2x, 3x, 4x)
   - Uses winit + pixels + wgpu
   - VSync and frame timing
   - Integration with egui for debug UI

8. **Input (src/input.rs, src/input/)** - Controller input handling
   - Keyboard and gamepad support via gilrs
   - Configurable key bindings (TOML)
   - NES controller interface
   - Unified input abstraction

9. **Audio (src/audio/)** - Audio output (optional feature)
   - APU output mixing
   - Sample rate conversion/resampling
   - Ring buffer for audio streaming
   - Uses cpal for audio output

10. **Debug (src/debug/)** - Debugging tools
    - CPU debugger with breakpoints and step execution
    - Memory viewer (hex dump)
    - PPU debugger (nametables, pattern tables, palettes, OAM)
    - Disassembler for 6502 instructions
    - Trace logging
    - egui-based debug UI

### Memory Map (CPU Address Space)

```
$0000-$07FF: 2KB Internal RAM
$0800-$1FFF: Mirrors of RAM (3 times)
$2000-$2007: PPU Registers
$2008-$3FFF: Mirrors of PPU Registers
$4000-$4017: APU and I/O Registers
$4020-$FFFF: Cartridge space (PRG-ROM, mapper registers)
```

### Component Communication

- **CPU ↔ Bus**: CPU reads/writes through Bus, which routes to appropriate components
- **Bus ↔ PPU**: PPU registers mapped at $2000-$2007, OAM DMA at $4014
- **Bus ↔ APU**: APU registers at $4000-$4015, $4017
- **Bus ↔ Cartridge**: Cartridge mapped at $4020-$FFFF via Mapper trait
- **PPU ↔ Cartridge**: PPU accesses CHR-ROM/RAM through mapper
- **Emulator**: Orchestrates all components, manages timing and synchronization

### Key Traits

- `MemoryMappedDevice`: For components with memory-mapped registers (PPU, APU, Cartridge)
- `Mapper`: For cartridge mappers to handle PRG/CHR banking and mirroring

## Code Organization

### Module Structure

- `src/lib.rs` - Library entry point, re-exports main types
- `src/main.rs` - Binary entry point
- Component modules follow hardware boundaries
- Test modules inline with `#[cfg(test)]` or in `tests/` for integration tests
- Examples in `examples/` demonstrate individual features

### Testing Strategy

1. **Unit tests**: Inline with modules using `#[cfg(test)]`
2. **Component tests**: In `src/*/tests/` subdirectories (CPU, PPU, APU)
3. **Integration tests**: In `tests/` directory using test ROMs
4. **Benchmarks**: In `benches/` using Criterion

### Performance Considerations

- Use `#[inline(always)]` for hot path methods (CPU instruction execution, PPU pixel rendering)
- Release profile uses LTO and single codegen unit for maximum optimization
- Debug profile optimized for fast compilation with incremental builds

## Development Workflow

### Branch Strategy

- Main branch: `main`
- Feature branches: `feat/description`
- Bug fixes: `fix/description`
- Refactoring: `refactor/description`

### Slash Commands

Custom slash commands are defined in `.claude/commands/`:

- `/impl <issue-number>`: Start implementing a GitHub issue
- `/finish [files...]`: Commit changes and create PR (keep under 100 lines)
- `/address-review`: Address code review suggestions (verify build/clippy/fmt, no commit)

### Before Committing

1. Format: `cargo x fmt`
2. Lint: `cargo x clippy`
3. Test: `cargo x test`
4. For significant changes, run: `cargo x ci`

### CI Pipeline

GitHub Actions runs on push/PR:
- Format check
- Clippy (all warnings denied)
- Build (with and without default features)
- Test suite
- Coverage reporting

## Configuration Files

- `emulator_config.toml` - Emulator settings (auto-generated)
- `input_config.toml` - Controller key bindings (auto-generated)
- `Cargo.toml` - Project dependencies and profiles
- `.cargo/config.toml` - Defines `cargo x` alias

## Hardware Accuracy Notes

### CPU (6502)
- Cycle-accurate instruction timing
- Proper flag behavior for all instructions
- Interrupt priority and timing
- Dummy reads where appropriate

### PPU (2C02)
- Scanline-based rendering (341 PPU cycles per scanline)
- Proper VRAM address behavior with t/v/x/w registers
- PPUDATA read buffering
- Sprite 0 hit detection
- Sprite overflow handling
- VBlank/NMI timing

### APU (2A03)
- Frame counter with 4-step and 5-step modes
- Length counters, envelopes, sweeps
- Triangle linear counter
- Noise LFSR
- DMC sample playback
- IRQ generation

### Mappers
Currently implemented mappers: 0, 1, 2, 3, 4, 7, 9, 10, 11, 66
Most common games are supported through these mappers.

## Additional Resources

- Test ROM documentation: `tests/README.md`, `tests/TEST_STATUS.md`
- Gamepad setup: `GAMEPAD_SETUP.md`
- Debug UI: `src/debug/README.md`
