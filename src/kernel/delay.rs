use core::arch::asm;

/// Runs nops to simulate some form of delay for now
pub fn nops(count: usize) {
    unsafe {
        for _i in 0..=count {
            asm!("nop;");
        }
    }
}
