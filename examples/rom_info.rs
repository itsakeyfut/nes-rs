// Simple tool to display ROM information
use nes_rs::cartridge::Cartridge;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rom_path>", args[0]);
        std::process::exit(1);
    }

    let rom_path = &args[1];
    println!("Loading ROM: {}", rom_path);
    println!();

    let cartridge = Cartridge::from_ines_file(rom_path)?;

    println!("ROM Information:");
    println!("================");
    println!("Mapper:         {}", cartridge.mapper);
    println!("Mirroring:      {:?}", cartridge.mirroring);
    println!("PRG-ROM Size:   {} bytes ({} KB)", cartridge.prg_rom_size(), cartridge.prg_rom_size() / 1024);
    println!("CHR-ROM Size:   {} bytes ({} KB)", cartridge.chr_rom_size(), cartridge.chr_rom_size() / 1024);
    println!("Has Trainer:    {}", cartridge.has_trainer());
    println!("Has Battery:    {}", cartridge.has_battery);

    Ok(())
}
