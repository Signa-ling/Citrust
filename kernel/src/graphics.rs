use library::{FrameBufferConfig, PixelFormat};

pub const FONT_A: [u8; 16] = [
    0b00000000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00100100,
    0b00100100,
    0b00100100,
    0b00100100,
    0b01111110,
    0b01000010,
    0b01000010,
    0b01000010,
    0b11100111,
    0b00000000,
    0b00000000,
];

#[derive(Clone, Copy)]
pub struct PixelColor(pub u8, pub u8, pub u8);

fn pixel_writer(config: &mut FrameBufferConfig, base: usize, color_val: [u8; 3]) {
    let fb = unsafe { core::slice::from_raw_parts_mut(config.frame_buffer, config.size * 4) };
    fb[base+0] = color_val[0];
    fb[base+1] = color_val[1];
    fb[base+2] = color_val[2];
}

pub struct Graphics {
    config: FrameBufferConfig,
    px_writer: fn(&mut FrameBufferConfig, usize, PixelColor),
}

impl Graphics {
    pub fn new(config: FrameBufferConfig) -> Self {
        fn px_writer_rgb(config: &mut FrameBufferConfig, base: usize, color: PixelColor) {
            pixel_writer(config, base, [color.0, color.1, color.2])
        }

        fn px_writer_bgr(config: &mut FrameBufferConfig, base: usize, color: PixelColor) {
            pixel_writer(config, base, [color.2, color.1, color.0])
        }

        // 生成時にformatを判定することで判定処理コストの削減
        let px_writer = match config.pixel_format {
            PixelFormat::PixelRGBResv8bitPerColor => px_writer_rgb,
            PixelFormat::PixelBGRResv8bitPerColor => px_writer_bgr,
        };

        Graphics {
            config,
            px_writer,
        }
    }

    pub fn write_px(&mut self, x: u32, y: u32, color: PixelColor){
        let px_pos = self.config.pixels_per_scan_line * y + x;
        let base = 4 * px_pos as usize;
        (self.px_writer)(&mut self.config, base, color);
    }

    pub fn write_ascii(&mut self, x: u32, y: u32, c: char, color: PixelColor){
        if c != 'A' {
            return;
        }
        for dx in 0..8 {
            for dy in 0..16 {
                if FONT_A[dy] << dx & 0x80 != 0 {
                    self.write_px(x+dx, y+(dy as u32), color);
                }
            }
        }
    }
}