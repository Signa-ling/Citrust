#![feature(abi_efiapi)]
#![no_std]
#![no_main]
use uefi::prelude::*;
use core::panic::PanicInfo;
use core::fmt::Write;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn efi_main(_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    let stdout = &mut st.stdout();
    writeln!(stdout, "Hello, citrust!").unwrap();

    loop {}
    // Status::SUCCESS
}