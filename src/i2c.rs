use crate::register::i2c::{self, Controller};
use core::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct I2c {
    ctrl: i2c::Controller,
}
impl I2c {
    pub fn new(ctrl: Controller, slave_addr: u8) -> I2c {
        let i2c = I2c { ctrl };
        i2c.init(slave_addr);
        i2c
    }
    const fn reg(&self, off: usize) -> *mut usize {
        (self.ctrl.base() + off) as *mut usize
    }
    pub unsafe fn write(&self, off: usize, val: usize) {
        unsafe { self.reg(off).write_volatile(val) }
    }
    pub unsafe fn read(&self, off: usize) -> usize {
        unsafe { self.reg(off).read_volatile() }
    }

    fn init(&self, slave_addr: u8) {
        unsafe {
            self.write(i2c::ENABLE, 0);
            let conf = self.read(i2c::CON);

            const CON: usize = (1 << 0)   // master mode
                             | (1 << 1)   // standard speed  (bits [2:1] = 0b01)
                           //| (0 << 4)   // 7-bit master addressing
                             | (1 << 6); // slave mode disabled

            self.write(i2c::CON, CON);
            self.write(i2c::TAR, slave_addr as usize);
            self.write(i2c::ENABLE, 1);
            while self.read(i2c::ENABLE_STATUS) & 1 == 0 {
                core::hint::spin_loop();
            }
        }
    }
    /// Send all the bytes in array and end with a stop condition
    pub fn send(&self, bytes: &[u8]) {
        let mut byte_iter = bytes.iter();
        let Some(&last) = byte_iter.next_back() else {
            return;
        };
        for &byte in byte_iter {
            let opts = byte as usize;
            unsafe { self.write(i2c::DATA_CMD, opts) }
        }
        let stop: usize = 1 << 9; // the stop condition
        let opts = stop | last as usize;
        unsafe { self.write(i2c::DATA_CMD, opts) }
        //self.flush();
    }
    /// Read a single byte from the FIFO RX queue
    ///
    /// Ideally, we split this into a 'set read' and 'read bytes'
    /// function
    ///
    /// I'm not using this curently yet though
    pub fn next(&self) -> u8 {
        unsafe {
            // set that I want to read
            self.write(i2c::DATA_CMD, 1 << 8);
            // read
            (self.read(i2c::DATA_CMD) & 0xFF) as u8
        }
    }
    fn flush(&self) {
        loop {
            let status = unsafe { self.read(i2c::STATUS) };
            let txflr = unsafe { self.read(i2c::TXFLR) };
            // TFE (bit 2) set, MST_ACTIVITY (bit 5) clear, FIFO empty
            if status & (1 << 2) != 0 && status & (1 << 5) == 0 && txflr == 0 {
                break;
            }
            core::hint::spin_loop();
        }
    }
}
