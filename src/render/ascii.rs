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

    fn render_frame(
        &mut self,
        frame: &Frame,
        offset_x: u16,
        offset_y: u16,
        cells_w: u16,
        cells_h: u16,
    ) -> Result<()> {
        let mut out = BufWriter::new(stdout());
        
        let available_w = self.width.saturating_sub(offset_x);
        let available_h = self.height.saturating_sub(offset_y);
        let render_w = frame.width.min(cells_w).min(available_w);
        let render_h = frame.height.min(cells_h).min(available_h);
        
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
