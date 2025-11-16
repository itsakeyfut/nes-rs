// Debug Example - Demonstrates the debugging features
//
// This example shows how to use the NES emulator debugger to:
// - Set breakpoints
// - Step through instructions
// - View CPU and PPU state
// - Dump memory
// - Trace execution

use nes_rs::{Bus, Cpu, Debugger, LogLevel, Ppu};

fn main() {
    println!("NES Emulator Debug Example");
    println!("==========================\n");

    // Create emulator components
    let mut cpu = Cpu::new();
    let ppu = Ppu::new();
    let mut bus = Bus::new();

    // Create and configure debugger
    let mut debugger = Debugger::new();
    debugger.enable();

    // Set up a simple test program
    setup_test_program(&mut bus);

    // Reset CPU to load PC from reset vector
    cpu.reset(&mut bus);

    println!("Initial CPU State:");
    println!("{}\n", debugger.cpu.dump_registers(&cpu));

    // Example 1: Set a breakpoint
    println!("Example 1: Setting a breakpoint at $8005");
    debugger.add_breakpoint(0x8005);
    println!("Breakpoints: {:?}\n", debugger.breakpoints());

    // Example 2: Step through instructions
    println!("Example 2: Step through first 5 instructions");
    for i in 0..5 {
        let state = debugger.get_cpu_state(&cpu, &mut bus);
        println!("Step {}: {}", i + 1, state);

        // Execute one instruction (simplified - normally would call cpu.step())
        // For this example, we just increment PC
        cpu.pc = cpu.pc.wrapping_add(state.instruction.length as u16);
    }
    println!();

    // Example 3: Memory dump
    println!("Example 3: Memory dump of test program");
    let mem_dump = debugger.memory.dump_cpu_memory(&mut bus, 0x8000, 64);
    println!("{}", mem_dump);

    // Example 4: Disassembly
    println!("Example 4: Disassemble program");
    use nes_rs::debug::disassemble_count;
    let instructions = disassemble_count(0x8000, 10, &mut bus);
    for instr in instructions {
        println!("{}", instr);
    }
    println!();

    // Example 5: PPU state
    println!("Example 5: PPU state");
    let ppu_state = debugger.get_ppu_state(&ppu);
    println!("{}", ppu_state.format());

    // Example 6: Logging
    println!("Example 6: Enable trace logging");
    debugger.logger.enable_cpu_trace();
    debugger.logger.set_log_level(LogLevel::Trace);

    // Simulate a few instruction traces
    for i in 0..3 {
        cpu.pc = 0x8000 + i;
        let state = debugger.get_cpu_state(&cpu, &mut bus);
        debugger.logger.log_cpu_state(&state);
    }

    println!(
        "Trace buffer ({} entries):",
        debugger.logger.trace_buffer().len()
    );
    println!("{}", debugger.logger.format_trace_buffer());

    // Example 7: Stack dump
    println!("Example 7: Stack dump");
    println!("{}", debugger.cpu.dump_stack(&cpu, &mut bus));

    // Example 8: Palette viewer
    println!("Example 8: PPU Palettes");
    println!("{}", debugger.ppu.format_palettes(&ppu));

    println!("\nDebug example complete!");
}

/// Set up a simple test program in memory
fn setup_test_program(bus: &mut Bus) {
    // Reset vector at $FFFC-$FFFD points to $8000
    bus.write(0xFFFC, 0x00);
    bus.write(0xFFFD, 0x80);

    // Simple test program at $8000
    let program: Vec<u8> = vec![
        0xA9, 0x42, // LDA #$42
        0x85, 0x10, // STA $10
        0xA2, 0x05, // LDX #$05
        0xA0, 0x0A, // LDY #$0A
        0x18, // CLC
        0x69, 0x10, // ADC #$10
        0x4C, 0x00, 0x80, // JMP $8000 (loop)
    ];

    for (i, &byte) in program.iter().enumerate() {
        bus.write(0x8000 + i as u16, byte);
    }
}
