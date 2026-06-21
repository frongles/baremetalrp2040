use crate::Box;
use crate::register;
use crate::util::Wrapped;

static ALARM_0: Wrapped<Option<Box<dyn Fn() + Send>>> = Wrapped::new(None, 1);
static ALARM_1: Wrapped<Option<Box<dyn Fn() + Send>>> = Wrapped::new(None, 2);
static ALARM_2: Wrapped<Option<Box<dyn Fn() + Send>>> = Wrapped::new(None, 3);
static ALARM_3: Wrapped<Option<Box<dyn Fn() + Send>>> = Wrapped::new(None, 4);

pub unsafe extern "C" fn alarm_intr0() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 0) };
    let guard = ALARM_0.lock();
    if let Some(f) = guard.as_ref() {
        f()
    }
}
pub unsafe extern "C" fn alarm_intr1() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 1) };
    let guard = ALARM_1.lock();
    if let Some(f) = guard.as_ref() {
        f()
    }
}
pub unsafe extern "C" fn alarm_intr2() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 2) };
    let guard = ALARM_2.lock();
    if let Some(f) = guard.as_ref() {
        f()
    }
}
pub unsafe extern "C" fn alarm_intr3() {
    let intr = register::time::INTR as *mut usize;
    unsafe { intr.write_volatile(1 << 3) };
    let guard = ALARM_3.lock();
    if let Some(f) = guard.as_ref() {
        f()
    }
}

/// Set `f` to be called on alam number `alarm`
pub fn set_interrupt(f: impl Fn() + 'static + Send, alarm: u8) {
    match alarm {
        0 => ALARM_0.set(f),
        1 => ALARM_1.set(f),
        2 => ALARM_2.set(f),
        3 => ALARM_3.set(f),
        _ => {}
    }
}

/// Set alarm `alarm`
pub fn set_alarm(time_us: u32, alarm: usize) {
    use register::time::ALARM0;
    use register::time::TIMELR;
    let alarm = (ALARM0 + alarm * 4) as *mut usize;
    let time = TIMELR as *mut usize;
    unsafe {
        let now = time.read_volatile();
        let future = now + time_us as usize;
        alarm.write_volatile(future);
    }
}
