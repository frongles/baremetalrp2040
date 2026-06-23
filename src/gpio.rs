use crate::{register, startup::startup::unreset, util::Mutex};
use core::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

pub trait PinState: sealed::Sealed {}

pub trait SpiRx: sealed::Sealed {}
pub trait SpiCsn: sealed::Sealed {}
pub trait SpiTx: sealed::Sealed {}
pub trait Spi0: sealed::Sealed {}
pub trait Spi1: sealed::Sealed {}
pub trait SpiPin: sealed::Sealed {}

pub trait UartTx: sealed::Sealed {}
pub trait UartRx: sealed::Sealed {}
pub trait UartCts: sealed::Sealed {}
pub trait UartRts: sealed::Sealed {}
pub trait Uart0: sealed::Sealed {}
pub trait Uart1: sealed::Sealed {}
pub trait UartPin: sealed::Sealed {}

pub trait I2cBank: sealed::Sealed {
    const BASE: usize;
}

pub trait I2cSdaPin {
    type Bank;
}
pub trait I2cSclPin {
    type Bank;
}

pub trait I2cPin: sealed::Sealed {}

pub trait SioPin: sealed::Sealed {}

pub trait PwmPin: sealed::Sealed {}

pub enum Unallocated {}

pub enum Spi {}
pub enum Uart {}
pub enum I2cSda {}
pub enum I2cScl {}
pub enum I2c {}
pub enum I2c0 {}
impl I2cBank for I2c0 {
    const BASE: usize = 0x40044000;
}
pub enum I2c1 {}
impl I2cBank for I2c1 {
    const BASE: usize = 0x40048000;
}
impl sealed::Sealed for I2c0 {}
impl sealed::Sealed for I2c1 {}
pub enum Pwm {}

pub enum Sio {}
enum Pio0 {}
enum Pio1 {}
enum Usb {}

impl SioPin for Sio {}
impl sealed::Sealed for Sio {}

impl I2cPin for I2c {}
impl sealed::Sealed for I2c {}

impl UartPin for Uart {}
impl sealed::Sealed for Uart {}

impl PwmPin for Pwm {}
impl sealed::Sealed for Pwm {}

static TAKEN: Mutex<bool> = Mutex::new(false);

pub struct Gpio<const N: usize, T> {
    _phantom: PhantomData<T>,
}

pub struct Pins {
    pub gpio0: Gpio<0, Unallocated>,
    pub gpio1: Gpio<1, Unallocated>,
    pub gpio2: Gpio<2, Unallocated>,
    pub gpio3: Gpio<3, Unallocated>,
    pub gpio4: Gpio<4, Unallocated>,
    pub gpio5: Gpio<5, Unallocated>,
    pub gpio6: Gpio<6, Unallocated>,
    pub gpio7: Gpio<7, Unallocated>,
    pub gpio8: Gpio<8, Unallocated>,
    pub gpio9: Gpio<9, Unallocated>,
    pub gpio10: Gpio<10, Unallocated>,
    pub gpio11: Gpio<11, Unallocated>,
    pub gpio12: Gpio<12, Unallocated>,
    pub gpio13: Gpio<13, Unallocated>,
    pub gpio14: Gpio<14, Unallocated>,
    pub gpio15: Gpio<15, Unallocated>,
    pub gpio16: Gpio<16, Unallocated>,
    pub gpio17: Gpio<17, Unallocated>,
    pub gpio18: Gpio<18, Unallocated>,
    pub gpio19: Gpio<19, Unallocated>,
    pub gpio20: Gpio<20, Unallocated>,
    pub gpio21: Gpio<21, Unallocated>,
    pub gpio22: Gpio<22, Unallocated>,
    pub gpio23: Gpio<23, Unallocated>,
    pub gpio24: Gpio<24, Unallocated>,
    pub gpio25: Gpio<25, Unallocated>,
    pub gpio26: Gpio<26, Unallocated>,
    pub gpio27: Gpio<27, Unallocated>,
    pub gpio28: Gpio<28, Unallocated>,
    pub gpio29: Gpio<29, Unallocated>,
}

impl Pins {
    /// Consumes ownership of the pad bank exactly once. After this call,
    /// there is no way to construct another Gpio<5> — the only `Gpio<5>`
    /// in existence lives inside this `Pins` instance, moved out into
    /// whatever owns it next.
    pub fn take() -> Option<Self> {
        {
            let mut guard = TAKEN.lock();
            if *guard {
                return None; // already taken — caller must handle,
                // no aliasing possible
            }
            *guard = true
        }
        // not taken, so we can take the gpio bank out of reset
        unreset(1 << 5); // unreset io bank
        unreset(1 << 8); // unreset pad bank

        Some(Pins {
            gpio0: Gpio {
                _phantom: PhantomData,
            },
            gpio1: Gpio {
                _phantom: PhantomData,
            },
            gpio2: Gpio {
                _phantom: PhantomData,
            },
            gpio3: Gpio {
                _phantom: PhantomData,
            },
            gpio4: Gpio {
                _phantom: PhantomData,
            },
            gpio5: Gpio {
                _phantom: PhantomData,
            },
            gpio6: Gpio {
                _phantom: PhantomData,
            },
            gpio7: Gpio {
                _phantom: PhantomData,
            },
            gpio8: Gpio {
                _phantom: PhantomData,
            },
            gpio9: Gpio {
                _phantom: PhantomData,
            },
            gpio10: Gpio {
                _phantom: PhantomData,
            },
            gpio11: Gpio {
                _phantom: PhantomData,
            },
            gpio12: Gpio {
                _phantom: PhantomData,
            },
            gpio13: Gpio {
                _phantom: PhantomData,
            },
            gpio14: Gpio {
                _phantom: PhantomData,
            },
            gpio15: Gpio {
                _phantom: PhantomData,
            },
            gpio16: Gpio {
                _phantom: PhantomData,
            },
            gpio17: Gpio {
                _phantom: PhantomData,
            },
            gpio18: Gpio {
                _phantom: PhantomData,
            },
            gpio19: Gpio {
                _phantom: PhantomData,
            },
            gpio20: Gpio {
                _phantom: PhantomData,
            },
            gpio21: Gpio {
                _phantom: PhantomData,
            },
            gpio22: Gpio {
                _phantom: PhantomData,
            },
            gpio23: Gpio {
                _phantom: PhantomData,
            },
            gpio24: Gpio {
                _phantom: PhantomData,
            },
            gpio25: Gpio {
                _phantom: PhantomData,
            },
            gpio26: Gpio {
                _phantom: PhantomData,
            },
            gpio27: Gpio {
                _phantom: PhantomData,
            },
            gpio28: Gpio {
                _phantom: PhantomData,
            },
            gpio29: Gpio {
                _phantom: PhantomData,
            },
        })
    }
}

impl<const N: usize, T: SioPin> Gpio<N, T> {
    pub fn output_enable(&self) {
        let sio_oe = register::sio::gpio::OE_SET as *mut usize;
        unsafe {
            sio_oe.write_volatile(1 << N);
        }
    }
    pub fn set(&self) {
        let sio_out = register::sio::gpio::OUT_SET as *mut usize;
        unsafe {
            sio_out.write_volatile(1 << N);
        }
    }
    pub fn flip(&self) {
        let xor = register::sio::gpio::OUT_XOR as *mut usize;
        unsafe {
            xor.write_volatile(1 << N);
        }
    }
    pub fn clear(&self) {
        let sio_out = register::sio::gpio::OUT_CLR as *mut usize;
        unsafe {
            sio_out.write_volatile(1 << N);
        }
    }
}

impl<const N: usize, T> Gpio<N, T> {
    /// Configure thig gpio to use the SPI function
    pub fn spi(self) -> Gpio<N, Spi> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(1) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the UART function
    pub fn uart(self) -> Gpio<N, Uart> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(2) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the i2c function
    pub fn i2c(self) -> Gpio<N, I2c> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(3) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the pwm function
    pub fn pwm(self) -> Gpio<N, Pwm> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(4) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the sio function
    pub fn sio(self) -> Gpio<N, Sio> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(5) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the pio0 function
    pub fn pio0(self) -> Gpio<N, Pio0> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(6) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the pio1 function
    pub fn pio1(self) -> Gpio<N, Pio1> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(7) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Configure thig gpio to use the usb function
    pub fn usb(self) -> Gpio<N, Usb> {
        let ctrl = register::gpio::ctrl(N) as *mut usize;
        unsafe { ctrl.write_volatile(9) }
        Gpio {
            _phantom: PhantomData,
        }
    }
    /// Set pad options for this GPIO pin
    /// bit0: slewrate, slow=0, fast=1
    /// bit1: schmitt trigger enable
    /// bit2: Pull down enable
    /// bit3: Pull up enable
    /// bit45: Drive strength
    ///     0=2milliamps
    ///     1=4ma
    ///     2=8ma
    ///     3=12ma
    /// bit6: Input enable
    /// bit7: Output disable
    pub fn pad_ctrl(&mut self, opts: u8) {
        let pad_ctrl = register::gpio::pad_ctrl(N) as *mut usize;
        unsafe { pad_ctrl.write_volatile(opts as usize) }
    }
}

impl<T> I2cSdaPin for Gpio<0, T> {
    type Bank = I2c0;
}
impl<T> sealed::Sealed for Gpio<0, T> {}

impl<T> I2cSclPin for Gpio<1, T> {
    type Bank = I2c0;
}
impl<T> sealed::Sealed for Gpio<1, T> {}
