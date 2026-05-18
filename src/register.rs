pub mod reset {
    pub const RESET: usize = 0x4000c000;
    pub const RESET_DONE: usize = 0x4000c008;
}
pub mod gpio {
    pub const IO_BANK0_GPIO25_CTRL: usize = 0x400140cc;
    pub const OE_SET: usize = 0xd0000024;
    pub const OUT_XOR: usize = 0xd000001c;
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
    pub const TIMEHR: usize = OFFSET + 0x08;
    pub const TIMELR: usize = OFFSET + 0x0C;
    /// Interrupt enable
    pub const INTR: usize = OFFSET + 0x34;
    pub const INTE: usize = OFFSET + 0x38;
}

pub mod watchdog {
    pub const OFFSET: usize = 0x40058000;
    pub const TICK: usize = OFFSET + 0x2c;
}

pub mod xosc {
    pub const OFFSET: usize = 0x40024000;
    pub const CTRL: usize = OFFSET + 0x00;
    pub const STATUS: usize = OFFSET + 0x04;
}

pub mod clock {
    pub const OFFSET: usize = 0x40008000;
    pub const REF_CTRL: usize = OFFSET + 0x30;
    pub const REF_DIV: usize = OFFSET + 0x34;
    pub const REF_SELECTED: usize = OFFSET + 0x38;
    pub const SYS_CTRL: usize = OFFSET + 0x3c;
    pub const SYS_DIV: usize = OFFSET + 0x40;
    pub const SYS_SELECTED: usize = OFFSET + 0x44;
}
pub mod pll_sys {
    pub const OFFSET: usize = 0x40028000;
    pub const CS: usize = OFFSET + 0x0;
    pub const PWD: usize = OFFSET + 0x4;
    pub const FBDIV_INT: usize = OFFSET + 0x8;
    pub const PRIM: usize = OFFSET + 0xc;
}
