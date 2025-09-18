#!/usr/bin/env python3
#
# SPDX-FileCopyrightText: 2025 Shomy
# SPDX-License-Identifier: AGPL-3.0-or-later
#
from parse_da import DAFile, DAType, DA, DAEntryRegion

if __name__ == "__main__":
    import sys
    if len(sys.argv) != 5:
        print(f"Usage: {sys.argv[0]} <donor_da> <da1.bin> <da2.bin> <output_da>")
        sys.exit(1)

    donor_da_path = sys.argv[1]
    da_paths = sys.argv[2:-1]
    output_da_path = sys.argv[-1]

    with open(donor_da_path, "rb") as f:
        donor_da_raw = f.read()
    
    donor_da_file = DAFile.parse_da(donor_da_raw)

    with open(da_paths[0], "rb") as f:
        da1_raw = f.read()

    with open(da_paths[1], "rb") as f:
        da2_raw = f.read()

    da = donor_da_file.das[0]
        
    da1_len = len(da1_raw)
    da2_len = len(da2_raw)

    original_da1 = da.get_da1()
    original_da2 = da.get_da2()

    original_da1.data = da1_raw + b'\x00' * original_da1.sig_len
    original_da2.data = da2_raw + b'\x00' * original_da2.sig_len

    original_da1.length = da1_len + original_da1.sig_len
    original_da2.length = da2_len + original_da2.sig_len

    original_da1.region_offset = da1_len
    original_da2.region_offset = da2_len

    header_end = min(r.offset for r in da.regions)
    current_offset = header_end

    for region in da.regions:
        region.offset = current_offset
        region.length = len(region.data)
        current_offset += region.length

    with open(output_da_path, "wb") as out:
        out.write(donor_da_raw[:header_end])

        for region in da.regions:
            out.seek(region.offset)
            out.write(region.data)

    print(f"Updated DA file written to: {output_da_path}")
    print("Updated region offsets:")
    for i, region in enumerate(da.regions):
        print(f"  Region {i}: offset=0x{region.offset:08X}, length=0x{region.length:08X}")

