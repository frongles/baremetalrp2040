
MEMORY
{
    XIP (rx) : ORIGIN = 0x10000000, LENGTH = 2048k
    SRAM (rwx) : ORIGIN = 0x20000000, LENGTH = 256k
}

SECTIONS
{
    .boot2 : 
    {
        _sboot2 = .;
        KEEP(*(.boot2*))
        _eboot2 = .;
        . = . + (252 - (_eboot2 - _sboot2));
    } > XIP

    .text :
    {
        . = ORIGIN(XIP) + 256;
        _svtable = .;
        KEEP(*(.vector*))
        KEEP(*(.text*))
    } > XIP

    .stack (NOLOAD) :
    { 
        . = ORIGIN(SRAM) + LENGTH(SRAM);
        _sstack = .;
    } > SRAM


    /DISCARD/ : {
        *(.ARM.exidx*)
        *(.ARM.extab*)
    }
}
