// PPU Benchmarks
// Performance benchmarks for PPU rendering operations

use criterion::{criterion_group, criterion_main, Criterion};
use nes_rs::cartridge::mappers::Mapper0;
use nes_rs::{Cartridge, MemoryMappedDevice, Mirroring, Ppu};
use std::cell::RefCell;
use std::hint::black_box;
use std::rc::Rc;

/// Helper function to create a test cartridge
fn create_test_cartridge() -> Cartridge {
    let mut cart = Cartridge::new();
    cart.prg_rom = vec![0; 16 * 1024]; // 16KB PRG-ROM (minimum for Mapper0)
    cart.chr_rom = vec![0xAA; 8 * 1024]; // 8KB CHR-ROM with test pattern
    cart.mirroring = Mirroring::Horizontal;
    cart
}

/// Benchmark PPU step execution (cycle-by-cycle)
/// This is the main performance-critical path for the PPU
fn bench_ppu_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("ppu_rendering");
    group.sample_size(20); // Reduce sample size for rendering benchmarks

    // Benchmark a full frame of PPU steps
    // One frame = 262 scanlines * 341 cycles = 89,342 cycles
    group.bench_function("full_frame_via_step", |b| {
        let mut ppu = Ppu::new();

        // Create Mapper0 with test cartridge
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));

        ppu.set_mapper(mapper_rc);
        ppu.set_mirroring(Mirroring::Horizontal);

        // Enable rendering via MemoryMappedDevice trait
        ppu.write(0x2001, 0b00011110); // PPUMASK: show background and sprites

        b.iter(|| {
            // Run one full frame worth of PPU cycles
            for _ in 0..89342 {
                ppu.step();
            }
            black_box(ppu.frame());
        });
    });

    group.finish();
}

/// Benchmark PPU step execution at different granularities
fn bench_ppu_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("ppu_step");

    group.bench_function("single_step", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            black_box(ppu.step());
        });
    });

    group.bench_function("scanline_341_cycles", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            // One scanline = 341 PPU cycles
            for _ in 0..341 {
                ppu.step();
            }
        });
    });

    group.finish();
}

/// Benchmark PPU register access patterns
fn bench_ppu_registers(c: &mut Criterion) {
    let mut group = c.benchmark_group("ppu_registers");

    group.bench_function("ppuctrl_write", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            ppu.write(black_box(0x2000), black_box(0b10010000));
        });
    });

    group.bench_function("ppustatus_read", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            black_box(ppu.read(0x2002));
        });
    });

    group.bench_function("ppudata_write_sequence", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            // Set VRAM address
            ppu.write(0x2006, 0x20); // High byte
            ppu.write(0x2006, 0x00); // Low byte

            // Write 32 bytes
            for i in 0..32 {
                ppu.write(0x2007, i);
            }
        });
    });

    group.finish();
}

/// Benchmark OAM (Object Attribute Memory) access patterns
fn bench_ppu_oam(c: &mut Criterion) {
    let mut group = c.benchmark_group("ppu_oam");

    group.bench_function("oam_write", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            // Write full OAM (256 bytes) via OAMDATA register
            ppu.write(0x2003, 0); // Set OAM address to 0
            for i in 0..=255u8 {
                ppu.write(0x2004, i); // Write to OAMDATA
            }
        });
    });

    group.bench_function("oam_read", |b| {
        let mut ppu = Ppu::new();
        let cart = create_test_cartridge();
        let mapper = Mapper0::new(cart);
        let mapper_rc = Rc::new(RefCell::new(Box::new(mapper) as Box<dyn nes_rs::Mapper>));
        ppu.set_mapper(mapper_rc);

        b.iter(|| {
            // Read from OAMDATA register
            black_box(ppu.read(0x2004));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ppu_rendering,
    bench_ppu_step,
    bench_ppu_registers,
    bench_ppu_oam
);
criterion_main!(benches);
