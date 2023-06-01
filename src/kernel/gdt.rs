use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
/// The Global Descriptor Table (GDT) is an old mechanism that was used for memory segmentation
/// prior to paging becoming the de facto standard. However, we still need it in 64-bit mode for
/// things such as, kernel/user mode cfg or Task State Segment (TSS) loading.
/// The GDT is a structure that contains segments of the program, used in older architectures
/// for program isolation prior to paging. Although segmentation is no longer supported in 64-bit
/// mode, the GDT refuses to leave!
///
/// The GDT must be configured as the processor expects it to exist.
/// A bootloader will put in its own GDT, but the OS will have little to no idea where
/// the bootloader's GDT is in memory. As such, it runs the risk of overwriting it.
/// Destroying the GDT results in an immediate triple fault. To avoid this,
/// the OS must configure its own GDT. This also allows the kernel to create a TSS entry,
/// which bootloaders are unlikely to include.
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Use lazy_static as Rusts const evaluator cannot perform this init at compile time...yet
lazy_static! {
    static ref TSS: TaskStateSegment  = {
        let mut tss = TaskStateSegment::new();
        // Define 0th IST as the double fault stack (note any other IST index can work too)
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            // Without memory management, there's no way to properly allocate a new stack,
            // so, use a `static mut` array as stack storage for now. If this was a `static` only
            // the bootloader would map this to a read-only page.
            // This stack also does not have a guard page to protect against stack overflows, meaning
            // that we need to be careful in our double fault handler not to overflow the stack. An overflow, may
            // silently fail corrupting any memory under the stack.
            // TODO: Implement proper stack allocation + guard page.
            //
            // x86_64 ABI requires 16 byte alignment
            // See: https://github.com/phil-opp/blog_os/issues/449#issuecomment-555811809
            #[repr(align(16))]
            struct Stack([u8; STACK_SIZE]);
            static mut STACK: Stack = Stack([0; STACK_SIZE]);

            // The `unsafe` is required here, as the compiler cannot guarantee race freedom
            // given that we are using a mutable static.
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            // Write the top address of a double fault stack to the 0th entry
            // Note: x86 stack grows downwards, from high addresses to low addresses
            stack_end
        };
        tss
    };
}

// Create a new GDT with a code segment and a TSS segment
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // A segment is a designated area of the address space.
        // It consists of a start address and a length which defines the area
        // the segment covers. Segments have access permissions which prevent
        // programs of lower privilege levels from accessing the memory within them.
        //
        // Segments are also designated as either data or code segments.
        // Data segments can be read or written but not executed
        // (i.e. a data segment selector can go in any segment register except
        // the Code Segment Selector register), code segments can only be executed
        // (i.e. a code segment selector can only go in the Code Segment Selector register).
        //
        // Segments can overlap. This means an area of memory can be designated as both data
        // and code and/or be accessible to one or more privilege levels. If segmentation is
        // being avoided (in favour of paging) then it is standard practice to configure four segments.
        // Two code segments and two data segments. One of the two has Descriptor Privilege Level 0
        // (i.e. Ring 0 meaning Kernel-mode access) and the other with DPL 3 (i.e. Ring 3 meaning User-mode access).
        // Almost no modern operating systems use Rings 1 and 2.
        //
        // A descriptor is an entry in the Global Descriptor Table.
        // It contains all the information about the start (/offset) address of segments and their length,
        // their access levels and types along with some other information.
        //
        // A segment selector/selector is an offset in the GDT that specifies a descriptor to use, selectors are
        // put into the Segment Selector registers.
        // A selector is the number of bytes from the start of the table to the start of the descriptor (/table entry)
        // For a standard GDT, this means the selector for the second entry (the first valid one after the NULL descriptor)
        // has selector 0x10 (16).
        //
        // More at: https://web.archive.org/web/20190217233448/https://www.flingos.co.uk/docs/reference/Global-Descriptor-Table/
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors{code_selector, tss_selector})
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

#[allow(dead_code)]
pub fn gdt_init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // Although the GDT is loaded, GDT segments are not yet
        // active because the segment and TSS registers still contain
        // the values from the old GDT.
        // Let's reload the code segment register (since we changed the GDT, the old segment
        // might point to some arbitrary descriptor in the new GDT), and load teh TSS.
        // We loaded a GDT that contains a TSS selector, but we still need to tell the
        // CPU that it should use the specified TSS.
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

// TODO FROM: A Stack Overflow Test
