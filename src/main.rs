#![no_std]
#![no_main]

mod gpio;
mod i2c;
mod register;
mod startup;
mod timer;
mod util;
mod vectors;
use crate::gpio::Sio;
use crate::i2c::I2c;
use crate::util::Mutex;
use alloc::boxed::Box;
use core::cell::RefCell;
use core::hint::black_box;
use core::ops::Deref;
use gpio::Gpio;
use timer::set_alarm;
use timer::set_interrupt;

static GPIO25: Mutex<RefCell<Option<Gpio<25, Sio>>>> =
    Mutex::new(RefCell::new(None));

mod allocator;
extern crate alloc;

pub fn main() {
    let a = Box::new(10);
    let val = black_box(*a);
    let time = val * 100_000;

    let pins = gpio::Pins::take().unwrap();
    let gpio25 = pins.gpio25;
    {
        let gpio25 = gpio25.sio();
        gpio25.output_enable();
        let guard = GPIO25.lock();
        guard.deref().replace(Some(gpio25));
    }
    // set_interrupt(move || alarm_led_loop(750_000, 1), 0);
    // set_interrupt(move || alarm_led_loop(250_000, 0), 1);

    // lets test some i2c now
    // checking the datasheet and looking at my board,,,
    // I'm gonna go with gpio 0 and 1
    // 0: I2C0_SDA
    // 1: I2C0_SCL

    // pull up enable
    // slew rate limit
    // schmitt trigger enable
    let pin0 = pins.gpio0;
    let pin1 = pins.gpio1;

    let addr = 0b0111100;
    let i2c = i2c::I2c::new(register::i2c::Controller::Zero, addr, pin0, pin1);
    i2c.send(&[0x00, 0x8D, 0x14]);
    branch_abort_src(&i2c);

    i2c.send(&[0x00, 0xAF]);
    branch_abort_src(&i2c);
    set_interrupt(move || display_on(750_000, 1, i2c), 0);
    set_interrupt(move || display_off(250_000, 0, i2c), 1);
    set_alarm(time, 0);

    loop {}
}
fn branch_abort_src(i2c: &I2c) {
    let abrt_src = unsafe { i2c.read(register::i2c::TX_ABRT_SOURCE) };
    if abrt_src != 0 {
        flash_err(500_000)
    }
}

fn display_on(time_us: u32, alarm: usize, i2c: I2c) {
    {
        if let Some(g) = GPIO25.lock().borrow().as_ref() {
            g.flip();
        }
    }
    let bytes = &[0, 0xAF];
    i2c.send(bytes);
    branch_abort_src(&i2c);
    let bytes = &[0, 0xA5];
    i2c.send(bytes);
    branch_abort_src(&i2c);

    set_alarm(time_us, alarm);
}
fn display_off(time_us: u32, alarm: usize, i2c: I2c) {
    let bytes = &[0, 0xAE];
    i2c.send(bytes);
    branch_abort_src(&i2c);

    set_alarm(time_us, alarm);
}

fn alarm_led_loop(time: u32, alarm: usize) {
    let sio_out = register::sio::gpio::OUT_XOR as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
    }
    set_alarm(time, alarm);
}
fn alarm_led_loop2(time: u32, alarm: usize) {
    let sio_out = register::sio::gpio::OUT_XOR as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
    }
    set_alarm(time, alarm);
}

fn flash_err(period: u32) -> ! {
    set_interrupt(move || alarm_led_loop(period / 2, 1), 0);
    set_interrupt(move || alarm_led_loop2(period / 2, 0), 1);
    set_alarm(period / 2, 0);
    loop {}
}
