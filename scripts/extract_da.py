#!/usr/bin/env python3
#
# SPDX-FileCopyrightText: 2025 Shomy
# SPDX-License-Identifier: AGPL-3.0-or-later
#
from parse_da import DAFile, DAType, DA, DAEntryRegion

if __name__ == "__main__":
    import sys
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <da_file>")
        sys.exit(1)
    
    with open(sys.argv[1], "rb") as f:
        da_raw_data = f.read()
    
    da_file = DAFile.parse_da(da_raw_data)

    da1 = da_file.das[0].get_da1()
    da2 = da_file.das[0].get_da2()

    with open("da1.bin", "wb") as f:
        f.write(da1.data[:-da1.sig_len])
        print(f"Wrote da1.bin, size: {len(da1.data[:-da1.sig_len])} bytes")

    with open("da2.bin", "wb") as f:
        f.write(da2.data[:-da2.sig_len])
        print(f"Wrote da2.bin, size: {len(da2.data[:-da2.sig_len])} bytes")
    
    if da1.sig_len > 0:
        with open("da1.sig", "wb") as f:
            f.write(da1.data[-da1.sig_len:])
            print(f"Wrote da1.sig, size: {da1.sig_len} bytes")
    
    if da2.sig_len > 0:
        with open("da2.sig", "wb") as f:
            f.write(da2.data[-da2.sig_len:])
            print(f"Wrote da2.sig, size: {da2.sig_len} bytes")
    
    print("DA stages extracted successfully.")
