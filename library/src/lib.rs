#![no_std]

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    PixelRGBResv8bitPerColor,
    PixelBGRResv8bitPerColor,
}

#[derive(Debug, Clone, Copy)]
pub struct FrameBufferConfig {
    pub frame_buffer: *mut u8,
    pub size: usize,
    pub pixels_per_scan_line: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: PixelFormat,
}
