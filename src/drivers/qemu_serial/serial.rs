use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // 0x3F8 ==  Standard port number for the first serial interface
        let mut sp = unsafe { SerialPort::new(0x3F8) };
        sp.init();
        Mutex::new(sp)
    };
}

/// Use the serial i/f to print to host
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        crate::drivers::qemu_serial::serial::_print(format_args!($($arg)*));
    };
}

/// Use the serial i/f to print to host appending newline;
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Failed to print to serial");
}

#[allow(dead_code)]
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        // 0xf4 = iobase of the `isa-debug-exit` dev.
        let mut port = Port::new(0xf4);
        // qemu exits with (exit_code << 1) | 1
        port.write(exit_code as u32);
    }
}
