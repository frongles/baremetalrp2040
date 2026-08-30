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
use gpio::Gpio;
use i2c::I2cTrait;
use timer::set_alarm;
use timer::set_interrupt;

static GPIO25: Mutex<Option<Gpio<25, Sio>>> = Mutex::new(None);
static DISPLAY: Mutex<Option<DisplayDriver>> = Mutex::new(None);

pub fn main() {
    let time = 1_000_000;
    // When I flash the device, the display works fine
    // On reboot, the display won't work
    // Unless I have this delay
    //
    // I belive the display is not ready quite as fast as the rp2040 is (needs to build charge pump
    // maybe?) but the docs don't actually mention a required wait time between power on and i2c
    // commands. I had found some evidence online that this was the case but lost it
    //
    for _ in 0..100000 {
        core::hint::black_box(());
    }

    let pins = gpio::Pins::take().unwrap();
    let gpio25 = pins.gpio25;

    // set up global gpio25
    {
        let gpio25 = gpio25.sio();
        gpio25.output_enable();
        *GPIO25.lock() = Some(gpio25);
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
        *DISPLAY.lock() = Some(display);
    }

    set_interrupt(move || display_on(5_000_000, 1), 0);
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
    flip_25();
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

pub fn flash_err(period: u32) -> ! {
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

const BUFFER_SIZE: usize = 128 * 64 / 8;
struct DisplayDriver {
    i2c: I2c<gpio::I2c0>,
    buffer: [u8; BUFFER_SIZE],
}

impl DisplayDriver {
    fn new(i2c: I2c<gpio::I2c0>) -> Self {
        let mut display = Self {
            i2c,
            buffer: [0; BUFFER_SIZE],
        };
        display.init();
        display
    }
    fn init(&mut self) {
        // set mux ratio
        self.i2c.send(&[0x00, 0xA8, 0x20]);
        // Set display offset
        self.i2c.send(&[0x00, 0xD3, 0x00]);
        // Set display start line
        self.i2c.send(&[0x00, 0x40]);
        // set segment remap
        self.i2c.send(&[0x00, 0xA0]); // 0xA0/0xA1
        // set COM output scan direction
        self.i2c.send(&[0x00, 0xC0]); // 0xC0/0xC8
        // set COM pins hardware config
        self.i2c.send(&[0x00, 0xDA, 0x02]);
        // set contrast control
        self.i2c.send(&[0x00, 0x81, 0x7F]);
        // disable entire display ON
        self.i2c.send(&[0x00, 0xA4]);
        // set normal display (not inverted)
        self.i2c.send(&[0x00, 0xA6]);
        // set osc freq
        self.i2c.send(&[0x00, 0xD5, 0x80]);
        // init charge pump
        self.i2c.send(&[0x00, 0x8D, 0x14]);
        // display on
        self.i2c.send(&[0x00, 0xAF]);
        branch_abrt(self.i2c.abrt_src());
    }
    fn display_on(&mut self) {
        self.i2c.send(&[0x00, 0xAF]);
        branch_abrt(self.i2c.abrt_src());
    }
    fn display_off(&mut self) {
        self.i2c.send(&[0x00, 0xAE]);
        branch_abrt(self.i2c.abrt_src());
    }
    fn all_on(&mut self) {
        self.i2c.send(&[0x00, 0xA5]);
        branch_abrt(self.i2c.abrt_src());
    }
    fn flush(&mut self) {
        let cmd = &[0x01]; // check this
        let ops = [
            i2c::Operation::Write(cmd),
            i2c::Operation::Write(&self.buffer),
        ];
        self.i2c.transaction(&ops);
    }
}
pub fn branch_abrt(src: usize) {
    if src & !1 != 0 {
        flash_err(1_000_000);
    }
}
