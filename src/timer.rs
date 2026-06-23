use crate::Box;
use crate::register;
use crate::util::Mutex;

static ALARM_0: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
static ALARM_1: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
static ALARM_2: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
static ALARM_3: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

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
