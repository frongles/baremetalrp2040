use crate::register;

pub fn init_watchdog() {
    use register::watchdog::TICK;
    let tick = TICK as *mut usize;
    unsafe {
        tick.write_volatile(1 << 9 | 12);
    }
}
