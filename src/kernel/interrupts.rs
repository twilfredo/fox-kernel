use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// Instead of using a static mut for the IDT which is far from idiomatic,
// as `static muts` are prone to data races and we have to use `unsafe`. Let's use
// a `lazy_static` Instead of evaluating a `static` at compile time, the macro
// performs initialization when the static is referenced for the first time.
// Allows us to do almost anything in the initialization block, even read runtime values.
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Specify breakpoint handler
        idt.breakpoint.set_handler_fn(bp_handler);
        idt
    };
}

/// Initialize the interrupt descriptor table
#[allow(dead_code)]
pub fn idt_init() {
    // For the CPU to use this IDT, we need to load it using the
    // load interrupt descriptor table (lidt) instruction
    // The x86_64 crate provides a load method for us.
    // Note: `load` expects a `&'static self` that lives during the
    // entire lifetime of the program, as the CPU will access this table
    // on every interrupt, until a different IDT is loaded.
    IDT.load();
}

/// Breakpoint handler
/// Note the `extern` here define foreign calling convention.
/// Here, it is `x86-interrupt` calling convention
///
/// Note: In rust x86-interrupt calling convention is still unstable
/// To use it anyways, we explicitly enable it with `#![feature(abi_x86_interrupt)]`
extern "x86-interrupt" fn bp_handler(sf: InterruptStackFrame) {
    println!("CPU EXCEPTION: Breakpoint\n {:#?}", sf);
}
