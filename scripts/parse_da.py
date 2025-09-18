#!/usr/bin/env python3
#
# SPDX-FileCopyrightText: 2025 Shomy
# SPDX-License-Identifier: AGPL-3.0-or-later
#
from dataclasses import dataclass
from enum import Enum, auto
from typing import List, Optional
import struct

class DAType(Enum):
    LEGACY = auto()
    V5 = auto()
    V6 = auto()

@dataclass
class DAEntryRegion:
    data: bytes
    offset: int
    length: int
    addr: int
    region_length: int
    sig_len: int


@dataclass
class DA:
    da_type: DAType
    regions: List[DAEntryRegion]
    magic: int
    hw_code: int
    hw_sub_code: int

    def get_da1(self) -> Optional[DAEntryRegion]:
        return self.regions[1] if len(self.regions) >= 3 else None

    def get_da2(self) -> Optional[DAEntryRegion]:
        return self.regions[2] if len(self.regions) >= 3 else None


class DAFile:
    def __init__(self, da_raw_data: bytes, da_type: DAType, das: List[DA]):
        self.da_raw_data = da_raw_data
        self.da_type = da_type
        self.das = das

    @staticmethod
    def parse_da(raw_data: bytes) -> 'DAFile':
        if len(raw_data) < 0x6C:
            raise ValueError("Invalid DA file: Too short")

        hdr = raw_data[:0x6C]


        if hdr.startswith(b'\xDA\xDA'):
            da_type = DAType.LEGACY
        elif b"MTK_DA_v6" in hdr:
            da_type = DAType.V6
        else:
            da_type = DAType.V5

        if da_type != DAType.LEGACY and b"MTK_DOWNLOAD_AGENT" not in hdr:
            raise ValueError("Invalid DA file: Missing MTK_DOWNLOAD_AGENT signature")

        version = struct.unpack_from("<I", hdr, 0x60)[0]
        num_socs = struct.unpack_from("<I", hdr, 0x68)[0]
        magic_number = hdr[0x64:0x68]

        da_entry_size = 0xD8 if da_type == DAType.LEGACY else 0xDC
        das = []

        for i in range(num_socs):
            start = 0x6C + i * da_entry_size
            end = start + da_entry_size
            da_entry = raw_data[start:end]

            magic = struct.unpack_from("<H", da_entry, 0x00)[0]
            hw_code = struct.unpack_from("<H", da_entry, 0x02)[0]
            hw_sub_code = struct.unpack_from("<H", da_entry, 0x04)[0]
            region_count = struct.unpack_from("<H", da_entry, 0x12)[0]

            regions = []
            region_offset = 0x14

            for _ in range(region_count):
                offset, length, addr, _, sig_len = struct.unpack_from("<IIIII", da_entry, region_offset)
                region_data = raw_data[offset:offset + length]

                regions.append(DAEntryRegion(
                    data=region_data,
                    offset=offset,
                    length=length,
                    addr=addr,
                    region_length=length - sig_len,
                    sig_len=sig_len
                ))

                region_offset += 20

            das.append(DA(
                da_type=da_type,
                regions=regions,
                magic=magic,
                hw_code=hw_code,
                hw_sub_code=hw_sub_code
            ))

        return DAFile(da_raw_data=raw_data, da_type=da_type, das=das)


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <da_file>")
        sys.exit(1)
    
    da_file_path = sys.argv[1]
    with open(da_file_path, "rb") as f:
        da_raw_data = f.read()
    
    da_file = DAFile.parse_da(da_raw_data)

    print("="*60)
    print(f"DA Type: {da_file.da_type.name}")
    print(f"Number of SoCs: {len(da_file.das)}")
    print("="*60)
    

    for i, da in enumerate(da_file.das):
        print(f"[SoC {i}]")
        print(f"  HW Code     : 0x{da.hw_code:04X}")
        print(f"  HW Sub Code : 0x{da.hw_sub_code:04X}")
        print(f"  Magic       : 0x{da.magic:04X}")
        print(f"  Regions     : {len(da.regions)}")
        for j, region in enumerate(da.regions):
            print(f"  Region {j}: Offset: 0x{region.offset:X}, Length: 0x{region.length:X}, Addr: 0x{region.addr:X}, Region Length: 0x{region.region_length:X}, Sig Len: 0x{region.sig_len:X}")
    
    
    