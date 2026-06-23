use crate::gpio::{self, Gpio, I2cBank, I2cSclPin, I2cSdaPin};
use crate::register::i2c;
use crate::startup::startup::unreset;
use crate::util::Mutex;
use core::marker::PhantomData;

static I2C0_TAKEN: Mutex<bool> = Mutex::new(false);
static I2C1_TAKEN: Mutex<bool> = Mutex::new(false);

pub struct Controller<C: I2cBank> {
    _bank: PhantomData<C>,
}

impl Controller<gpio::I2c0> {
    pub fn take() -> Option<Self> {
        let mut g = I2C0_TAKEN.lock();
        if *g {
            return None;
        }
        *g = true;
        unreset(1 << 3); // RESETS bit 3 = i2c0
        Some(Controller { _bank: PhantomData })
    }
}

impl Controller<gpio::I2c1> {
    pub fn take() -> Option<Self> {
        let mut g = I2C0_TAKEN.lock();
        if *g {
            return None;
        }
        *g = true;
        unreset(1 << 4); // RESETS bit 4 = i2c1
        Some(Controller { _bank: PhantomData })
    }
}

pub struct I2c<C: I2cBank> {
    ctrl: Controller<C>,
}
impl<C: I2cBank> I2c<C> {
    pub fn new<const N: usize, const B: usize, T, S>(
        ctrl: Controller<C>,
        slave_addr: u8,
        sda: Gpio<B, S>,
        scl: Gpio<N, T>,
    ) -> I2c<C>
    where
        Gpio<N, T>: I2cSclPin<Bank = C>,
        Gpio<B, S>: I2cSdaPin<Bank = C>,
    {
        let mut sda = sda.i2c();
        let mut scl = scl.i2c();
        let opts = 1 << 1 // schmitt enable
                 | 1 << 3 // pull up enable
                 | 1 << 6; // input enable
        sda.pad_ctrl(opts);
        scl.pad_ctrl(opts);
        let i2c = I2c { ctrl };
        i2c.init(slave_addr)
    }
    const fn reg(&self, off: usize) -> *mut usize {
        (C::BASE + off) as *mut usize
    }
    pub unsafe fn write(&mut self, off: usize, val: usize) {
        unsafe { self.reg(off).write_volatile(val) }
    }
    pub unsafe fn read(&mut self, off: usize) -> usize {
        unsafe { self.reg(off).read_volatile() }
    }

    fn init(mut self, slave_addr: u8) -> I2c<C> {
        unsafe {
            self.write(i2c::ENABLE, 0);

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
        self
    }
    /// Send all the bytes in array and end with a stop condition
    pub fn send(&mut self, bytes: &[u8]) {
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
    pub fn next(&mut self) -> u8 {
        unsafe {
            // set that I want to read
            self.write(i2c::DATA_CMD, 1 << 8);
            // read
            (self.read(i2c::DATA_CMD) & 0xFF) as u8
        }
    }
    fn flush(&mut self) {
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
