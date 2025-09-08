The seccfg (Security Configuration) partition in Mediatek devices holds (as the name implies) security configurations.

![[seccfg_v4.png]]
*Seccfg V4 hex*

In it, the following information are stored:
* Seccfg version
* Seccfg size
* Lock state
* Critical Lock state
* Sboot runtime

The seccfg V4 structure is as follows

| Name                    | Value                                                                                         | Length                   |
| ----------------------- | --------------------------------------------------------------------------------------------- | ------------------------ |
| Magic Start             | `0x4D4D4D4D`                                                                                  | 4 bytes                  |
| Seccfg version          | `0x4` (seccfg v4)                                                                             | 4 bytes                  |
| Lock state              | `0x1` (bootloader locked) / `0x3` (bootloader unlocked)                                       | 4 bytes                  |
| Critical lock state     | `0x1` (bootloader locked) / `0x0` (bootloader unlocked)                                       | 4 bytes                  |
| Sboot runtime           | `0x0`                                                                                         | 4 bytes                  |
| Magic End               | `0x45454545` (Unless seccfg is malformed)                                                     | 4 bytes                  |
| Encrypted Hash (sha256) | sha256 of the previous values packed together, then encrypted with [[SEJ]], unique per device | 32 bytes                 |
| Padding                 | `0x00`                                                                                        | Until `0x200` is reached |
