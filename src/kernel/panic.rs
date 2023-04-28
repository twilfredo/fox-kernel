#[allow(unused_imports)]
use crate::drivers::qemu_serial::serial::*;
#[allow(unused_imports)]
use crate::println;
#[allow(unused_imports)]
use crate::serial_println;
use core::panic::PanicInfo;

// Default handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// Test Handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    project_fox::test_panic_handler(info);
}
