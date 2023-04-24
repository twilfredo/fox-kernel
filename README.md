# FOX

## Overview

An `x86-64` kernel written in Rust. The primary goal of this project is to learn OS/Kernel development.

## Build an Run

You can build the project with

```shell
cargo build
```

To build an executable binary:

```shell
cargo bootimage
```

This will create a bootable disk image named `bootimage-project_fox.bin` in the `target/x86_64-fox/debug` directory. This tool willl recompile the kernel then compiles the bootloader, then `bootimage` combine the kernel and the bootloader into a bootable disk image.

The `bootimage` tool works by:

* It compiles our kernel to an ELF file.
* It compiles the bootloader dependency as a standalone executable.
* It links the bytes of the kernel ELF file to the bootloader.

During boot, the bootloader reads and parses the appended ELF file. It then maps the program segments to virtual addresses in the page tables, zeroes the `.bss` section, and sets up a stack. Finally, it reads the entry point address (our _start function) and jumps to it.

### Booting it in QEMU

You can boot the kernel in QEMU with

```shell
cargo run
```