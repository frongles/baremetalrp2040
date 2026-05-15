#![no_std]
#![no_main]

mod gpio_init;
mod register;
mod ssi_init;
mod startup;
mod vectors;

pub fn main() {
    let sio_out = register::gpio::OUT_XOR as *mut usize;
    unsafe {
        sio_out.write_volatile(1 << 25);
        for _ in 0..49 {
            // Wait for some time
            for _ in 0..100000 {
                core::hint::black_box(());
            }

            sio_out.write_volatile(1 << 25);
        }
    }
    loop {}
}
