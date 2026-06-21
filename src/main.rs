#![no_std]
#![no_main]

mod i2c;
mod register;
mod startup;
mod timer;
mod util;
mod vectors;
use alloc::boxed::Box;
use core::hint::black_box;
use timer::set_alarm;
use timer::set_interrupt;

use crate::i2c::I2c;

mod allocator;
extern crate alloc;

pub fn main() {
    let a = Box::new(10);
    let val = black_box(*a);
    let time = val * 100_000;
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
    let pad_ctrl_0 = register::gpio::pad_ctrl(0) as *mut usize;
    let pad_ctrl_1 = register::gpio::pad_ctrl(1) as *mut usize;
    unsafe {
        let opts = 1 << 1 // schmitt enable
                 | 1 << 3 // pull up enable
                 | 1 << 6; // input enable
        pad_ctrl_0.write_volatile(opts);
        pad_ctrl_1.write_volatile(opts);
    }
    register::gpio::set_i2c(0);
    register::gpio::set_i2c(1);

    let addr = 0b0111100;
    let i2c = i2c::I2c::new(register::i2c::Controller::Zero, addr);
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
    let sio_out = register::sio::gpio::OUT_SET as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
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
    let sio_out = register::sio::gpio::OUT_CLR as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
    }
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
