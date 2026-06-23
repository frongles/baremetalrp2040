pub mod reset {
    pub const RESET: usize = 0x4000c000;
    pub const RESET_DONE: usize = 0x4000c008;
}
/// Single-cyle IO block
///
/// Contains a bunch of stuff
/// - spinlocks
/// - gpio controls (gpio config is in [`gpio`]
/// - Inter-core FIFOs
pub mod sio {
    const OFFSET: usize = 0xd0000000;
    const SPINLOCK_OFFSET: usize = OFFSET + 0x100;
    /// Get the register address for the `n`th spinlock
    ///
    /// Reading from a spinlock will:
    /// - return 0 if it's locked
    /// - otherwise return non zero, and claim the lock
    ///
    /// Writing any value releases the lock
    ///
    /// There are 32 spinlocks
    #[inline(always)]
    pub const fn spinlock(n: usize) -> usize {
        SPINLOCK_OFFSET + 4 * n
    }

    /// SIO gpio registers
    ///
    /// Registers for enabling and writing to GPIO
    pub mod gpio {
        use super::*;

        /// Read returns last value written (not input value on pins)
        /// Write sets output level (1/0 -> high/low)
        ///
        /// For atomic OR write, go to [`OUT_SET`]
        pub const OUT: usize = OFFSET + 0x10;
        /// Atomic bitwise OR
        ///
        /// GPIO_OUT |= wdata (atomic)
        pub const OUT_SET: usize = OFFSET + 0x14;
        /// Atomic bit-clear
        ///
        /// GPIO_OUT &= ~wdata
        pub const OUT_CLR: usize = OFFSET + 0x18;
        /// Atomic bitwise OR
        ///
        /// GPIO_OUT ^= wdata
        pub const OUT_XOR: usize = OFFSET + 0x1c;
        /// Output enable register
        ///
        /// Enable GPIO n as output for bit n up to 29
        pub const OE: usize = OFFSET + 0x20;
        /// Atomic bitset on [`OE`]
        pub const OE_SET: usize = OFFSET + 0x24;
    }
}

/// User bank io register addresss
///
/// These registers are primarily for *configuring* the gpio pins, which
/// must be done before using them.
/// For enabling output and writing to the gpio, see [`sio::gpio`].
/// Note that you can override output values from the registers in this module,
/// but it's better to write in a single cycle with the accesses from the [`sio`]
/// module
pub mod gpio {
    const USER_BANK_IO_OFFSET: usize = 0x4001_4000;
    const PAD_BANK_OFFSET: usize = 0x4001c000;
    /// Returns the memory address of GPIOn_STATUS register
    #[inline(always)]
    pub const fn status(n: usize) -> usize {
        USER_BANK_IO_OFFSET + 8 * n
    }

    /// Returns the memory address of GPIOn_CTRL register
    ///
    /// The most important field is the `FUNC_SELECT` field in bits[0..=4]
    /// See section 2.19.2 in the rp2040 docs
    ///
    /// Each GPIO can have 1 function at a time. Likewise, each peripheral input
    /// (e.g. UART0 RX) should only be selected on one GPIO at a time
    #[inline(always)]
    pub(crate) const fn ctrl(n: usize) -> usize {
        USER_BANK_IO_OFFSET + 8 * n + 4
    }

    /// Returns the memory address of PADS_BANK GPIOn register
    #[inline(always)]
    pub const fn pad_ctrl(n: usize) -> usize {
        PAD_BANK_OFFSET + 8 * n + 4
    }

    /// Set the nth GPIO pin to be I2C
    ///
    /// Read section 2.19.2 of the data sheet to see if the nth GPIO pin
    /// is a I2C0_SDA, I2C0_SCL, I2C1_SDA, or I2C1_SCL
    #[inline(always)]
    pub fn set_i2c(n: usize) {
        // should probably read then re write here...
        let ctrl = ctrl(n) as *mut usize;
        unsafe { ctrl.write_volatile(3) }
    }
    /// Set nth GPIO to be configured as PWM output
    ///
    /// Take care when setting pwm. There are 16 pwm channels shared
    /// between 29 gpio pins.
    ///
    /// pins 0-13 and 16-19 share channels 0A, 0B, 1A, 1B... 6A, 6B
    /// i.e. pin 0 and pin 16 are both on channel 0A, and so on.
    ///
    /// Pins 14 and 15 have channels 7A and 7B to themself
    #[inline(always)]
    pub fn set_pwm(n: usize) {
        // should probably read then re write here...
        let ctrl = ctrl(n) as *mut usize;
        unsafe { ctrl.write_volatile(4) }
    }
    /// Set nth GPIO to be configured as SIO single cycle IO
    ///
    /// Write to the pin with [`super::sio::gpio`] registers
    #[inline(always)]
    pub fn set_sio(n: usize) {
        // should probably read then re write here...
        let ctrl = ctrl(n) as *mut usize;
        unsafe { ctrl.write_volatile(5) }
    }
}
pub mod ssi {
    pub const OFFSET: usize = 0x18000000;
    pub const CTRL0: usize = 0x18000000;
    pub const SSIENR: usize = OFFSET + 0x008;
    pub const BAUDR: usize = OFFSET + 0x014;
    pub const SR: usize = OFFSET + 0x028;
    pub const DR0: usize = OFFSET + 0x060;
    pub const SPI_CTRL0: usize = OFFSET + 0x0f4;
}
pub mod m0plus {
    pub const OFFSET: usize = 0xe0000000;
    pub const VTOR: usize = OFFSET + 0xed08;
    /// Nested Vector Interrupt Controller
    /// Interrupt Set Enable Register
    pub const NVIC_ISER: usize = OFFSET + 0xe100;
}
pub mod flash {
    pub const XIP_BASE: usize = 0x10000000;
}

pub mod time {
    pub const OFFSET: usize = 0x40054000;
    pub const ALARM0: usize = OFFSET + 0x10;
    pub const ALARM1: usize = OFFSET + 0x14;
    pub const ALARM2: usize = OFFSET + 0x18;
    pub const ALARM3: usize = OFFSET + 0x1c;
    pub const TIMEHR: usize = OFFSET + 0x08;
    pub const TIMELR: usize = OFFSET + 0x0C;
    /// Interrupt enable
    pub const INTR: usize = OFFSET + 0x34;
    pub const INTE: usize = OFFSET + 0x38;
}

pub mod watchdog {
    const OFFSET: usize = 0x40058000;
    pub const TICK: usize = OFFSET + 0x2c;
}

pub mod xosc {
    const OFFSET: usize = 0x40024000;
    pub const CTRL: usize = OFFSET;
    pub const STATUS: usize = OFFSET + 0x04;
}

pub mod clock {
    const OFFSET: usize = 0x40008000;
    pub const REF_CTRL: usize = OFFSET + 0x30;
    pub const REF_DIV: usize = OFFSET + 0x34;
    pub const REF_SELECTED: usize = OFFSET + 0x38;
    pub const SYS_CTRL: usize = OFFSET + 0x3c;
    pub const SYS_DIV: usize = OFFSET + 0x40;
    pub const SYS_SELECTED: usize = OFFSET + 0x44;
}
pub mod pll_sys {
    const OFFSET: usize = 0x40028000;
    pub const CS: usize = OFFSET;
    pub const PWD: usize = OFFSET + 0x4;
    pub const FBDIV_INT: usize = OFFSET + 0x8;
    pub const PRIM: usize = OFFSET + 0xc;
}
pub mod i2c {
    pub const CON: usize = 0x00;
    pub const TAR: usize = 0x04;
    pub const SAR: usize = 0x04;
    pub const DATA_CMD: usize = 0x10;
    pub const SS_SCL_HCNT: usize = 0x14;
    pub const SS_SCL_LCNT: usize = 0x18;
    pub const FS_SCL_HCNT: usize = 0x1c;
    pub const FS_SCL_LCNT: usize = 0x20;
    pub const INTR_STAT: usize = 0x2c;
    pub const INTR_MASK: usize = 0x30;
    pub const RAW_INTR_STAT: usize = 0x34;
    pub const RX_TL: usize = 0x38;
    pub const TX_TL: usize = 0x3c;
    pub const CLR_INTR: usize = 0x40;
    pub const CLR_RX_UNDER: usize = 0x44;
    pub const CLR_RX_OVER: usize = 0x48;
    pub const CLR_RD_REQ: usize = 0x50;
    pub const CLR_TX_ABRT: usize = 0x54;
    pub const CLR_RX_DONE: usize = 0x58;
    pub const CLR_ACTIVITY: usize = 0x5c;
    pub const CLR_STOP_DET: usize = 0x60;
    pub const CLR_START_DET: usize = 0x64;
    pub const CLR_GEN_CALL: usize = 0x68;
    pub const ENABLE: usize = 0x6c;
    pub const STATUS: usize = 0x70;
    pub const TXFLR: usize = 0x74;
    pub const RXFLR: usize = 0x78;
    pub const SDA_HOLD: usize = 0x7c;
    pub const TX_ABRT_SOURCE: usize = 0x80;
    pub const SLV_DATA_NACK_ONLY: usize = 0x84;
    pub const DMA_CR: usize = 0x88;
    pub const DMA_TDLR: usize = 0x8c;
    pub const DMA_RDLR: usize = 0x90;
    pub const SDA_SETUP: usize = 0x94;
    pub const ACK_GENERAL_CALL: usize = 0x98;
    pub const ENABLE_STATUS: usize = 0x9c;
    pub const FS_SPKLEN: usize = 0xa0;
    pub const CLR_RESTART_DET: usize = 0xa8;
    pub const COMP_PARAM_1: usize = 0xf4;
    pub const COMP_VERSION: usize = 0xf8;
    pub const COMP_TYPE: usize = 0xfc;
}
