use crate::allocator;
use crate::gpio_init::init_gpio;
use crate::main;
use crate::register;
use crate::register::flash::XIP_BASE;
use crate::ssi_init;
use crate::timer::init_alarm;
use crate::timer::init_xosc;
use crate::vectors;
use crate::watchdog::init_watchdog;
use core::panic::PanicInfo;

unsafe extern "C" {
    pub fn _sdata();
    pub fn _edata();
    pub fn _sdataf();

    pub fn _sbss();
    pub fn _ebss();

    pub fn _sheap();
    pub fn _eheap();
}

pub unsafe extern "C" fn reset_handler() {
    let data_start = _sdata as *mut usize;
    let data_end = _edata as *const usize;
    let mut flash_ptr = _sdataf as *const usize;
    let mut ram_ptr = data_start;

    while (ram_ptr as *const usize) < data_end {
        unsafe {
            ram_ptr.write_volatile(flash_ptr.read());
            ram_ptr = ram_ptr.add(1);
            flash_ptr = flash_ptr.add(1);
        }
    }

    let mut bss_start = _sbss as *mut usize;
    let bss_end = _ebss as *mut usize;
    while bss_start < bss_end {
        unsafe {
            bss_start.write_volatile(0);
            bss_start = bss_start.add(1);
        }
    }

    // init io stuffs
    init_gpio();
    init_xosc();
    init_watchdog();
    init_alarm();

    unsafe {
        allocator::ALLOCATOR.get().init(
            _sheap as *const usize as usize,
            _eheap as *const usize as usize,
        )
    };

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
