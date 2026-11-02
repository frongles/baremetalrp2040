# Bare-metal Rust on the RP2040

A learning project exploring the low level workings of embedded development. The firmware implements a boot loader, interrupt vector table, basic allocator, basic GPIO and I2C abstractions for the RP2040 chip on the Raspberry Pi Pico board. There are no third party dependencies for the run time of the firmware. Third party software is used for converting the compiled binary to UF2 format to aid flashing the binary to the device.

The motivation for this project was to learn more about low level hardware and explore the levels of code which are usually abstracted over when using high level frameworks and IDEs, to give better context when using high level frameworks in the future. 

By doing this project, useful insight was gained on hardware implementations for atomics, clock initialisation, and different modes of sleep and power consumption.

The original goal was to connect to a peripheral CO2 sensor and a small display and display live CO2 readings. Currently, and OLED display on/off demo has been achieved.

## Current functionality and evidence

The RP2040 has a 2 stage boot. The first stage is baked into the hardware. It reads 256 bytes from external flash over SPI, performs a checksum on the bytes, then executes those bytes. 

It is then up to the programmer to write a sufficent bootloader that will bootstrap the SPI interface, begin executing bytes from the rest of the flash (no longer limited to 256 bytes), and then bootstrap the required interfaces for their program (clocks, GPIO, I2C, etc). 


| Area | Implemented |
| --- | --- |
| Boot and runtime | Second-stage boot, quad-SPI execute-in-place setup, vector table, `.data`/`.bss` initialisation, oscillator and PLL setup |
| GPIO | Owned pin resources, type-state function selection, SIO output control |
| Timers | Hardware alarm interrupts |
| OLED / I2C | Initialisation, entire-display-on, and display-off commands |
| Allocation | 32 KiB heap, bump allocator with limited reclamation, hardware-spinlock-backed mutex |
| Build | ARM ELF → raw binary → boot CRC patch → UF2 |


## Build

Run commands from the repository root

Prerequisites:

- Rust
- GNU Arm binutils providing `arm-none-eabi-objcopy` on `PATH`
- Python 3.9 or later for the CRC patcher and UF2 converter.

For a fresh checkout, use `git clone --recurse-submodules <repository-url>` to fetch the converter along with the project.

Build and convert the firmware:

```sh
cargo run --release
```

Cargo's runner invokes `build.sh`, which converts the ELF to a raw binary, patches the boot checksum, and generates `target/thumbv6m-none-eabi/release/baremetal.uf2`. The script stops if any step fails. This command generates the UF2 and automatically flashes the board if one is connected.

## Source

| File | Responsibility |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Demo, early OLED driver, display startup workaround |
| [`src/startup/`](src/startup/) | Second-stage boot, flash/XIP, clocks, runtime initialisation |
| [`src/vectors.rs`](src/vectors.rs), [`link.ld`](link.ld) | Interrupt vectors and memory layout |
| [`src/gpio.rs`](src/gpio.rs), [`src/i2c.rs`](src/i2c.rs) | Pin ownership/function types and I²C register access |
| [`src/timer.rs`](src/timer.rs) | Alarm callbacks and timing experiments |
| [`src/allocator.rs`](src/allocator.rs), [`src/util.rs`](src/util.rs) | Heap allocator and synchronisation primitives |
| [`src/register.rs`](src/register.rs) | Register addresses and offsets |
| [`build.sh`](build.sh), [`patch_bootloader_crc.py`](patch_bootloader_crc.py) | Binary conversion, boot CRC patching, UF2 generation |

