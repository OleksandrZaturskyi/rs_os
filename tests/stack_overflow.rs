#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use rs_os::{
    exit_qemu,
    gdt::{init_gdt, DOUBLE_FAULT_IST_INDEX},
    serial_print, serial_println, QemuExitCode,
};
use volatile::Volatile;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    init_gdt();
    init_test_idt();

    stack_overflow();

    panic!("execution continued after stack overflow")
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rs_os::test_panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    Volatile::new(0).read(); // prevent tail recursion optimizations
}

fn init_test_idt() {
    TEST_IDT.load();
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");

    exit_qemu(QemuExitCode::Success);

    loop {}
}
