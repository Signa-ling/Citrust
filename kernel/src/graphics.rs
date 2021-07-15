use library::{FrameBufferConfig, PixelFormat};

#[derive(Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}


pub unsafe fn write_px(config: &FrameBufferConfig, x: u32, y: u32, color: PixelColor){
    let px_pos = config.pixels_per_scan_line * y + x;
    let (c0, c1, c2) = match config.pixel_format {
        PixelFormat::PixelRGBResv8bitPerColor => (color.r, color.g, color.b),
        PixelFormat::PixelBGRResv8bitPerColor => (color.b, color.g, color.r),        
    };

    let fb = core::slice::from_raw_parts_mut(config.frame_buffer, config.size * 4);
    let base = 4 * px_pos as usize;
    fb[base+0] = c0;
    fb[base+1] = c1;
    fb[base+2] = c2;
}