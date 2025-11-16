// CPU Benchmarks
// Performance benchmarks for CPU instruction execution

use criterion::{criterion_group, criterion_main, Criterion};
use nes_rs::{Bus, Cpu};
use std::hint::black_box;

/// Benchmark CPU instruction execution
/// Tests various common instruction patterns to measure dispatch and execution performance
fn bench_cpu_instructions(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_instructions");

    // Benchmark NOP instruction (simplest operation)
    group.bench_function("nop", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up a simple program with NOPs
        // NOP = 0xEA (2 cycles each)
        for i in 0..256 {
            bus.write(i, 0xEA); // NOP
        }
        cpu.reset(&mut bus);

        b.iter(|| {
            cpu.step(black_box(&mut bus));
        });
    });

    // Benchmark LDA immediate (common load operation)
    group.bench_function("lda_immediate", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // LDA #$42 (0xA9 0x42)
        for i in (0..256).step_by(2) {
            bus.write(i, 0xA9); // LDA immediate
            bus.write(i + 1, 0x42); // Value
        }
        cpu.reset(&mut bus);

        b.iter(|| {
            cpu.step(black_box(&mut bus));
        });
    });

    // Benchmark ADC immediate (arithmetic operation)
    group.bench_function("adc_immediate", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // ADC #$01 (0x69 0x01)
        for i in (0..256).step_by(2) {
            bus.write(i, 0x69); // ADC immediate
            bus.write(i + 1, 0x01); // Value
        }
        cpu.reset(&mut bus);

        b.iter(|| {
            cpu.step(black_box(&mut bus));
        });
    });

    // Benchmark STA absolute (memory write operation)
    group.bench_function("sta_absolute", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // STA $0200 (0x8D 0x00 0x02)
        for i in (0..256).step_by(3) {
            if i + 2 < 256 {
                bus.write(i, 0x8D); // STA absolute
                bus.write(i + 1, 0x00); // Low byte
                bus.write(i + 2, 0x02); // High byte
            }
        }
        cpu.reset(&mut bus);

        b.iter(|| {
            cpu.step(black_box(&mut bus));
        });
    });

    // Benchmark JMP absolute (control flow)
    group.bench_function("jmp_absolute", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // JMP $C000 (0x4C 0x00 0xC0)
        // Create a loop that jumps back to itself
        bus.write(0xC000, 0x4C); // JMP absolute
        bus.write(0xC001, 0x00); // Low byte
        bus.write(0xC002, 0xC0); // High byte

        cpu.reset(&mut bus);
        cpu.pc = 0xC000;

        b.iter(|| {
            cpu.step(black_box(&mut bus));
        });
    });

    group.finish();
}

/// Benchmark a sequence of mixed instructions (realistic workload)
fn bench_instruction_sequence(c: &mut Criterion) {
    let mut group = c.benchmark_group("instruction_sequences");

    group.bench_function("typical_sequence", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Set up a typical instruction sequence
        let mut addr = 0xC000u16;

        // LDA #$00
        bus.write(addr, 0xA9);
        addr += 1;
        bus.write(addr, 0x00);
        addr += 1;

        // STA $0200
        bus.write(addr, 0x8D);
        addr += 1;
        bus.write(addr, 0x00);
        addr += 1;
        bus.write(addr, 0x02);
        addr += 1;

        // LDX #$05
        bus.write(addr, 0xA2);
        addr += 1;
        bus.write(addr, 0x05);
        addr += 1;

        // INX
        bus.write(addr, 0xE8);
        addr += 1;

        // DEX
        bus.write(addr, 0xCA);
        addr += 1;

        // BNE back to LDA
        bus.write(addr, 0xD0);
        addr += 1;
        bus.write(addr, 0xF6); // -10 bytes

        cpu.reset(&mut bus);
        cpu.pc = 0xC000;

        b.iter(|| {
            // Execute one full iteration (11 instructions)
            for _ in 0..11 {
                cpu.step(black_box(&mut bus));
            }
        });
    });

    group.finish();
}

/// Benchmark CPU execution over multiple frames
/// Simulates realistic emulator workload
fn bench_frame_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_execution");
    group.sample_size(20); // Reduce sample size for longer benchmarks

    group.bench_function("1000_cycles", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Fill memory with NOPs
        for i in 0..=0xFFFF {
            bus.write(i, 0xEA); // NOP
        }

        cpu.reset(&mut bus);

        b.iter(|| {
            let start_cycles = cpu.cycles;
            while cpu.cycles - start_cycles < 1000 {
                cpu.step(black_box(&mut bus));
            }
        });
    });

    group.bench_function("29780_cycles_one_frame", |b| {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        // Fill memory with NOPs
        for i in 0..=0xFFFF {
            bus.write(i, 0xEA); // NOP
        }

        cpu.reset(&mut bus);

        b.iter(|| {
            // NES CPU runs at ~1.789773 MHz
            // At 60 FPS: ~29,780 cycles per frame
            let start_cycles = cpu.cycles;
            while cpu.cycles - start_cycles < 29780 {
                cpu.step(black_box(&mut bus));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cpu_instructions,
    bench_instruction_sequence,
    bench_frame_execution
);
criterion_main!(benches);
