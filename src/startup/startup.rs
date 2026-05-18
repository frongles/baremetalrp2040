use crate::allocator::ALLOCATOR;
use crate::main;
use crate::register;
use crate::register::flash::XIP_BASE;
use crate::startup::ssi_init;
use crate::vectors;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use core::slice::SplitInclusive;

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
    init_spinlock();
    init_xosc();
    select_xosc_as_ref_clk();

    unreset(1 << 12); // unreset pll
    init_pll_sys();
    select_pll_as_sys_clk();

    init_data();
    init_bss();

    unreset(1 << 21); // unreset timer
    init_watchdog();
    init_alarm();
    unreset(1 << 5); // unreset io bank
    init_gpio();
    ALLOCATOR.init(
        _sheap as *const usize as usize,
        _eheap as *const usize as usize,
    );

    main();
    loop {}
}

fn init_data() {
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
}

fn init_bss() {
    let mut bss_start = _sbss as *mut usize;
    let bss_end = _ebss as *mut usize;
    while bss_start < bss_end {
        unsafe {
            bss_start.write_volatile(0);
            bss_start = bss_start.add(1);
        }
    }
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

/// Pull the specified bits that are high in `bits` out of reset
/// This function will
/// - Take the bitwise NOT of `bits` and perform
///   an AND with the existing reset bits.
/// - Wait until all systems are out of reset
///
pub fn unreset(bits: usize) {
    let reset = (register::reset::RESET) as *mut usize;
    let reset_done = register::reset::RESET_DONE as *const usize;
    unsafe {
        let old = reset.read_volatile();
        let new = old & !bits;
        reset.write_volatile(new);
        while reset_done.read_volatile() & bits != bits {}
    }
}

pub fn init_gpio() {
    // init io stuffs
    let io_bank_ctr = register::gpio::IO_BANK0_GPIO25_CTRL as *mut usize;
    let sio_oe = register::gpio::OE_SET as *mut usize;
    unsafe {
        // Set GPIO 25 function to SIO
        io_bank_ctr.write_volatile(5);
        // Set output enable for GPIO 25 in SIO
        sio_oe.write_volatile(1 << 25);
    }
}

pub fn init_alarm() {
    use register::m0plus::NVIC_ISER;
    use register::time::INTE;

    let inte = INTE as *mut usize;
    let nvic = NVIC_ISER as *mut usize;
    unsafe {
        // enable ALARM_0
        inte.write_volatile(1 << 0);
        // enable TIMER_IRQ_0
        nvic.write_volatile(1 << 0);
    }
}

pub fn select_xosc_as_ref_clk() {
    use register::clock::{REF_CTRL, REF_DIV, REF_SELECTED};
    let ctrl = REF_CTRL as *mut usize;
    let div = REF_DIV as *mut usize;
    let selected = REF_SELECTED as *mut usize;

    unsafe {
        // Set divider to 1 (bits 9:8)
        div.write_volatile(1 << 8);

        // clk_ref glitchless mux: bits 1:0
        // 0 = ROSC, 1 = aux mux, 2 = XOSC
        ctrl.write_volatile(0x2);

        // Poll REF_SELECTED until bit 2 is set (XOSC = source 2, so bit 1<<2)
        while selected.read_volatile() & (1 << 2) == 0 {}
    }
}

pub fn select_pll_as_sys_clk() {
    use register::clock::SYS_CTRL;
    use register::clock::SYS_SELECTED;
    let ctrl = SYS_CTRL as *mut usize;
    let selected = SYS_SELECTED as *mut usize;
    unsafe {
        ctrl.write_volatile(1);
        while selected.read_volatile() & (1 << 1) == 0 {}
    }
}

pub fn init_pll_sys() {
    use register::pll_sys::*;
    let fbdiv = FBDIV_INT as *mut usize;
    let pwr = PWD as *mut usize;
    let cs = CS as *mut usize;
    let prim = PRIM as *mut usize;
    unsafe {
        cs.write_volatile(1 << 0);
        fbdiv.write_volatile(100);
        pwr.write_volatile(!(1 << 0 | 1 << 5));
        while cs.read_volatile() >> 31 == 0 {}
        prim.write_volatile(6 << 16 | 2 << 12);
        pwr.write_volatile(!(1 << 0 | 1 << 2 | 1 << 3 | 1 << 5));
    }
}

pub fn init_xosc() {
    use register::xosc::CTRL;
    use register::xosc::STATUS;
    let ctrl = CTRL as *mut usize;
    let stat = STATUS as *mut usize;

    unsafe {
        let read = stat.read_volatile();
        if (read >> 31) & 1 == 1 {
            return;
        }
        ctrl.write_volatile(0xfab << 12);
        loop {
            let read = stat.read_volatile();
            let _enabled = (read >> 12) & 1;
            let _bad_write = (read >> 24) & 1;
            let stable = (read >> 31) & 1;
            if stable == 1 {
                return;
            }
        }
    }
}

pub fn init_watchdog() {
    use register::watchdog::TICK;
    let tick = TICK as *mut usize;
    unsafe {
        tick.write_volatile(1 << 9 | 12);
    }
}

fn init_spinlock() {
    use register::sio::SPINLOCK_OFFSET;
    let mut i = 0;
    while i < 32 {
        let spinlock = (SPINLOCK_OFFSET + i * 4) as *mut usize;
        unsafe { spinlock.write_volatile(1) }
        i = i + 1;
    }
}
