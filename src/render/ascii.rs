use super::{Renderer, Frame};
use anyhow::Result;
use crossterm::{execute, queue, terminal, cursor, style::{Print, ResetColor}};
use std::io::{stdout, Write, BufWriter};

pub struct AsciiRenderer {
    width: u16,
    height: u16,
    previous_frame: Vec<u8>, // Stores luminance values
}

impl AsciiRenderer {
    pub fn new() -> Self {
        Self { width: 0, height: 0, previous_frame: Vec::new() }
    }

    fn get_char(luminance: u8) -> &'static str {
        // Standard ASCII ramp from dark to light
        const RAMP: &[&str] = &[" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];
        let idx = (luminance as usize * (RAMP.len() - 1)) / 255;
        RAMP[idx]
    }
}

impl Renderer for AsciiRenderer {
    fn init(&mut self, width: u16, height: u16) -> Result<()> {
        self.width = width;
        self.height = height;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
        self.previous_frame = vec![0; width as usize * height as usize];
        Ok(())
    }

    fn render_frame(&mut self, frame: &Frame, offset_x: u16, offset_y: u16, _cells_w: u16, _cells_h: u16) -> Result<()> {
        let mut out = BufWriter::new(stdout());
        
        let render_w = std::cmp::min(self.width, frame.width);
        let render_h = std::cmp::min(self.height, frame.height);
        
        for y in 0..render_h {
            for x in 0..render_w {
                let curr_idx = (y as usize * frame.width as usize + x as usize) * 3;
                let r = frame.pixels[curr_idx] as f64;
                let g = frame.pixels[curr_idx + 1] as f64;
                let b = frame.pixels[curr_idx + 2] as f64;
                
                // Luminance formula
                let luminance = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;
                
                let term_x = x + offset_x;
                let term_y = y + offset_y;
                let prev_idx = term_y as usize * self.width as usize + term_x as usize;
                let prev_lum = self.previous_frame[prev_idx];

                if luminance != prev_lum {
                    queue!(
                        out,
                        cursor::MoveTo(term_x, term_y),
                        Print(Self::get_char(luminance))
                    )?;
                    self.previous_frame[prev_idx] = luminance;
                }
            }
        }
        out.flush()?;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        execute!(
            stdout(),
            ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        Ok(())
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.width = width;
        self.height = height;
        self.previous_frame = vec![0; width as usize * height as usize];
        Ok(())
    }

    fn supported() -> bool {
        true
    }

    fn get_logical_size(&self, term_width: u16, term_height: u16) -> (u16, u16) {
        // ASCII mode is 1:1 with characters
        (term_width, term_height)
    }
}
