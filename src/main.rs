#![no_std]
#![no_main]

mod allocator;
mod gpio;
mod i2c;
mod register;
mod startup;
mod timer;
mod util;
mod vectors;
extern crate alloc;
use crate::gpio::Sio;
use crate::i2c::I2c;
use crate::util::Mutex;
use alloc::boxed::Box;
use core::hint::black_box;
use gpio::Gpio;
use i2c::I2cTrait;
use timer::set_alarm;
use timer::set_interrupt;

static GPIO25: Mutex<Option<Gpio<25, Sio>>> = Mutex::new(None);
static DISPLAY: Mutex<Option<DisplayDriver<I2c<gpio::I2c0>>>> = Mutex::new(None);

pub fn main() {
    let a = Box::new(10);
    let val = black_box(*a);
    let time = val * 100_000;

    let pins = gpio::Pins::take().unwrap();
    let gpio25 = pins.gpio25;
    // set up global gpio25
    {
        let gpio25 = gpio25.sio();
        gpio25.output_enable();
        let mut guard = GPIO25.lock();
        *guard = Some(gpio25);
    }
    // set up global i2c
    {
        let pin0 = pins.gpio0;
        let pin1 = pins.gpio1;
        let addr = 0b0111100;
        let ctrl = i2c::Controller::<gpio::I2c0>::take().unwrap();
        let i2c = i2c::I2c::new(ctrl, addr, pin0, pin1);
        let mut display = DisplayDriver::new(i2c);
        display.display_on();
        let mut guard = DISPLAY.lock();
        *guard = Some(display);
    }

    set_interrupt(move || display_on(750_000, 1), 0);
    set_interrupt(move || display_off(250_000, 0), 1);
    set_alarm(time, 0);

    loop {}
}

fn display_on(time_us: u32, alarm: usize) {
    flip_25();
    if let Some(display) = DISPLAY.lock().as_mut() {
        display.display_on();
        display.all_on();
    }
    set_alarm(time_us, alarm);
}
fn display_off(time_us: u32, alarm: usize) {
    if let Some(display) = DISPLAY.lock().as_mut() {
        display.display_off();
    }
    set_alarm(time_us, alarm);
}

fn alarm_led_loop(time: u32, alarm: usize) {
    flip_25();
    set_alarm(time, alarm);
}
fn alarm_led_loop2(time: u32, alarm: usize) {
    flip_25();
    set_alarm(time, alarm);
}

fn flash_err(period: u32) -> ! {
    set_interrupt(move || alarm_led_loop(period / 2, 1), 0);
    set_interrupt(move || alarm_led_loop2(period / 2, 0), 1);
    set_alarm(period / 2, 0);
    loop {}
}

fn flip_25() {
    {
        if let Some(g) = GPIO25.lock().as_ref() {
            g.flip();
        }
    }
}

struct DisplayDriver<S: I2cTrait> {
    i2c: S,
}

impl<S: I2cTrait> DisplayDriver<S> {
    fn new(i2c: S) -> Self {
        let mut display = Self { i2c };
        display.init();
        display
    }
    fn init(&mut self) {
        self.i2c.send(&[0x00, 0x8D, 0x14]);
        if self.i2c.abrt_src() != 0 {
            flash_err(1_000_000)
        }
    }
    fn display_on(&mut self) {
        self.i2c.send(&[0x00, 0xAF]);
        if self.i2c.abrt_src() != 0 {
            flash_err(1_000_000)
        }
    }
    fn display_off(&mut self) {
        self.i2c.send(&[0x00, 0xAE]);
        if self.i2c.abrt_src() != 0 {
            flash_err(1_000_000)
        }
    }
    fn all_on(&mut self) {
        self.i2c.send(&[0x00, 0xA5]);
        if self.i2c.abrt_src() != 0 {
            flash_err(1_000_000)
        }
    }
}
