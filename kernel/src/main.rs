#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::graphics::{PixelColor, write_px};
use library::{FrameBufferConfig};

// note: _start()じゃないと読み込んでくれない
#[no_mangle]
pub extern "efiapi" fn _start(config: FrameBufferConfig) -> ! {
    for x in 0..config.horizontal_resolution {
        for y in 0..config.vertical_resolution {
            unsafe{ write_px(&config, x, y, PixelColor {r: 255, g: 255, b: 255 }) };
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            unsafe{ write_px(&config, 100 + x, 100 + y, PixelColor {r: 0, g: 255, b: 0 }) };
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
