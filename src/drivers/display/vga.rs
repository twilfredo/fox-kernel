use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Note: Statics are initialized at compile time
// and  Rust’s const evaluator is not able to convert raw pointers to references at compile time (so far)
//
// A workaround is to use lazy_static! macro that defines a lazily initialized static.
// Instead of computing its value at compile time, the static lazily initializes itself when accessed for the first time.
// Thus, the initialization happens at runtime, so arbitrarily complex initialization code is possible.
//
// Spin-locking: Since we have no mutex services, use a spinlock, this avoids us from having to use mutable statics!
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0,
        colour_code: ColourCode::new(Colour::White, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Modified implementation of the stdlib print macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

// Modified implementation of the stdlib println macro
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

impl ColourCode {
    fn new(fg: Colour, bg: Colour) -> Self {
        Self((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    colour_code: ColourCode,
}

// VGA MMIO Buffer Size
const BUFFER_Y: usize = 25;
const BUFFER_X: usize = 80;

#[repr(transparent)]
struct Buffer {
    // Do not optimize away writes to the mmio vga buffer, so use volatile crate to write
    // Note: The compiler cannot know that this is an mmio op to the VGA buffer
    //       and that we are printing characters. So it *may* decide to optimize away
    //       any writes specially as we are not reading anything back.
    chars: [[Volatile<ScreenChar>; BUFFER_X]; BUFFER_Y],
}

pub struct Writer {
    column_pos: usize,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_X {
                    self.new_line();
                }
                let row = BUFFER_Y - 1;
                let col = self.column_pos;

                let colour_code = self.colour_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    colour_code,
                });
                self.column_pos += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Iterate over all characters and move everything a row up
        // 0th index in Y is shifter off the screen, so omit that
        for row in 1..BUFFER_Y {
            for col in 0..BUFFER_X {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        // Clear the last row
        self.clear_row(BUFFER_Y - 1).unwrap();
        self.column_pos = 0;
    }

    fn clear_row(&mut self, row: usize) -> Result<(), ()> {
        if row >= BUFFER_Y {
            return Err(());
        }
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };

        for col in 0..BUFFER_X {
            self.buffer.chars[row][col].write(blank);
        }

        Ok(())
    }

    pub fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                0x20..=0x7e | b'\n' => self.write_byte(b),
                // Not part of printable ASCII, replaced with ■ == 0xfe
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[test_case]
fn test_println_simple() {
    println!("vga: test println!() output");
}

#[test_case]
fn test_println_mul() {
    for _ in 0..500 {
        println!("vga: test println!() output");
    }
}

#[test_case]
fn test_verify_vga_output() {
    let line = "123456789 0xBADCAFE 0xDEADBEEF";
    println!("{line}");
    for (i, c) in line.chars().enumerate() {
        // BUFFER_Y - 2 == {line + newline}
        let vga_char = WRITER.lock().buffer.chars[BUFFER_Y - 2][i].read();
        assert_eq!(char::from(vga_char.ascii_character), c);
    }
    print!("{line}");
    for (i, c) in line.chars().enumerate() {
        // BUFFER_Y - 1 == {line}; no newline with `print()`
        let vga_char = WRITER.lock().buffer.chars[BUFFER_Y - 1][i].read();
        assert_eq!(char::from(vga_char.ascii_character), c);
    }
}
