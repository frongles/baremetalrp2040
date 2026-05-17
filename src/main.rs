#![no_std]
#![no_main]

mod gpio_init;
mod register;
mod ssi_init;
mod startup;
mod timer;
mod vectors;
mod watchdog;
use core::hint::black_box;

use alloc::boxed::Box;
use timer::set_alarm;
use timer::set_interrupt;

mod allocator;
extern crate alloc;

pub fn main() {
    let a = Box::new(100);
    let val = black_box(*a);
    let time = val * 100_000;
    set_interrupt(|| alarm_led_loop(100_000));
    set_alarm(time);
    loop {}
}

fn alarm_led_loop(time: u32) {
    let sio_out = register::gpio::OUT_XOR as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
    }
    set_alarm(time);
}
