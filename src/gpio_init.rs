use crate::register;

pub fn init_gpio() {
    // init io stuffs
    let io_bank_ctr = register::gpio::IO_BANK0_GPIO25_CTRL as *mut usize;
    let sio_oe = register::gpio::OE_SET as *mut usize;
    unsafe {
        // Set GPIO 25 function to SIO
        io_bank_ctr.write_volatile(5);
        // Set output enable for GPIO 25 in SIO
        sio_oe.write_volatile(1 << 25);
    }
}
