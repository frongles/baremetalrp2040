#![no_std]
#![no_main]

mod gpio_init;
mod register;
mod ssi_init;
mod startup;
mod timer;
mod vectors;
mod watchdog;
use timer::set_alarm;

use crate::timer::set_interrupt;

pub fn main() {
    set_interrupt(alarm_led_loop);
    set_alarm(100_000);
    loop {}
}

fn alarm_led_loop() {
    let sio_out = register::gpio::OUT_XOR as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
    }
    set_alarm(100_000);
}
