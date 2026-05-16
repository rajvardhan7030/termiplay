use anyhow::{Result, anyhow};

pub mod ansi;
pub mod unicode;
pub mod kitty;
pub mod ascii;

pub struct Frame {
    pub width: u16,
    pub height: u16,
    pub pixels: Vec<u8>, // Raw RGBRGB... buffer for speed
    pub timestamp: f64,  // Presentation time in seconds
}

pub trait Renderer {
    fn init(&mut self, width: u16, height: u16) -> Result<()>;
    fn render_frame(&mut self, frame: &Frame, offset_x: u16, offset_y: u16, cells_w: u16, cells_h: u16) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn resize(&mut self, width: u16, height: u16) -> Result<()>;
    fn supported() -> bool where Self: Sized;
    
    /// Returns the logical resolution needed for this renderer given a terminal size.
    fn get_logical_size(&self, term_width: u16, term_height: u16) -> (u16, u16);
}

pub fn create_renderer(mode: &str, low: bool) -> Result<Box<dyn Renderer>> {
    match mode {
        "ansi" => Ok(Box::new(ansi::AnsiRenderer::new())),
        "unicode" => Ok(Box::new(unicode::UnicodeRenderer::new())),
        "kitty" => Ok(Box::new(kitty::KittyRenderer::new(low))),
        "ascii" => Ok(Box::new(ascii::AsciiRenderer::new())),
        _ => Err(anyhow!("Renderer mode '{}' not implemented yet.", mode)),
    }
}
