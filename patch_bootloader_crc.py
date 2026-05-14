#!/usr/bin/env python3
"""
Bootloader CRC Patcher
======================
Reads the first 256 bytes of a binary file (the bootloader region),
pads to 252 bytes if necessary, calculates a CRC-32/MPEG-2 over those
252 bytes, writes the 4-byte CRC (big-endian) into bytes [252..255]
(i.e. the 253rd–256th bytes, 0-indexed 252–255), then saves the result.

Usage:
    python patch_bootloader_crc.py <input_file> [output_file]

If output_file is omitted the input file is overwritten in-place.
"""

import sys
import struct
import argparse

# ---------------------------------------------------------------------------
# CRC-32/MPEG-2
#   Poly  : 0x04C11DB7
#   Init  : 0xFFFFFFFF
#   RefIn : False
#   RefOut: False
#   XorOut: 0x00000000
# ---------------------------------------------------------------------------
POLY = 0x04C11DB7
INIT = 0xFFFFFFFF

def _build_table() -> list[int]:
    table = []
    for i in range(256):
        crc = i << 24
        for _ in range(8):
            if crc & 0x80000000:
                crc = ((crc << 1) ^ POLY) & 0xFFFFFFFF
            else:
                crc = (crc << 1) & 0xFFFFFFFF
        table.append(crc)
    return table

_TABLE = _build_table()


def crc32_mpeg2(data: bytes) -> int:
    """Return the CRC-32/MPEG-2 of *data* as an unsigned 32-bit integer."""
    crc = INIT
    for byte in data:
        crc = ((crc << 8) & 0xFFFFFFFF) ^ _TABLE[(crc >> 24) ^ byte]
    return crc


# ---------------------------------------------------------------------------
# Main logic
# ---------------------------------------------------------------------------
BOOTLOADER_SIZE = 256   # total bootloader region in bytes
CRC_OFFSET      = 252   # CRC starts at byte index 252 (0-based)
CRC_SIZE        = 4     # 4-byte CRC
DATA_SIZE       = CRC_OFFSET  # bytes covered by the CRC (0..251)


def patch(input_path: str, output_path: str) -> None:
    # --- read file ----------------------------------------------------------
    with open(input_path, "rb") as f:
        file_data = bytearray(f.read())

    print(f"[*] Read {len(file_data)} bytes from '{input_path}'")

    # --- work on a mutable bootloader region --------------------------------
    # Extract (or create) the first BOOTLOADER_SIZE bytes
    if len(file_data) >= BOOTLOADER_SIZE:
        boot_region = bytearray(file_data[:BOOTLOADER_SIZE])
    else:
        # File is shorter than 256 bytes – extend with zeros
        boot_region = bytearray(file_data) + bytearray(BOOTLOADER_SIZE - len(file_data))
        print(f"[*] File shorter than {BOOTLOADER_SIZE} bytes; padded to {BOOTLOADER_SIZE} bytes")

    # --- zero-pad data region if needed ------------------------------------
    # Bytes 0..251 are the CRC input; ensure they exist (already guaranteed
    # above, but be explicit for clarity).
    data_region = boot_region[:DATA_SIZE]   # first 252 bytes
    # (already length 252 because boot_region is exactly 256 bytes)

    # --- calculate CRC ------------------------------------------------------
    crc_value = crc32_mpeg2(data_region)
    crc_bytes  = struct.pack(">I", crc_value)   # big-endian 4 bytes

    print(f"[*] CRC-32/MPEG-2 over bytes [0..{DATA_SIZE-1}] = 0x{crc_value:08X}")

    # --- insert CRC into boot region ----------------------------------------
    boot_region[CRC_OFFSET : CRC_OFFSET + CRC_SIZE] = crc_bytes
    print(f"[*] Written CRC bytes {list(crc_bytes)} into offsets [{CRC_OFFSET}..{CRC_OFFSET+CRC_SIZE-1}]")

    # --- reassemble the full file -------------------------------------------
    if len(file_data) >= BOOTLOADER_SIZE:
        patched = boot_region + file_data[BOOTLOADER_SIZE:]
    else:
        patched = boot_region   # file was entirely within the boot region

    # --- save ---------------------------------------------------------------
    with open(output_path, "wb") as f:
        f.write(patched)

    print(f"[*] Saved {len(patched)} bytes to '{output_path}'")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Patch CRC-32/MPEG-2 into bytes 253-256 of a bootloader binary."
    )
    parser.add_argument("input",  help="Path to the binary file to patch")
    parser.add_argument("output", nargs="?", help="Output path (default: overwrite input)")
    args = parser.parse_args()

    output = args.output or args.input
    patch(args.input, output)


if __name__ == "__main__":
    main()
