#![no_std]
// Allow use of unstable `x86-interrupt` calling convention
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod drivers;
pub mod kernel;
#[allow(unused_imports)]
pub use crate::drivers::display::vga;
use crate::drivers::qemu_serial::serial::*;
pub use crate::kernel::interrupts;
use core::panic::PanicInfo;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// lib.rs is tested independently of `main.rs` so it required an entry point
/// and a panic_handler for when it is compiled in `test mode.
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Init kernel sub-routines
    if let Err(()) = init() {
        panic!("Kernel Init Failed");
    }
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// Initialize OS, central place for initialization subroutines
/// that are shared between `_start` functions (main/lib/tests)
pub fn init() -> Result<(), ()> {
    interrupts::idt_init();
    Ok(())
}
