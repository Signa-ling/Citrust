#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::graphics::{PixelColor, Graphics};
use library::{FrameBufferConfig};

// note: _start()じゃないと読み込んでくれない
#[no_mangle]
pub extern "efiapi" fn _start(config: FrameBufferConfig) -> ! {
    let mut grahpics_struct = Graphics::new(config);
    for x in 0..config.horizontal_resolution {
        for y in 0..config.vertical_resolution {
            grahpics_struct.write_px(x, y, PixelColor(255, 255, 255));
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            grahpics_struct.write_px(100 + x, 100 + y, PixelColor(0, 255, 0));
        }
    }

    grahpics_struct.write_ascii(50, 50, 'A', PixelColor(0, 0, 0));
    grahpics_struct.write_ascii(50, 50, 'A', PixelColor(0, 0, 0));

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
