#![no_std]
#![no_main]
// Allow use of unstable `x86-interrupt` calling convention
#![feature(abi_x86_interrupt)]
// Custom test framework
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod drivers;
mod kernel;
use crate::drivers::display::vga;
#[allow(unused_imports)]
use crate::drivers::qemu_serial::serial::*;
use crate::kernel::delay::nops;
use core::panic::PanicInfo;
#[allow(unused_imports)]
use project_fox::test_runner;

/// For typical rust binary that links to stdlib, execution start in
/// the C runtime lib`crt0` ("C runtime zero"). `crt0` initializes the environment
/// for a C application, i.e creating a stack and placing the args in the right regs.
/// C runtime then invokes entry point of the Rust runtime [`lang_start`](https://github.com/rust-lang/rust/blob/bb4d1491466d8239a7a5fd68bd605e3276e97afb/src/libstd/rt.rs#L32-L73)
/// Note that it is marked by the `start` language item.
/// Rust has a minimal runtime, that sets up stack overflow guards and printing a bt on panic.
/// This runtime then calls `main`.
///
/// Since we are a freestanding executable, we do not have access to the Rust runtime nor `crt0`.
/// So the below defines our own entry point by overwriting the `crt0` entry point directly.
/// Note that implementing the `start` language item isn't useful, since it still needs `crt0`.
///
/// With `#[no_mangle]` we disable name mangling so that the Rust compiler outputs a function named `_start`
/// not some cryptic symbol in an effort to give every function a unique name. We need the actual name so we can
/// tell the linker where to look for entry.
///
/// The reason this function is named `_start`, typically the default entry point name
/// for most systems.
///
/// This function also does not return `!` as it is not called by any function, but directly
/// by the `bootloader` or `OS`, so instead of returning, the entry point should e.g. invoke the
/// exit system call of the operating system. For our case, shutting down/looping is sufficient.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("----- Booting Fox Kernel v0.0.1 -----");

    // Init kernel sub-routines
    if let Err(()) = project_fox::init() {
        panic!("Kernel Init Failed");
    }

    #[cfg(test)]
    test_main();

    let mut loop_count: usize = 0;
    loop {
        nops(1000000);
        println!("Kernel Loop Count: {}", loop_count);
        loop_count = loop_count.wrapping_add(1);
    }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    project_fox::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(true, true);
}
