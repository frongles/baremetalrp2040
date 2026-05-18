#![no_std]
#![no_main]

mod interrupt;
mod register;
mod startup;
mod timer;
mod vectors;
use alloc::boxed::Box;
use core::hint::black_box;
use timer::set_alarm;
use timer::set_interrupt;

mod allocator;
extern crate alloc;

pub fn main() {
    let a = Box::new(10);
    let val = black_box(*a);
    let time = val * 100_000;
    set_interrupt(move || alarm_led_loop(val * 500_000));
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
