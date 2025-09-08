## What are DA Extensions?

DA Extensions are supplemental code (like an addon) developed by [bkerler](https://github.com/bkerler) that is loaded alongside the original DA2, to remove restrictions imposed by stock [[Download Agent|DAs]].

DA Extensions are available for [[XFlash DA Protocol|XFlash]] and [[XML DA Protocol|XML]] Download Agents.

## How do they work?

To load DA Extensions, you first need to be able to boot patched download agents (or at least, a custom DA2).
This is to ensure hash check is disabled.

The DA Extensions are loaded at `0x68000000`, to ensure the original DA2 is not being overwritten.

Before being sent, the DA extension binary is patched to hook into the original DA2 calls, ensuring compatibility with it.

## Removed restrictions / Restored functions

* Restored memory read and write command (Registers)
* RPMB read and write
* SEJ