#![no_std]
#![no_main]
use core::panic::PanicInfo;

const RESETS_RESET: u32 = 0x4000c000;
const RESETS_RESET_DONE: u32 = 0x4000c008;
const IO_BANK0_GPIO25_CTRL: u32 = 0x400140cc;
const SIO_GPIO_OE_SET: u32 = 0xd0000024;
const SIO_GPIO_OUT_XOR: u32 = 0xd000001c;

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
pub fn boot2() -> ! {
    // Bring IO_BANK0 out of reset state
    let reset = RESETS_RESET as *mut u32;
    let reset_done = RESETS_RESET_DONE as *mut u32;
    let io_bank_ctr = IO_BANK0_GPIO25_CTRL as *mut u32;
    let sio_oe = SIO_GPIO_OE_SET as *mut u32;
    let sio_out = SIO_GPIO_OUT_XOR as *mut u32;
    unsafe {
        reset.write_volatile(reset.read_volatile() & !(1 << 5));
        while (reset_done.read_volatile() & (1 << 5)) == 0 {}
        // Set GPIO 25 function to SIO
        io_bank_ctr.write_volatile(5);
        // Set output enable for GPIO 25 in SIO
        sio_oe.write_volatile(1 << 25);
        loop {
            // Wait for some time
            for _ in 0..100000 {
                core::hint::black_box(());
            }

            sio_out.write_volatile(1 << 25);
        }
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
