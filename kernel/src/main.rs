#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// note: _start()じゃないと読み込んでくれない
#[no_mangle]
pub extern "efiapi" fn _start(mut fb_addr: *mut u8, fb_size: usize) -> ! {
    // 画面の白塗り
    unsafe {
        let mut cnt = 0;
        while cnt < fb_size {
            *fb_addr = 255;
            fb_addr = fb_addr.add(1);
            cnt = cnt + 1;
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
