#!/bin/bash
elf="$1"
arm-none-eabi-objcopy -O binary "$elf" "$elf.bin"
python3 patch_bootloader_crc.py "$elf.bin" "$elf.bin"
python3 /usr/local/lib/uf2/utils/uf2conv.py -b 0x10000000 -f 0xe48bff56 "$elf.bin" -o "$elf.uf2"

