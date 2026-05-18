use crate::interrupt::InterruptSlot;
use crate::register;

static ALARM_0: InterruptSlot = InterruptSlot::new();
static ALARM_1: InterruptSlot = InterruptSlot::new();
static ALARM_2: InterruptSlot = InterruptSlot::new();
static ALARM_3: InterruptSlot = InterruptSlot::new();

pub unsafe extern "C" fn alarm_intr0() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 0) };
    ALARM_0.call();
}
pub unsafe extern "C" fn alarm_intr1() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 1) };
    ALARM_1.call();
}
pub unsafe extern "C" fn alarm_intr2() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 2) };
    ALARM_2.call();
}
pub unsafe extern "C" fn alarm_intr3() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 3) };
    ALARM_3.call();
}

/// Set `f` to be called on alam number `alarm`
pub fn set_interrupt(f: impl Fn() + 'static, alarm: u8) {
    match alarm {
        0 => ALARM_0.set(f),
        1 => ALARM_1.set(f),
        2 => ALARM_2.set(f),
        3 => ALARM_3.set(f),
        _ => {}
    }
}

/// Set alarm 0
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
