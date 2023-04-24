#![no_std]
#![no_main]
mod panic;

static HELLO: &[u8] = b"Hello World!";

/// For typical rust binary that links to stdlib, execution start in
/// the C runtime lib`crt0` ("C runtime zero"). `crt0` initializes the environment
/// for a C application, i.e creating a stack and placing the args in the right regs.
/// C runtime then invokes entry point of the Rust runtime [`lang_start`](https://github.com/rust-lang/rust/blob/bb4d1491466d8239a7a5fd68bd605e3276e97afb/src/libstd/rt.rs#L32-L73)
/// Note that it is marked by the `start` language item.
/// Rust has a minimal runtime, that sets up stack overflow guards and printing a bt on panic.
/// This runtime then calls `main`.
///
/// Since we are a freestanding executable, we do not have access to teh Rust runtime nor `crt0`.
/// So the below defines our own entry point by overwriting the `crt0` entry point directly.
/// Note that implementing the `start` language item isn't useful, since it still need `crt0`.
///
/// With `#[no_mangle]` we disable name mangling so that the Rust compiler outputs a function named `_start`
/// not some cryptic symbol in an effort to give every function a unique name. We need the actual name so we can
/// tell the linker where to look for entry.
///
/// The reason this function is named `_start`, typically the default entry point name
/// for most systems.
///
/// This function also does not return `!` as it is not called by any function, by directly
/// by the `bootloader` or `OS`, so instead of returning he entry point should e.g. invoke the
/// exit system call of the operating system.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buf = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buf.offset(i as isize * 2) = byte;
            *vga_buf.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
