// Interactive Debug ROM Example
//
// This example demonstrates the full debug UI with ROM loading and interactive
// debugging capabilities. It renders NES output and shows debug information
// via terminal when stepping through instructions.
//
// Note: Due to wgpu version incompatibility between pixels (0.19) and egui-wgpu (27),
// the egui overlay is not rendered directly on the NES window. Debug information
// is displayed in the terminal instead. For full egui integration, ensure
// compatible versions of pixels and egui-wgpu are used.
//
// Usage:
//   cargo run --example debug_rom -- "path/to/rom.nes"
//
// Keyboard Shortcuts:
//   F5:  Play/Pause
//   F6:  Step instruction
//   F7:  Step frame
//   F8:  Reset
//   F9:  Toggle CPU info display
//   F10: Toggle PPU info display
//   F11: Toggle memory dump display
//   F12: Toggle execution log
//   D:   Dump current state to terminal
//   Escape: Exit

use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use nes_rs::cartridge::Cartridge;
use nes_rs::debug::{disassemble_count, Debugger, StepMode};
use nes_rs::display::{copy_ppu_to_display, FrameBuffer, SCREEN_HEIGHT, SCREEN_WIDTH};
use nes_rs::{Bus, Cpu, Ppu};

/// NES screen dimensions
const NES_WIDTH: u32 = SCREEN_WIDTH as u32;
const NES_HEIGHT: u32 = SCREEN_HEIGHT as u32;

/// Window configuration
const WINDOW_SCALE: u32 = 3;
const WINDOW_WIDTH: u32 = NES_WIDTH * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = NES_HEIGHT * WINDOW_SCALE;

/// Display options
#[derive(Default)]
struct DisplayOptions {
    show_cpu_info: bool,
    show_ppu_info: bool,
    show_memory_dump: bool,
    show_execution_log: bool,
}

/// Application state
struct DebugRomApp {
    /// Window handle
    window: Option<Arc<Window>>,
    /// Pixels buffer for NES output
    pixels: Option<Pixels<'static>>,

    // Emulator components
    cpu: Cpu,
    ppu: Ppu,
    bus: Bus,
    cartridge: Option<Cartridge>,

    // Debug components
    debugger: Debugger,

    // Display
    frame_buffer: FrameBuffer,

    // State
    rom_path: Option<String>,
    last_frame_time: Instant,
    running: bool,

    // Display options
    display_options: DisplayOptions,
}

impl DebugRomApp {
    fn new(rom_path: Option<String>) -> Self {
        let mut app = Self {
            window: None,
            pixels: None,

            cpu: Cpu::new(),
            ppu: Ppu::new(),
            bus: Bus::new(),
            cartridge: None,

            debugger: Debugger::new(),

            frame_buffer: FrameBuffer::new(),

            rom_path,
            last_frame_time: Instant::now(),
            running: false,

            display_options: DisplayOptions {
                show_cpu_info: true,
                show_ppu_info: false,
                show_memory_dump: false,
                show_execution_log: false,
            },
        };

        // Enable debugger by default
        app.debugger.enable();
        // Start paused
        app.debugger.pause();

        app
    }

    /// Load ROM from path
    fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cartridge = Cartridge::from_ines_file(path)?;

        // Load PRG-ROM into bus
        if !cartridge.prg_rom.is_empty() {
            self.bus.load_rom(&cartridge.prg_rom, 0x3FE0);
        }

        self.cartridge = Some(cartridge);
        self.rom_path = Some(path.to_string());

        // Reset CPU
        self.cpu.reset(&mut self.bus);

        self.print_rom_info();

        Ok(())
    }

    /// Print ROM information
    fn print_rom_info(&self) {
        println!("\n=== ROM Information ===");
        if let Some(ref path) = self.rom_path {
            println!("File: {}", path);
        }
        if let Some(ref cart) = self.cartridge {
            println!("Mapper: {}", cart.mapper);
            println!("PRG-ROM: {} KB", cart.prg_rom_size() / 1024);
            println!("CHR-ROM: {} KB", cart.chr_rom_size() / 1024);
            println!("Mirroring: {:?}", cart.mirroring);
            println!("Battery: {}", if cart.has_battery { "Yes" } else { "No" });
            println!("Trainer: {}", if cart.has_trainer() { "Yes" } else { "No" });
        }
        println!("========================\n");
    }

    /// Step the emulator
    fn step_emulator(&mut self) {
        if self.debugger.is_paused() && self.debugger.step_mode() == StepMode::None {
            return;
        }

        // Execute CPU instruction
        if self.debugger.before_instruction(&self.cpu, &mut self.bus) {
            let _cycles = self.cpu.step(&mut self.bus);

            // Step PPU (3 PPU cycles per CPU cycle)
            for _ in 0..3 {
                self.ppu.step();
                self.debugger.after_ppu_step(&self.ppu);
            }
        }
    }

    /// Run one frame of emulation
    fn run_frame(&mut self) {
        // Track frame start
        self.debugger.on_frame_start(&self.cpu);

        // Run until frame is complete (approximately 29780 CPU cycles per frame)
        let frame_cycles = 29780u64;
        let start_cycles = self.cpu.cycles;

        while self.cpu.cycles - start_cycles < frame_cycles {
            if self.debugger.is_paused() && self.debugger.step_mode() == StepMode::None {
                break;
            }
            self.step_emulator();
        }

        // Render PPU frame
        self.ppu.render_frame();

        // Track frame end
        self.debugger.on_frame_end(&self.cpu);
    }

    /// Update display buffer from PPU
    fn update_display(&mut self) {
        copy_ppu_to_display(self.ppu.frame(), &mut self.frame_buffer);
    }

    /// Print CPU state
    fn print_cpu_state(&mut self) {
        let state = self.debugger.cpu.capture_state(&self.cpu, &mut self.bus);
        println!("\n=== CPU State ===");
        println!("{}", state);
        println!("=================\n");
    }

    /// Print PPU state
    fn print_ppu_state(&self) {
        let state = self.debugger.ppu.capture_state(&self.ppu);
        println!("\n=== PPU State ===");
        println!("{}", state.format());
        println!("=================\n");
    }

    /// Print memory dump
    fn print_memory_dump(&mut self) {
        println!("\n=== Memory Dump (PC area) ===");
        let start = self.cpu.pc.saturating_sub(16);
        let dump = self
            .debugger
            .memory
            .dump_cpu_memory(&mut self.bus, start, 64);
        println!("{}", dump);
        println!("==============================\n");
    }

    /// Print disassembly
    fn print_disassembly(&mut self) {
        println!("\n=== Disassembly ===");
        let instructions = disassemble_count(self.cpu.pc, 10, &mut self.bus);
        for instr in instructions {
            let marker = if instr.address == self.cpu.pc {
                ">>>"
            } else {
                "   "
            };
            println!("{} {}", marker, instr);
        }
        println!("===================\n");
    }

    /// Print current state
    fn print_current_state(&mut self) {
        println!("\n{}", "=".repeat(50));
        println!(
            "Debugger Status: {}",
            if self.debugger.is_paused() {
                "PAUSED"
            } else {
                "RUNNING"
            }
        );
        println!("Total Frames: {}", self.debugger.metrics.total_frames);
        println!(
            "Total Instructions: {}",
            self.debugger.metrics.total_instructions
        );
        println!("FPS: {:.2}", self.debugger.metrics.fps);

        if self.display_options.show_cpu_info {
            self.print_cpu_state();
        }

        if self.display_options.show_ppu_info {
            self.print_ppu_state();
        }

        if self.display_options.show_memory_dump {
            self.print_memory_dump();
        }

        self.print_disassembly();
    }

    /// Handle keyboard shortcuts
    fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        if !pressed {
            return;
        }

        match key {
            KeyCode::F5 => {
                // Play/Pause
                if self.debugger.is_paused() {
                    self.debugger.resume();
                    self.running = true;
                    println!(">>> RESUMED");
                } else {
                    self.debugger.pause();
                    self.running = false;
                    println!(">>> PAUSED");
                    self.print_current_state();
                }
            }
            KeyCode::F6 => {
                // Step instruction
                println!(">>> Step Instruction");
                self.debugger.step_instruction();
                self.step_emulator();
                self.print_current_state();
            }
            KeyCode::F7 => {
                // Step frame
                println!(">>> Step Frame");
                self.debugger.step_frame();
                self.run_frame();
                self.print_current_state();
            }
            KeyCode::F8 => {
                // Reset
                println!(">>> RESET");
                self.cpu.reset(&mut self.bus);
                self.debugger.metrics.reset();
                self.print_current_state();
            }
            KeyCode::F9 => {
                // Toggle CPU info display
                self.display_options.show_cpu_info = !self.display_options.show_cpu_info;
                println!(
                    ">>> CPU info display: {}",
                    if self.display_options.show_cpu_info {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::F10 => {
                // Toggle PPU info display
                self.display_options.show_ppu_info = !self.display_options.show_ppu_info;
                println!(
                    ">>> PPU info display: {}",
                    if self.display_options.show_ppu_info {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::F11 => {
                // Toggle memory dump display
                self.display_options.show_memory_dump = !self.display_options.show_memory_dump;
                println!(
                    ">>> Memory dump display: {}",
                    if self.display_options.show_memory_dump {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::F12 => {
                // Toggle execution log
                self.display_options.show_execution_log = !self.display_options.show_execution_log;
                if self.display_options.show_execution_log {
                    self.debugger.execution_log.enable_instruction_logging();
                } else {
                    self.debugger.execution_log.disable_instruction_logging();
                }
                println!(
                    ">>> Execution log: {}",
                    if self.display_options.show_execution_log {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::KeyD => {
                // Dump current state
                self.print_current_state();
            }
            KeyCode::Escape => {
                // Exit is handled by the event loop
            }
            _ => {}
        }
    }

    /// Render NES output to pixels buffer
    fn render_nes(&mut self) -> Result<(), pixels::Error> {
        if let Some(pixels) = &mut self.pixels {
            let frame = pixels.frame_mut();
            self.frame_buffer.to_rgba(frame);
            pixels.render()?;
        }
        Ok(())
    }

    /// Check if it's time for next frame (for frame limiting)
    fn should_render_frame(&mut self) -> bool {
        let elapsed = self.last_frame_time.elapsed();
        let frame_duration = Duration::from_micros(16667); // ~60 FPS

        if elapsed >= frame_duration {
            self.last_frame_time = Instant::now();
            true
        } else {
            false
        }
    }
}

impl ApplicationHandler for DebugRomApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let title = if let Some(ref path) = self.rom_path {
            format!(
                "NES Debugger - {}",
                std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
            )
        } else {
            "NES Debugger - No ROM".to_string()
        };

        let window_attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_resizable(false);

        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        let window = Arc::new(window);
        let window_size = window.inner_size();

        // Create pixels buffer
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());

        let pixels = Pixels::new(NES_WIDTH, NES_HEIGHT, surface_texture)
            .expect("Failed to create pixel buffer");

        self.window = Some(window);
        self.pixels = Some(pixels);

        // Load ROM if path was provided
        if let Some(ref path) = self.rom_path.clone() {
            if let Err(e) = self.load_rom(path) {
                eprintln!("Failed to load ROM: {}", e);
            }
        }

        // Generate test pattern if no ROM loaded
        if self.cartridge.is_none() {
            println!("No ROM loaded, displaying test pattern.");
            self.frame_buffer.test_pattern();
        }

        // Print initial state
        println!("\n>>> Initial State");
        self.print_current_state();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\nClose requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                if key == KeyCode::Escape && state == ElementState::Pressed {
                    event_loop.exit();
                }
                self.handle_key(key, state == ElementState::Pressed);
            }
            WindowEvent::Resized(new_size) => {
                if let Some(ref mut pixels) = self.pixels {
                    if pixels
                        .resize_surface(new_size.width, new_size.height)
                        .is_err()
                    {
                        eprintln!("Failed to resize surface");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Run emulation if not paused
                if self.running && !self.debugger.is_paused() {
                    // Run some instructions
                    for _ in 0..1000 {
                        self.step_emulator();
                        if self.debugger.is_paused() {
                            self.running = false;
                            println!(">>> Hit breakpoint or pause condition");
                            self.print_current_state();
                            break;
                        }
                    }
                }

                // Update display from PPU
                self.update_display();

                // Render NES output
                if self.should_render_frame() {
                    if let Err(e) = self.render_nes() {
                        eprintln!("Render error: {}", e);
                        event_loop.exit();
                        return;
                    }
                }

                // Request next frame
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request a redraw
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NES Emulator - Interactive Debug ROM Viewer");
    println!("============================================");
    println!();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let rom_path = args.get(1).cloned();

    if rom_path.is_none() {
        println!("Usage: cargo run --example debug_rom -- <rom.nes>");
        println!();
        println!("No ROM specified, starting with test pattern.");
    } else {
        println!("ROM: {}", rom_path.as_ref().unwrap());
    }
    println!();

    println!("Keyboard Shortcuts:");
    println!("  F5:  Play/Pause");
    println!("  F6:  Step instruction");
    println!("  F7:  Step frame");
    println!("  F8:  Reset");
    println!("  F9:  Toggle CPU info display");
    println!("  F10: Toggle PPU info display");
    println!("  F11: Toggle memory dump display");
    println!("  F12: Toggle execution log");
    println!("  D:   Dump current state to terminal");
    println!("  Escape: Exit");
    println!();

    // Create event loop
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create application
    let mut app = DebugRomApp::new(rom_path);

    // Run event loop
    event_loop.run_app(&mut app)?;

    println!("Application closed.");
    Ok(())
}
