#!/bin/bash
set -eu

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
converter="$repo_dir/tools/uf2/utils/uf2conv.py"

if [[ ! -f "$converter" ]]; then
    echo "UF2 converter missing. Run: git submodule update --init --recursive" >&2
    exit 1
fi

elf="$1"
arm-none-eabi-objcopy -O binary "$elf" "$elf.bin"
python3 "$repo_dir/patch_bootloader_crc.py" "$elf.bin" "$elf.bin"
python3 "$converter" -b 0x10000000 -f 0xe48bff56 "$elf.bin" -o "$elf.uf2"

