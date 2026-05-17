// init watchdog tick
//  TICK register...
// set up alarm
// create alarm interrupt handler
//      must clear interrupt bits
//      then do what you want it to
//  decide on an alarm number 0
//  enable interrupt for alarm number 0
//  set handler to pointer of irq handler for the irq number of alarm 0
//  enable irq number of alarm 0
//  write target time to lower 32 bits of alarm
//
//
// Write to ALARM_INTE to enable ALARM0
// Enable appropriate timer interrupt TIMER0
// Write time you want to fire at ALARM0
//      i.e. current time in TIMERAWL + alarm time in microseconds
// To clear latched interrupt, write 1 to bit in INTR
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
    // 'check if alarm 1 is enabled. If it is, return an error.
    // requiring the caller to disable the alarm
    // For now, it's fine
    unsafe {
        ALARM_INTR0 = f;
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
            let enabled = (read >> 12) & 1;
            let bad_write = (read >> 24) & 1;
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
