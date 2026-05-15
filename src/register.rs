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
}
pub mod flash {
    pub const XIP_BASE: usize = 0x10000000;
}
