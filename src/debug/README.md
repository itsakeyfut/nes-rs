# Debug Module

The debug module provides comprehensive debugging tools for the NES emulator, including CPU and PPU state inspection, memory viewing, instruction disassembly, and execution tracing.

## Features

### CPU Debugger
- **Step Execution**: Execute one instruction at a time
- **Breakpoints**: Set address-based breakpoints
- **Register Dump**: View all CPU registers and status flags
- **Disassembly**: Disassemble instructions at any address
- **Stack Inspection**: View stack contents with SP highlighting

### Memory Viewer
- **CPU Memory**: Hex dump of CPU address space ($0000-$FFFF)
- **PPU Memory**: View nametables, pattern tables, palette RAM, and OAM
- **Memory Search**: Search for byte patterns in memory
- **Formatted Output**: Hex dump with ASCII representation

### PPU Debugger
- **Nametable Viewer**: Inspect nametable contents
- **Pattern Table Viewer**: View CHR-ROM/RAM tiles
- **Palette Viewer**: Display all background and sprite palettes
- **OAM Viewer**: Inspect sprite data (position, tile, attributes)
- **PPU State**: View all PPU registers and internal state

### Logging System
- **CPU Trace**: Log every instruction executed
- **PPU Trace**: Log PPU state changes
- **Configurable Log Levels**: None, Error, Warning, Info, Debug, Trace
- **File Output**: Write logs to file
- **Memory Buffer**: Keep recent traces in memory

## Usage

### Basic Setup

```rust
use nes_rs::{Bus, Cpu, Debugger, Ppu};

// Create emulator components
let mut cpu = Cpu::new();
let mut ppu = Ppu::new();
let mut bus = Bus::new();

// Create and enable debugger
let mut debugger = Debugger::new();
debugger.enable();
```

### Setting Breakpoints

```rust
// Add breakpoint at program start
debugger.add_breakpoint(0x8000);

// Add multiple breakpoints
debugger.add_breakpoint(0x8100);
debugger.add_breakpoint(0x9000);

// Check if we should break
if debugger.should_break(&cpu) {
    println!("Breakpoint hit at ${:04X}", cpu.pc);
    debugger.pause();
}

// Remove breakpoint
debugger.remove_breakpoint(0x8000);

// Clear all breakpoints
debugger.clear_breakpoints();
```

### Step Execution

```rust
// Execute one instruction and pause
debugger.step();

// Check if paused
if debugger.is_paused() {
    println!("Execution paused");
}

// Resume execution
debugger.resume();
```

### Viewing CPU State

```rust
// Get current CPU state
let state = debugger.get_cpu_state(&cpu, &mut bus);

// Display formatted state
println!("{}", state);

// Access individual fields
println!("PC: ${:04X}", state.pc);
println!("A: ${:02X}", state.a);
println!("Flags: {}", state.format_status());

// Dump all registers
println!("{}", debugger.cpu.dump_registers(&cpu));
```

### Memory Viewing

```rust
// Dump CPU memory
let dump = debugger.memory.dump_cpu_memory(&mut bus, 0x8000, 256);
println!("{}", dump);

// Dump specific regions
println!("{}", debugger.memory.dump_zero_page(&mut bus));
println!("{}", debugger.memory.dump_stack(&mut bus));

// Read individual bytes/words
let byte = debugger.memory.read_byte(&mut bus, 0x1234);
let word = debugger.memory.read_word(&mut bus, 0x1234);

// Search for patterns
let pattern = vec![0xDE, 0xAD, 0xBE, 0xEF];
let matches = debugger.memory.search_cpu_memory(&mut bus, &pattern, 0x0000, 0xFFFF);
```

### Disassembly

```rust
use nes_rs::debug::{disassemble_instruction, disassemble_count, disassemble_range};

// Disassemble single instruction
let instr = disassemble_instruction(0x8000, &mut bus);
println!("{}", instr.format_assembly());

// Disassemble multiple instructions
let instructions = disassemble_count(0x8000, 10, &mut bus);
for instr in instructions {
    println!("{}", instr);
}

// Disassemble address range
let instructions = disassemble_range(0x8000, 0x8100, &mut bus);
```

### PPU Debugging

```rust
// Get PPU state
let ppu_state = debugger.get_ppu_state(&ppu);
println!("{}", ppu_state.format());

// View palettes
println!("{}", debugger.ppu.format_palettes(&ppu));

// View sprites
let sprites = debugger.ppu.get_visible_sprites(&ppu);
for sprite in sprites {
    println!("{}", sprite.format());
}

// Dump OAM
println!("{}", debugger.ppu.format_oam(&ppu, true)); // visible only
```

### Logging and Tracing

```rust
use nes_rs::LogLevel;

// Enable CPU trace logging
debugger.logger.enable_cpu_trace();
debugger.logger.set_log_level(LogLevel::Trace);

// Enable PPU trace logging
debugger.logger.enable_ppu_trace();

// Log to file
debugger.logger.open_log_file("trace.log").unwrap();

// In your emulation loop
if debugger.before_instruction(&cpu, &mut bus) {
    // Execute instruction...
}

debugger.after_ppu_step(&ppu);

// View trace buffer
let recent = debugger.logger.last_entries(10);
for entry in recent {
    println!("{}", entry);
}

// Save trace to string
let trace = debugger.logger.format_trace_buffer();
```

### Integration with Emulator Loop

```rust
fn run_with_debugger(cpu: &mut Cpu, ppu: &mut Ppu, bus: &mut Bus, debugger: &mut Debugger) {
    loop {
        // Check breakpoints and stepping before each instruction
        if !debugger.before_instruction(cpu, bus) {
            // Execution paused - wait for user input
            break;
        }

        // Execute CPU instruction
        cpu.step(bus);

        // Execute PPU cycles (3 PPU cycles per CPU cycle)
        for _ in 0..3 {
            ppu.step();
            debugger.after_ppu_step(ppu);
        }
    }
}
```

## Performance Considerations

The debugger is designed to have minimal overhead when disabled:

```rust
// Disable debugging for production/fast execution
debugger.disable();

// Debugging checks are skipped when disabled
if !debugger.is_enabled() {
    // Fast path
}
```

When debugging is enabled, you can reduce overhead by:
- Disabling stack capture: `debugger.cpu.set_capture_stack(false)`
- Limiting trace buffer size: `debugger.logger.set_max_buffer_size(1000)`
- Using appropriate log levels (avoid `Trace` for long runs)

## Examples

See `examples/debug_example.rs` for a complete working example demonstrating all debugger features.

Run with:
```bash
cargo run --example debug_example
```

## Module Structure

- `mod.rs` - Main debugger interface
- `cpu.rs` - CPU debugger and state capture
- `ppu.rs` - PPU debugger and state capture
- `memory.rs` - Memory viewer and search
- `disassembler.rs` - Instruction disassembly
- `logger.rs` - Trace logging system

## Testing

Run the debug module tests:
```bash
cargo test --lib debug
```
