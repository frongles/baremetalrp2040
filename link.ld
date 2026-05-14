
MEMORY
{
    XIP (rx) : ORIGIN = 0x10000000, LENGTH = 2048k
    SRAM (rwx) : ORIGIN = 0x20000000, LENGTH = 256k
}

SECTIONS
{
    .text : 
    {
        KEEP(*(.boot2*))
        *(.text*)
    } > XIP
}
