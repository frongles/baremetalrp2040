use crate::gpio_init::init_gpio;
use crate::register;
use crate::startup::reset_handler;
// declare the _sstack linker variable as a function...
// we are cheating. Rust allows this but doesn't allow things like
// extern "C" usize, or extern "C" u32, since they are not Sync
// But it allows functions...
// So we pretend the SP is a function
// and the linker will will in _sstack
unsafe extern "C" {
    pub fn _sstack();
    pub fn _svtable();
}
type Handler = unsafe extern "C" fn();

unsafe extern "C" fn default_handler() {
    // setup io bank
    init_gpio();
    // startup blinky sequence
    let sio_out = register::gpio::OUT_XOR as *mut usize;
    loop {
        unsafe {
            // Wait for some time
            for _ in 0..100000 {
                core::hint::black_box(());
            }

            sio_out.write_volatile(1 << 25);
        }
    }
}

const fn make_vector_table() -> [Option<Handler>; 48] {
    let default = Some(default_handler as Handler);
    let mut table = [default as Option<Handler>; 48]; // fill all with default
    table[0] = Some(_sstack);
    table[1] = Some(reset_handler);
    // reserved block from 4 - 10
    let mut i = 4;
    while i < 11 {
        table[i] = None;
        i = i + 1;
    }
    table[12] = None; // reserved
    table[13] = None; // reserved

    // reserved block form 42 - 47
    let mut i = 42;
    while i < 48 {
        table[i] = None;
        i = i + 1;
    }

    // etc
    table
}

#[unsafe(link_section = ".vector")]
#[unsafe(no_mangle)]
pub static VECTOR_TABLE: [Option<Handler>; 48] = make_vector_table();
