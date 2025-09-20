#!/usr/bin/env python3
#
# SPDX-FileCopyrightText: 2025 Shomy
# SPDX-License-Identifier: AGPL-3.0-or-later
#
import sys
import struct
from typing import Dict, List, Optional, Union

# SEJ contants (see https://github.com/shomykohai/penumbra/blob/main/src/core/crypto/sej.rs#L86)
SEJ_CONSTANTS = [
    0x9ED40400,
    0x0E884A1,
    0xE3F083BD,
    0x2F4E6D8A,
    0x40000002
]

PATTERNS = [struct.pack("<I", k) for k in SEJ_CONSTANTS]

def find_sej_constant_pos(data) -> dict:
    offsets = {}
    for const, pattern in zip(SEJ_CONSTANTS, PATTERNS):
        pos = 0
        while True:
            idx = data.find(pattern, pos)
            if idx == -1:
                break
            if const not in offsets:
                offsets[const] = []
            offsets[const].append(idx)
            pos = idx + 1
    return offsets

# To be sure we're looking exactly at the SEJ related constants,
# we check if we find them close to each other, as well as 
# having at least a minimum number of constants in that cluster.
def cluster_constants(offsets, max_gap=0x10, min_k=3) -> List[Dict[int, int]]:
    offset_sorted = sorted((offset, val) for val, lst in offsets.items() for offset in lst)

    clusters = []
    current = []
    unique_vals = set()
    last_offset = None

    for offset, val in offset_sorted:
        if last_offset is None or offset - last_offset <= max_gap:
            current.append((val, offset))
            unique_vals.add(val)
        else:
            if len(unique_vals) >= min_k:
                cluster_dict = {val: off for val, off in current}
                clusters.append(cluster_dict)

            current = [(val, offset)]
            unique_vals = {val}

        last_offset = offset

    if current and len(unique_vals) >= min_k:
        cluster_dict = {val: off for val, off in current}
        clusters.append(cluster_dict)

    return clusters


def find_ldr_addr(data, ref_offset, value: Optional[int] = None, reg_to_find: Optional[int] = None) -> Optional[Union[int, int, int]]:
    # Search backwards for the LDR instruction
    # 0x100 is an arbitrary size that has no special meaning :3
    for off in range(ref_offset - 2, ref_offset - 0x100, -2):
        instruction = struct.unpack_from("<H", data, off)[0]
        # Some notes so that I don't forget how this works:
        # Let's say we decode 19 49 (taken from Ghidra, lamu da2):
        # In LE this translates to 0x4919, which in binary is 
        # 0100 1001 0001 1001
        # The top 5 bits (01001) identify the LDR instruction.
        # The following 3 bits (001) indicate the destination register, in this case R1.
        # The rest of the bits (0011001) gives us the offset (in words) from PC.
        # In ARM, a word is 4 bytes, thus we multiply this by 4 to get the byte offset.
        # 
        # To identify if it's an LDR instruction, we can just mask the top 5 bits to
        # see if it matches. So, we transform 1111100000000000 to hex: 0xF800
        # This leaves us with a *template* for the LDR instruction (01001 000 00000000)
        # which is 0x4800 in hex.
        if (instruction & 0xF800) == 0x4800:
            # Extract lower 8 bits from the instruction to find the word offset
            word_off = instruction & 0xFF
            # Arm PC is always 4 bytes (or 2 instructions) ahead apparently :D,
            # At least, this makes it show the same values as Ghidra.
            pc = off + 4
            addr = pc + (word_off * 4)
            found_ldr_val = struct.unpack_from("<I", data, addr)[0]

            # 0100100100011001 becomes 01001001, then mask it with 00000111
            # to get the register (001 = R1) 
            register = (instruction >> 8) & 0x7
            if value is not None and found_ldr_val == value:
                return found_ldr_val, off, register
            if reg_to_find is not None and register == reg_to_find:
                return found_ldr_val, off, register
    return None, None, None


def find_str_addr(data, start_off, ldr_reg) -> Optional[Union[int, int, int]]:
    for off in range(start_off + 2, start_off + 0x100, 2):
        instruction = struct.unpack_from("<H", data, off)[0]
        # A minimal STR instructions looks like 01100 00000 000 000
        # 01100 is STR
        # 00000 is the offset 
        # 000 is the dest register
        # 000 is the source register
        # so, for example: 0110000000011001
        # becomes: 01100 00000 011 001
        # which translates to STR R1, [R3, #0]
        if (instruction & 0xF800) == 0x6000:
            src_reg = instruction & 0x7
            if src_reg == ldr_reg:
                # Shift 6 bits to remove the registers, then 00011111 mask
                word_off = (instruction >> 6) & 0x1F
                dest_reg = (instruction >> 3) & 0x7
                offset = word_off * 4

                return dest_reg, off, offset
                
    return None, None, None

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <something-that-implements-sej>")
        sys.exit(1)

    with open(sys.argv[1], "rb") as f:
        data = f.read()

    offsets = find_sej_constant_pos(data)
    if not offsets:
        print("No SEJ related constants found.")
        sys.exit(1)

    # for const, offs in offsets.items():
    for i, (const, offs) in enumerate(offsets.items()):
        if offs:
            print(f"Constant {i+1} (0x{const:X}) found at offsets: {', '.join(f'0x{o:X}' for o in offs)}")

    print("="*60)
    clusters = cluster_constants(offsets)
    if not clusters:
        print("No clusters of constants found.")
        sys.exit(1)

    print(f"Found {len(clusters)} cluster.")
    for idx, cluster in enumerate(clusters):
        print(f"Cluster {idx + 1}:")
        for val, off in cluster.items():
            print(f"  - Constant 0x{val:X} at offset 0x{off:X}")

    print("="*60)

    for idx, cluster in enumerate(clusters):
        ref_offset = min(cluster.values())

        # Get in which register the first SEJ constant is loaded, as well as the offset in the file of
        # the LDR instruction. 
        loaded_value, ldr_start_off, dst_reg = find_ldr_addr(data, ref_offset, value=SEJ_CONSTANTS[0])
        if loaded_value != SEJ_CONSTANTS[0] or loaded_value is None:
            print("Could not find LDR loading SEJ constant.")
            continue
        
        print(f"Cluster {idx + 1}: Found LDR loading SEJ constant 0x{loaded_value:X} at offset 0x{ldr_start_off:X} into R{dst_reg}")
        str_dest_reg, str_off, offset = find_str_addr(data, ldr_start_off, dst_reg)
        if str_dest_reg is None:
            print("Could not find STR base register.")
            continue
        
        print(f"Cluster {idx + 1}: Found STR using base register R{str_dest_reg} at offset 0x{str_off:X} with offset {offset}")

        const_dest_addr, ldr_base_off, _ = find_ldr_addr(data, str_off, reg_to_find=str_dest_reg)
        if const_dest_addr is None:
            print("Could not find LDR loading address into STR base register.")
            continue
        
        print(f"Cluster {idx + 1}: Found LDR loading address 0x{const_dest_addr:X} at offset 0x{ldr_base_off:X} into R{str_dest_reg} ")

        aligned_base = const_dest_addr & ~0xFF
        print(f"SEJ base: 0x{aligned_base:X}")