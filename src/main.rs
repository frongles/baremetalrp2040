#![no_std]
#![no_main]
use core::panic::PanicInfo;
mod registers {
    pub mod reset {
        pub const RESET: u32 = 0x4000c000;
        pub const RESET_DONE: u32 = 0x4000c008;
    }
    pub mod gpio {
        pub const IO_BANK0_GPIO25_CTRL: u32 = 0x400140cc;
        pub const OE_SET: u32 = 0xd0000024;
        pub const OUT_XOR: u32 = 0xd000001c;
    }
    pub mod ssi {
        pub const OFFSET: u32 = 0x18000000;
        pub const CTRL0: u32 = 0x18000000;
        pub const SSIENR: u32 = OFFSET + 0x008;
        pub const BAUDR: u32 = OFFSET + 0x014;
        pub const SPI_CTRL0: u32 = OFFSET + 0x0f4;
    }
}

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
pub fn boot2() -> ! {
    // Bring IO_BANK0 out of reset state
    let reset = registers::reset::RESET as *mut u32;
    let reset_done = registers::reset::RESET_DONE as *mut u32;
    unsafe {
        reset.write_volatile(reset.read_volatile() & !(1 << 5));
        while (reset_done.read_volatile() & (1 << 5)) == 0 {}
    }
    init_ssi();

    let io_bank_ctr = registers::gpio::IO_BANK0_GPIO25_CTRL as *mut u32;
    let sio_oe = registers::gpio::OE_SET as *mut u32;
    unsafe {
        // Set GPIO 25 function to SIO
        io_bank_ctr.write_volatile(5);
        // Set output enable for GPIO 25 in SIO
        sio_oe.write_volatile(1 << 25);
    }

    let sio_out = registers::gpio::OUT_XOR as *mut u32;
    unsafe {
        for _ in 0..10 {
            // Wait for some time
            for _ in 0..100000 {
                core::hint::black_box(());
            }

            sio_out.write_volatile(1 << 25);
        }
        sio_out.write_volatile(1 << 25);
    }
    let sio_out = registers::gpio::OUT_XOR as *mut u32;
    unsafe {
        let main_ptr = 0x10000101u32 as *const (); // +1 for Thumb bit
        let main_fn: fn() = core::mem::transmute(main_ptr);
        main_fn();
    }
    loop {}
}

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
#[inline(never)]
pub fn main() {
    let sio_out = registers::gpio::OUT_XOR as *mut u32;
    unsafe {
        sio_out.write_volatile(1 << 25);
        loop {
            // Wait for some time
            for _ in 0..1000000 {
                core::hint::black_box(());
            }

            sio_out.write_volatile(1 << 25);
        }
    }
}
/// missing form this function
/// - writes to TXFLTR, RXFLTR for FIFO level
/// - IMR interrupt masks
/// for both master and slave
#[inline(always)]
fn init_ssi() {
    // To setup SPI
    // set SSI type to SPI in CTRL_0
    // reset SSI
    //  set SSI_EN to 0
    // init master
    //  write CTRL0 to be SPI
    //  write BAUDR set baudrate
    //      make sure to set SCKDV clock ratio and make it an integer
    //  write TXFLTR to set FIFO level
    //  write RXFLTR to set FIFO level
    //  write IMR set interrupt masks
    //  write SER register bit 0 to 1
    //  write SSIENR bit 0 to 1 to enable master
    // init slave
    //  write CTRL0 as well...
    //  write TXFLTR to set FIFO level
    //  write RXFLTR to set FIFO level
    //  write IMR set interrupt masks
    //  write SSIENR bit 0 to 1 to enable slave
    //  write to TX FIFO if ready to receive bytes
    // start transfer
    //  write datas into master's TX FIFO
    //  user can poll busy status, or use an ISR interrupt
    //
    use registers::ssi::*;
    let ctrl0 = CTRL0 as *mut u32;
    let spi_ctrl0 = SPI_CTRL0 as *mut u32;
    let enr = SSIENR as *mut u32;
    let baud = BAUDR as *mut u32;
    unsafe {
        // reset ssi
        enr.write_volatile(0);

        // set clock divider to 4
        baud.write_volatile(4);

        // writes opts to ctrl0 register
        let tmod_eeprom_read = 3 << 8;
        let data_frame_size = 31 << 16;
        let opts = tmod_eeprom_read | data_frame_size;
        ctrl0.write_volatile(opts);

        // write opts to spi ctrl0 register
        let addr_len = 6 << 2;
        let instr_len = 2 << 8;
        let ssi_cmd = 0x03 << 24;
        let opts = addr_len | instr_len | ssi_cmd;
        spi_ctrl0.write_volatile(opts);

        // set ssi
        enr.write_volatile(1);
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
