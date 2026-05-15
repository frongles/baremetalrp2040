use crate::gpio_init::init_gpio;
use crate::main;
use crate::register;
use crate::ssi_init;
use crate::vectors;
use core::panic::PanicInfo;

use crate::register::flash::XIP_BASE;
pub unsafe extern "C" fn reset_handler() {
    // init io stuffs
    init_gpio();
    main();
    loop {}
}

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
pub fn boot2() -> ! {
    // Reset system
    let reset = register::reset::RESET as *mut usize;
    let reset_done = register::reset::RESET_DONE as *mut usize;
    unsafe {
        reset.write_volatile(reset.read_volatile() & !(1 << 5));
        while (reset_done.read_volatile() & (1 << 5)) == 0 {}
    }
    // init ssi
    ssi_init::init_quad_ssi_io();

    // Now we can load Vector Table Offset Register to point to our VTable
    let vtor = register::m0plus::VTOR as *mut usize;
    unsafe { vtor.write_volatile(vectors::_svtable as *const () as usize) };

    // Now we can load our stack pointer from the vtable
    unsafe {
        let sp = *((XIP_BASE + 0x100) as *const usize);
        core::arch::asm!("msr msp, {sp}", sp = in(reg) sp);

        // and we can branch to our reset handler
        let rh = *((XIP_BASE + 0x104) as *const usize);
        core::arch::asm!("bx {rh}", rh = in(reg) rh, options(noreturn));
    }
}
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
