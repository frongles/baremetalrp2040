use crate::interrupt::InterruptSlot;
use crate::register;

static ALARM_0: InterruptSlot = InterruptSlot::new();

pub unsafe extern "C" fn alarm_intr0() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 0) };
    ALARM_0.call();
}
pub unsafe extern "C" fn alarm_intr1() {}
pub unsafe extern "C" fn alarm_intr2() {}
pub unsafe extern "C" fn alarm_intr3() {}

pub fn set_interrupt(f: impl Fn() + 'static) {
    // Writing to a static variable is unsafe due to thread safety
    // I think a safe way around this is
    // 'check if alarm 1 is enabled. If it is, return an error'
    // requiring the caller to disable the alarm, and then update it
    // For now, it's probably fine
    ALARM_0.set(f);
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
