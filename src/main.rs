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
        pub const SR: u32 = OFFSET + 0x028;
        pub const DR0: u32 = OFFSET + 0x060;
        pub const SPI_CTRL0: u32 = OFFSET + 0x0f4;
    }
}

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
pub fn boot2() -> ! {
    // Reset system
    let reset = registers::reset::RESET as *mut u32;
    let reset_done = registers::reset::RESET_DONE as *mut u32;
    unsafe {
        reset.write_volatile(reset.read_volatile() & !(1 << 5));
        while (reset_done.read_volatile() & (1 << 5)) == 0 {}
    }
    // init ssi
    //init_ssi();
    //init_quad_ssi();
    init_quad_ssi_io();

    // setup io bank
    let io_bank_ctr = registers::gpio::IO_BANK0_GPIO25_CTRL as *mut u32;
    let sio_oe = registers::gpio::OE_SET as *mut u32;
    unsafe {
        // Set GPIO 25 function to SIO
        io_bank_ctr.write_volatile(5);
        // Set output enable for GPIO 25 in SIO
        sio_oe.write_volatile(1 << 25);
    }

    // startup blinky sequence
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
        let data_frame_size = 31 << 16; // 32 clocks per frame
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

#[inline(always)]
fn init_quad_ssi() {
    // similar to init_ssi,
    // but need to enable QE bit on the flash
    // So we need to ask the flash about it's config and set it if it's not set
    use registers::ssi::*;
    let ctrl0 = CTRL0 as *mut u32;
    let enr = SSIENR as *mut u32;
    let baud = BAUDR as *mut u32;
    // first ini basic spi
    unsafe {
        // reset ssi
        enr.write_volatile(0);

        // set clock divider to 4
        baud.write_volatile(4);

        // writes opts to ctrl0 register
        let data_frame_size = 7 << 16; // 8 clocks per frame now
        let opts = data_frame_size;
        ctrl0.write_volatile(opts);

        // we don't change spi_crtl0, since we make assumptions about start up
        // state...

        // set ssi
        enr.write_volatile(1);
    }

    // this is my data register
    // Write to it to send data
    // Read from it to receive data
    let sr = SR as *mut u32;
    let dr0 = DR0 as *mut u32;
    unsafe {
        // these commands go into an SPI FIFO TX queue
        // We can read the results from the same register, which from from an RX
        // FIF queue
        dr0.write_volatile(0x05); // read flash status register 1
        dr0.write_volatile(0x35); // read flash status register 2
        // wait while busy and while queue not empty
        wait_done(sr);
        let flash_sr1 = dr0.read_volatile();
        let flash_sr2 = dr0.read_volatile();

        if flash_sr2 & (1 << 1) == 0 {
            // QE not enabled, let's enable it
            dr0.write_volatile(0x06); // write enable
            wait_done(sr);
            //wait_done(sr);
            dr0.write_volatile(0x01); // write to status registers
            dr0.write_volatile(flash_sr1); // write to sr1
            //  write to sr2 with new QE bit
            dr0.write_volatile(flash_sr2 | (1 << 1));
            wait_done(sr);
            //wait_done(sr);
            dr0.write_volatile(0x04); // write disable
        }
    }
    // now we have to restart our own ssi, to work in quad mode, with the new
    // settings
    // write opts to spi ctrl0 register
    let spi_ctrl0 = SPI_CTRL0 as *mut u32;
    unsafe {
        // reset enable regesiter
        enr.write_volatile(0);

        // configure spi ctrl
        let tmod_eeprom_read = 3 << 8; // set to eeprom read mode
        let data_frame_size = 31 << 16; // 32 clocks per frame again... 
        let spi_frame_format = 2 << 21; // quad spi mode!
        let opts = tmod_eeprom_read | data_frame_size | spi_frame_format;
        ctrl0.write_volatile(opts);

        // address length is 24 bits (24 / 4 = 6, at r[2..3]
        let addr_len = 6 << 2;
        // instruction length is 8 bits (8 / 4 = 2)
        let instr_len = 2 << 8;
        // this ssi cmd is for the flash to send out quad spi
        let ssi_cmd = 0x6B << 24;
        // wait 8 clock cycles between sending data and receiving
        // this is what the flash requires
        let wait_cycles = 8 << 11;
        let opts = addr_len | instr_len | ssi_cmd | wait_cycles;
        spi_ctrl0.write_volatile(opts);

        enr.write_volatile(1);
    }
    // done!
}

#[inline(always)]
fn init_quad_ssi_io() {
    // similar to init_ssi,
    // but need to enable QE bit on the flash
    // So we need to ask the flash about it's config and set it if it's not set
    use registers::ssi::*;
    let ctrl0 = CTRL0 as *mut u32;
    let enr = SSIENR as *mut u32;
    let baud = BAUDR as *mut u32;
    // first ini basic spi
    unsafe {
        // reset ssi
        enr.write_volatile(0);

        // set clock divider to 4
        baud.write_volatile(4);

        // writes opts to ctrl0 register
        let data_frame_size = 7 << 16; // 8 clocks per frame now
        let opts = data_frame_size;
        ctrl0.write_volatile(opts);

        // we don't change spi_crtl0, since we make assumptions about start up
        // state...

        // set ssi
        enr.write_volatile(1);
    }

    // this is my data register
    // Write to it to send data
    // Read from it to receive data
    let sr = SR as *mut u32;
    let dr0 = DR0 as *mut u32;
    unsafe {
        // these commands go into an SPI FIFO TX queue
        // We can read the results from the same register, which from from an RX
        // FIF queue
        dr0.write_volatile(0x05); // read flash status register 1
        dr0.write_volatile(0x35); // read flash status register 2
        // wait while busy and while queue not empty
        wait_done(sr);
        let flash_sr1 = dr0.read_volatile();
        let flash_sr2 = dr0.read_volatile();

        if flash_sr2 & (1 << 1) == 0 {
            // QE not enabled, let's enable it
            dr0.write_volatile(0x06); // write enable
            wait_done(sr);
            //wait_done(sr);
            dr0.write_volatile(0x01); // write to status registers
            dr0.write_volatile(flash_sr1); // write to sr1
            //  write to sr2 with new QE bit
            dr0.write_volatile(flash_sr2 | (1 << 1));
            wait_done(sr);
            //wait_done(sr);
            dr0.write_volatile(0x04); // write disable
        }
    }
    // now we have to restart our own ssi, to work in quad mode, with the new
    // settings
    // write opts to spi ctrl0 register
    let spi_ctrl0 = SPI_CTRL0 as *mut u32;
    unsafe {
        // reset enable regesiter
        enr.write_volatile(0);

        // configure spi ctrl
        let tmod_eeprom_read = 3 << 8; // set to eeprom read mode
        let data_frame_size = 31 << 16; // 32 clocks per frame again... 
        let spi_frame_format = 2 << 21; // quad spi mode!
        let opts = tmod_eeprom_read | data_frame_size | spi_frame_format;
        ctrl0.write_volatile(opts);

        // address length is 32 bits addr + 8 bit mode
        let addr_len = 8 << 2;
        // instruction length is 8 bits (8 / 4 = 2)
        let instr_len = 2 << 8;
        // this ssi cmd is for the flash to send out quad spi
        // Its' set to 1 to go read the command from FRF
        let ssi_cmd = 0x01 << 0;
        // wait 4 clock cycles between sending data and receiving
        // this is what the flash requires for quad spio
        let wait_cycles = 4 << 11;
        let opts = addr_len | instr_len | ssi_cmd | wait_cycles;
        spi_ctrl0.write_volatile(opts);

        enr.write_volatile(1);
    }

    // send dummy bytes to register mode bits
    unsafe {
        dr0.write_volatile(0xeb); // this is quadIO cmd
        dr0.write_volatile(0xa0); // this is the mode bits
        wait_done(sr);
        dr0.read_volatile();
        dr0.read_volatile();
    }

    // reconfigure ssi to not send instruction
    unsafe {
        enr.write_volatile(0);
        // configure spi ctrl
        // address length is 32 bits addr + 8 bit mode
        let addr_len = 8 << 2;
        // instruction length is 0 bites
        let instr_len = 0 << 8;
        // tell mcu to append ssi_cmd to the address and send no instruction
        let tx_type = 0x02 << 0;
        // ssi command for QuadIO
        let ssi_cmd = 0xa0 << 24;
        // wait 4 clock cycles between sending data and receiving
        // this is what the flash requires for quad spio
        let wait_cycles = 4 << 11;
        let opts = addr_len | instr_len | ssi_cmd | wait_cycles | tx_type;
        spi_ctrl0.write_volatile(opts);
        enr.write_volatile(1);
    }

    // done!
}

#[inline(always)]
unsafe fn wait_done(sr: *mut u32) {
    loop {
        let s = unsafe { sr.read_volatile() };
        if s & 1 == 0 && s & (1 << 2) != 0 {
            break;
        }
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
