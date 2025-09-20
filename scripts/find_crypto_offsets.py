#!/usr/bin/env python3
import sys
import struct
from typing import List, Dict

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


## TODO: Improve this to find the actual LDR instruction and not the first one it finds
def find_ldr_addr(data, ref_offset):
    # Search backwards for the LDR instruction
    # 0x40 is an arbitrary size that has no special meaning :3
    for off in range(ref_offset - 2, ref_offset - 0x40, -2):
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
            found_ldr = struct.unpack_from("<I", data, addr)[0]
            return found_ldr, off
    return None, None


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
        # The sej function I'm using as a reference to finding the sej_base
        # has as the last LDR address with something like:
        #   _DAT_1000a00c = 0x10;
        # So we just find the LDR istruction that loads _DAT_1000a00c to then
        # remove the last two numbers (0x0c) to get the base.
        ref_offset = min(cluster.values())
        base_addr, ldr_off = find_ldr_addr(data, ref_offset)

        if base_addr is None:
            print(f"Could not determine LDR base near cluster {i+1}.")
            continue

        print(f"Found ldr base address at offset 0x{ldr_off:X}: 0x{base_addr:X}")

        aligned_base = base_addr & ~0xFF
        print(f"SEJ base: 0x{aligned_base:X}")