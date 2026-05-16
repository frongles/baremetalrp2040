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

pub fn main() {
    let sio_out = register::gpio::OUT_XOR as *mut usize;
    set_alarm(1_000_000);
    unsafe {
        sio_out.write_volatile(1 << 25);
        for _ in 0..999 {
            // Wait for some time
            for _ in 0..1000000 {
                core::hint::black_box(());
            }

            sio_out.write_volatile(1 << 25);
        }
    }
    loop {}
}
