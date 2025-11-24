// Memory Viewer Panel - Enhanced with color coding, special views, and search

use super::DebugUI;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::debug::{CpuMemoryRegionType, Debugger, MemoryRegion, MemoryViewer};
use crate::ppu::Ppu;

/// Memory viewer tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryViewerTab {
    CpuMemory = 0,
    ZeroPage = 1,
    Stack = 2,
    PpuVram = 3,
    Oam = 4,
}

impl From<usize> for MemoryViewerTab {
    fn from(value: usize) -> Self {
        match value {
            0 => MemoryViewerTab::CpuMemory,
            1 => MemoryViewerTab::ZeroPage,
            2 => MemoryViewerTab::Stack,
            3 => MemoryViewerTab::PpuVram,
            4 => MemoryViewerTab::Oam,
            _ => MemoryViewerTab::CpuMemory,
        }
    }
}

/// Show the memory viewer panel
pub(super) fn show(
    ui_state: &mut DebugUI,
    ctx: &egui::Context,
    debugger: &Debugger,
    bus: &mut Bus,
    cpu: &Cpu,
    ppu: &Ppu,
) {
    let mut is_open = ui_state.show_memory_panel;

    egui::Window::new("Memory Viewer")
        .open(&mut is_open)
        .default_width(800.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            let current_tab: MemoryViewerTab = ui_state.memory_tab.into();

            // Tab selection
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(current_tab == MemoryViewerTab::CpuMemory, "CPU Memory")
                    .clicked()
                {
                    ui_state.memory_tab = MemoryViewerTab::CpuMemory as usize;
                }
                if ui
                    .selectable_label(current_tab == MemoryViewerTab::ZeroPage, "Zero Page")
                    .clicked()
                {
                    ui_state.memory_tab = MemoryViewerTab::ZeroPage as usize;
                }
                if ui
                    .selectable_label(current_tab == MemoryViewerTab::Stack, "Stack")
                    .clicked()
                {
                    ui_state.memory_tab = MemoryViewerTab::Stack as usize;
                }
                if ui
                    .selectable_label(current_tab == MemoryViewerTab::PpuVram, "PPU VRAM")
                    .clicked()
                {
                    ui_state.memory_tab = MemoryViewerTab::PpuVram as usize;
                }
                if ui
                    .selectable_label(current_tab == MemoryViewerTab::Oam, "OAM")
                    .clicked()
                {
                    ui_state.memory_tab = MemoryViewerTab::Oam as usize;
                }
            });

            ui.separator();

            // Render the selected tab
            match current_tab {
                MemoryViewerTab::CpuMemory => show_cpu_memory_tab(ui, ui_state, debugger, bus, cpu),
                MemoryViewerTab::ZeroPage => show_zero_page_tab(ui, debugger, bus),
                MemoryViewerTab::Stack => show_stack_tab(ui, ui_state, debugger, bus, cpu),
                MemoryViewerTab::PpuVram => show_ppu_vram_tab(ui, ui_state, debugger, ppu),
                MemoryViewerTab::Oam => show_oam_tab(ui, debugger, ppu),
            }
        });

    ui_state.show_memory_panel = is_open;
}

/// Show CPU memory viewer tab
fn show_cpu_memory_tab(
    ui: &mut egui::Ui,
    ui_state: &mut DebugUI,
    debugger: &Debugger,
    bus: &mut Bus,
    cpu: &Cpu,
) {
    // Controls
    ui.horizontal(|ui| {
        ui.label("Address:");
        if ui
            .text_edit_singleline(&mut ui_state.cpu_mem_address)
            .lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            // Jump to address on Enter
            if let Ok(addr) = u16::from_str_radix(&ui_state.cpu_mem_address, 16) {
                ui_state.cpu_mem_address = format!("{:04X}", addr);
            }
        }

        ui.label("Bytes:");
        ui.add(egui::DragValue::new(&mut ui_state.cpu_mem_bytes).range(16..=4096));

        ui.checkbox(&mut ui_state.follow_pc, "Follow PC");

        if ui.button("Jump to PC").clicked() {
            ui_state.cpu_mem_address = format!("{:04X}", cpu.pc);
            ui_state.follow_pc = false;
        }
    });

    // Search functionality
    ui.horizontal(|ui| {
        ui.label("Search (hex):");
        if ui
            .text_edit_singleline(&mut ui_state.search_pattern)
            .lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            // Parse search pattern and search
            if let Some(pattern) = parse_hex_pattern(&ui_state.search_pattern) {
                let start_addr = u16::from_str_radix(&ui_state.cpu_mem_address, 16).unwrap_or(0);
                ui_state.search_results = debugger
                    .memory
                    .search_cpu_memory(bus, &pattern, start_addr, 0xFFFF);
                ui_state.search_result_index = 0;
            }
        }

        if !ui_state.search_results.is_empty() {
            ui.label(format!(
                "Results: {} ({}/{})",
                ui_state.search_results.len(),
                ui_state.search_result_index + 1,
                ui_state.search_results.len()
            ));

            if ui.button("Prev").clicked() && ui_state.search_result_index > 0 {
                ui_state.search_result_index -= 1;
                ui_state.cpu_mem_address = format!(
                    "{:04X}",
                    ui_state.search_results[ui_state.search_result_index]
                );
            }

            if ui.button("Next").clicked()
                && ui_state.search_result_index < ui_state.search_results.len() - 1
            {
                ui_state.search_result_index += 1;
                ui_state.cpu_mem_address = format!(
                    "{:04X}",
                    ui_state.search_results[ui_state.search_result_index]
                );
            }
        }
    });

    ui.separator();

    // Quick access buttons
    ui.horizontal(|ui| {
        if ui.button("Zero Page ($0000)").clicked() {
            ui_state.cpu_mem_address = String::from("0000");
            ui_state.cpu_mem_bytes = 256;
            ui_state.follow_pc = false;
        }
        if ui.button("Stack ($0100)").clicked() {
            ui_state.cpu_mem_address = String::from("0100");
            ui_state.cpu_mem_bytes = 256;
            ui_state.follow_pc = false;
        }
        if ui.button("PPU Regs ($2000)").clicked() {
            ui_state.cpu_mem_address = String::from("2000");
            ui_state.cpu_mem_bytes = 64;
            ui_state.follow_pc = false;
        }
        if ui.button("ROM ($8000)").clicked() {
            ui_state.cpu_mem_address = String::from("8000");
            ui_state.cpu_mem_bytes = 512;
            ui_state.follow_pc = false;
        }
    });

    ui.separator();

    // Color legend
    ui.horizontal(|ui| {
        ui.label("Legend:");
        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "■ RAM");
        ui.colored_label(egui::Color32::from_rgb(255, 220, 100), "■ Stack");
        ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "■ PPU Registers");
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "■ Cartridge");
        ui.colored_label(
            egui::Color32::from_rgb(255, 100, 255),
            "■ Recently Modified",
        );
    });

    ui.separator();

    // Follow PC mode
    if ui_state.follow_pc {
        ui_state.cpu_mem_address = format!("{:04X}", cpu.pc.saturating_sub(32));
    }

    // Memory dump with color coding
    if let Ok(addr) = u16::from_str_radix(&ui_state.cpu_mem_address, 16) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_cpu_memory_hex_dump(ui, debugger, bus, cpu, addr, ui_state.cpu_mem_bytes);
            });
    } else {
        ui.label("Invalid address");
    }
}

/// Render CPU memory hex dump with color coding
fn render_cpu_memory_hex_dump(
    ui: &mut egui::Ui,
    debugger: &Debugger,
    bus: &mut Bus,
    cpu: &Cpu,
    start: u16,
    length: usize,
) {
    let bytes_per_row = 16;
    let rows = length.div_ceil(bytes_per_row);

    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

    for row in 0..rows {
        let addr = start.wrapping_add((row * bytes_per_row) as u16);

        ui.horizontal(|ui| {
            // Address with highlighting for PC
            if cpu.pc >= addr && cpu.pc < addr + bytes_per_row as u16 {
                ui.colored_label(egui::Color32::YELLOW, format!("${:04X}:", addr));
            } else {
                ui.colored_label(egui::Color32::GRAY, format!("${:04X}:", addr));
            }

            // Region label
            let region_type = MemoryViewer::get_cpu_region_type(addr);
            let region_label = match region_type {
                CpuMemoryRegionType::Ram => "RAM",
                CpuMemoryRegionType::Stack => "STK",
                CpuMemoryRegionType::PpuRegisters => "PPU",
                CpuMemoryRegionType::ApuIo => "I/O",
                CpuMemoryRegionType::Cartridge => "ROM",
                CpuMemoryRegionType::Other => "???",
            };
            ui.colored_label(egui::Color32::DARK_GRAY, format!("[{:3}]", region_label));

            // Hex bytes with color coding
            for col in 0..bytes_per_row {
                let offset = row * bytes_per_row + col;
                if offset < length {
                    let byte_addr = start.wrapping_add(offset as u16);
                    let byte = bus.read(byte_addr);
                    let region_type = MemoryViewer::get_cpu_region_type(byte_addr);

                    // Determine color based on region and modification status
                    let color = if debugger.memory.is_recently_modified(byte_addr) {
                        egui::Color32::from_rgb(255, 100, 255) // Magenta for recently modified
                    } else {
                        match region_type {
                            CpuMemoryRegionType::Ram => egui::Color32::from_rgb(100, 200, 100),
                            CpuMemoryRegionType::Stack => egui::Color32::from_rgb(255, 220, 100),
                            CpuMemoryRegionType::PpuRegisters => {
                                egui::Color32::from_rgb(100, 150, 255)
                            }
                            CpuMemoryRegionType::ApuIo => egui::Color32::from_rgb(150, 150, 150),
                            CpuMemoryRegionType::Cartridge => {
                                egui::Color32::from_rgb(255, 100, 100)
                            }
                            CpuMemoryRegionType::Other => egui::Color32::WHITE,
                        }
                    };

                    ui.colored_label(color, format!("{:02X}", byte));
                } else {
                    ui.label("  ");
                }
            }

            ui.label("|");

            // ASCII representation
            for col in 0..bytes_per_row {
                let offset = row * bytes_per_row + col;
                if offset < length {
                    let byte_addr = start.wrapping_add(offset as u16);
                    let byte = bus.read(byte_addr);
                    let ch = if (0x20..=0x7E).contains(&byte) {
                        byte as char
                    } else {
                        '.'
                    };
                    ui.colored_label(egui::Color32::GRAY, ch.to_string());
                } else {
                    ui.label(" ");
                }
            }
        });
    }
}

/// Show zero page viewer tab
fn show_zero_page_tab(ui: &mut egui::Ui, debugger: &Debugger, bus: &mut Bus) {
    ui.heading("Zero Page ($0000-$00FF)");
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            let dump = debugger.memory.dump_zero_page(bus);
            ui.label(dump);
        });
}

/// Show stack viewer tab
fn show_stack_tab(
    ui: &mut egui::Ui,
    _ui_state: &mut DebugUI,
    _debugger: &Debugger,
    bus: &mut Bus,
    cpu: &Cpu,
) {
    ui.heading("Stack ($0100-$01FF)");
    ui.horizontal(|ui| {
        ui.label(format!("Stack Pointer: ${:02X}", cpu.sp));
        ui.label(format!("Stack Top: ${:04X}", 0x0100 | cpu.sp as u16));
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

            // Show stack with SP indicator
            let bytes_per_row = 16;
            for row in 0..16 {
                let addr = 0x0100 + (row * bytes_per_row);

                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::GRAY, format!("${:04X}:", addr));

                    for col in 0..bytes_per_row {
                        let byte_addr = addr + col;
                        let byte = bus.read(byte_addr as u16);

                        // Highlight stack pointer position
                        let color = if byte_addr == (0x0100 | cpu.sp as u16) as usize {
                            egui::Color32::YELLOW
                        } else if byte_addr > (0x0100 | cpu.sp as u16) as usize {
                            egui::Color32::from_rgb(255, 220, 100) // Active stack area
                        } else {
                            egui::Color32::from_rgb(100, 100, 100) // Unused stack area
                        };

                        ui.colored_label(color, format!("{:02X}", byte));
                    }

                    ui.label("|");

                    // ASCII
                    for col in 0..bytes_per_row {
                        let byte = bus.read((addr + col) as u16);
                        let ch = if (0x20..=0x7E).contains(&byte) {
                            byte as char
                        } else {
                            '.'
                        };
                        ui.colored_label(egui::Color32::GRAY, ch.to_string());
                    }
                });
            }
        });
}

/// Show PPU VRAM viewer tab
fn show_ppu_vram_tab(ui: &mut egui::Ui, ui_state: &mut DebugUI, debugger: &Debugger, ppu: &Ppu) {
    let ppu_region: MemoryRegion = match ui_state.ppu_mem_region {
        0 => MemoryRegion::PpuNametables,
        1 => MemoryRegion::PpuPalette,
        _ => MemoryRegion::PpuNametables,
    };

    ui.horizontal(|ui| {
        ui.label("Region:");
        if ui
            .selectable_label(ppu_region == MemoryRegion::PpuNametables, "Nametables")
            .clicked()
        {
            ui_state.ppu_mem_region = 0;
        }
        if ui
            .selectable_label(ppu_region == MemoryRegion::PpuPalette, "Palette")
            .clicked()
        {
            ui_state.ppu_mem_region = 1;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Offset:");
        ui.text_edit_singleline(&mut ui_state.ppu_mem_address);

        ui.label("Bytes:");
        ui.add(egui::DragValue::new(&mut ui_state.ppu_mem_bytes).range(16..=4096));
    });

    ui.separator();

    // Quick access buttons
    ui.horizontal(|ui| {
        if ui.button("Nametable 0 ($2000)").clicked() {
            ui_state.ppu_mem_region = 0;
            ui_state.ppu_mem_address = String::from("0");
            ui_state.ppu_mem_bytes = 1024;
        }
        if ui.button("Nametable 1 ($2400)").clicked() {
            ui_state.ppu_mem_region = 0;
            ui_state.ppu_mem_address = String::from("400");
            ui_state.ppu_mem_bytes = 1024;
        }
        if ui.button("Palette ($3F00)").clicked() {
            ui_state.ppu_mem_region = 1;
            ui_state.ppu_mem_address = String::from("0");
            ui_state.ppu_mem_bytes = 32;
        }
    });

    ui.separator();

    if let Ok(offset) = usize::from_str_radix(&ui_state.ppu_mem_address, 16) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                let dump = debugger.memory.dump_ppu_memory(
                    ppu,
                    ppu_region,
                    offset,
                    ui_state.ppu_mem_bytes,
                );
                ui.label(dump);
            });
    } else {
        ui.label("Invalid offset");
    }
}

/// Show OAM viewer tab
fn show_oam_tab(ui: &mut egui::Ui, debugger: &Debugger, ppu: &Ppu) {
    ui.heading("OAM (Sprite Memory)");
    ui.label("64 sprites × 4 bytes each (Y, Tile, Attributes, X)");
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            let dump = debugger.memory.dump_oam(ppu);
            ui.label(dump);
        });
}

/// Parse hex pattern from string (e.g., "DE AD BE EF" -> [0xDE, 0xAD, 0xBE, 0xEF])
fn parse_hex_pattern(pattern: &str) -> Option<Vec<u8>> {
    let tokens: Vec<&str> = pattern.split_whitespace().collect();
    let mut bytes = Vec::new();

    for token in tokens {
        if let Ok(byte) = u8::from_str_radix(token, 16) {
            bytes.push(byte);
        } else {
            return None;
        }
    }

    if bytes.is_empty() {
        None
    } else {
        Some(bytes)
    }
}
