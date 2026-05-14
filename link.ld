
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
        KEEP(*(.main*))
        KEEP(*(.text*))
    } > XIP


    /DISCARD/ : {
        *(.ARM.exidx*)
        *(.ARM.extab*)
    }
}
