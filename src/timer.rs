use crate::register;

static mut ALARM_INTR0: fn() = default_alarm;

pub unsafe extern "C" fn alarm_intr0() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 0) };
    unsafe { ALARM_INTR0() }
}
pub unsafe extern "C" fn alarm_intr1() {}
pub unsafe extern "C" fn alarm_intr2() {}
pub unsafe extern "C" fn alarm_intr3() {}

pub fn default_alarm() {}

pub fn set_interrupt(f: fn()) {
    // Writing to a static variable is unsafe due to thread safety
    // I think a safe way around this is
    // 'check if alarm 1 is enabled. If it is, return an error'
    // requiring the caller to disable the alarm, and then update it
    // For now, it's probably fine
    unsafe {
        ALARM_INTR0 = f;
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

pub fn set_alarm(time_us: u32) {
    use register::time::ALARM0;
    use register::time::TIMELR;
    let alarm = ALARM0 as *mut usize;
    let time = TIMELR as *mut usize;
    unsafe {
        let now = time.read_volatile();
        let future = now + time_us as usize;
        alarm.write_volatile(future);
    }
}
