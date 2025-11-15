# Gamepad Support Setup

This document explains how to set up gamepad support for the NES emulator.

## System Requirements

### Linux (including WSL)

Gamepad support on Linux requires the `libudev` development library:

```bash
# Ubuntu/Debian
sudo apt-get install libudev-dev

# Fedora/RHEL
sudo dnf install systemd-devel

# Arch Linux
sudo pacman -S systemd
```

### macOS

No additional dependencies required. Gamepad support works out of the box.

### Windows

No additional dependencies required. Gamepad support works out of the box.

## Input Configuration

The emulator supports both keyboard and gamepad inputs simultaneously.

### Default Keyboard Mappings

**Player 1:**
- Arrow Keys: D-pad
- X: A button
- Z: B button
- Enter: Start
- Right Shift: Select

**Player 2:**
- WASD: D-pad
- K: A button
- J: B button
- Y: Start
- U: Select

### Default Gamepad Mappings

- D-pad/Left Stick: NES D-pad
- South button (A/Cross): B button
- East button (B/Circle): A button
- Start: Start
- Select/Back: Select

## Configuration File

On first run, the emulator creates an `input_config.toml` file with default mappings. You can edit this file to customize keyboard and gamepad bindings for both players.

### Example Configuration

```toml
[keyboard_player1]
button_a = "KeyX"
button_b = "KeyZ"
select = "ShiftRight"
start = "Enter"
up = "ArrowUp"
down = "ArrowDown"
left = "ArrowLeft"
right = "ArrowRight"

[keyboard_player2]
button_a = "KeyK"
button_b = "KeyJ"
select = "KeyU"
start = "KeyY"
up = "KeyW"
down = "KeyS"
left = "KeyA"
right = "KeyD"

[gamepad_player1]
button_a = "East"
button_b = "South"
select = "Select"
start = "Start"
up = "DPadUp"
down = "DPadDown"
left = "DPadLeft"
right = "DPadRight"

[gamepad_player2]
button_a = "East"
button_b = "South"
select = "Select"
start = "Start"
up = "DPadUp"
down = "DPadDown"
left = "DPadLeft"
right = "DPadRight"
```

## Gamepad Detection

Connected gamepads are automatically detected on startup:
- First gamepad is assigned to Player 1
- Second gamepad is assigned to Player 2

Gamepad connection messages are printed to the console when the emulator starts.

## Troubleshooting

### No gamepads detected

1. Make sure your gamepad is connected before starting the emulator
2. On Linux, check that you have the udev rules set up for your controller
3. Try unplugging and replugging the gamepad

### Gamepad not responding

1. Check that your gamepad is in the correct mode (DirectInput/XInput on Windows)
2. Test the gamepad with another application to ensure it's working
3. Check the console output for any error messages

### Build fails with libudev error

Make sure you have installed the libudev development library as described in the System Requirements section above.

## Supported Controllers

The gamepad support uses the `gilrs` library, which supports a wide range of controllers including:
- Xbox controllers (Xbox 360, Xbox One, Xbox Series X/S)
- PlayStation controllers (DualShock 3, DualShock 4, DualSense)
- Nintendo Switch Pro Controller
- Generic USB/Bluetooth gamepads

For a complete list of supported controllers, see the [gilrs documentation](https://gitlab.com/gilrs-project/gilrs).
