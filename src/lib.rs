#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

use core::panic::PanicInfo;
use core::{alloc::Layout, any};
use gdt::init_gdt;
use interrupts::{init_idt, PICS};
use x86_64::instructions::{hlt, interrupts as x86_64_interrupts, port::Port};

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(kernel_test_main);

#[cfg(test)]
fn kernel_test_main(_boot_info: &'static BootInfo) -> ! {
    init();

    test_main();

    hlt_loop()
}

pub fn init() {
    init_gdt();
    init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64_interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        hlt()
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {layout:?}")
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success)
}
pub trait Testable {
    fn run(&self) -> ();
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);

    exit_qemu(QemuExitCode::Fail);

    hlt_loop()
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) -> () {
        serial_print!("{}...\t", any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Fail = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32)
    }
}
