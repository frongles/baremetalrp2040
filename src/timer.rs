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

pub unsafe extern "C" fn alarm_intr0() {
    loop {}
}
pub unsafe extern "C" fn alarm_intr1() {}
pub unsafe extern "C" fn alarm_intr2() {}
pub unsafe extern "C" fn alarm_intr3() {}

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
