use super::{Renderer, Frame};
use anyhow::Result;
use crossterm::{execute, queue, terminal, cursor, style::{SetForegroundColor, Color, Print}};
use std::io::{stdout, Write, BufWriter};

pub struct AnsiRenderer {
    width: u16,
    height: u16,
    previous_frame: Vec<u8>, // Raw RGB buffer
}

impl AnsiRenderer {
    pub fn new() -> Self {
        Self { width: 0, height: 0, previous_frame: Vec::new() }
    }
}

impl Renderer for AnsiRenderer {
    fn init(&mut self, width: u16, height: u16) -> Result<()> {
        self.width = width;
        self.height = height;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide
        )?;
        self.previous_frame = vec![0; (width as usize * height as usize) * 3];
        Ok(())
    }

    fn render_frame(&mut self, frame: &Frame, offset_x: u16, offset_y: u16, _cells_w: u16, _cells_h: u16) -> Result<()> {
        let mut out = BufWriter::new(stdout());
        
        let render_w = std::cmp::min(self.width, frame.width);
        let render_h = std::cmp::min(self.height, frame.height);
        
        for y in 0..render_h {
            for x in 0..render_w {
                let curr_idx = (y as usize * frame.width as usize + x as usize) * 3;
                let term_x = x + offset_x;
                let term_y = y + offset_y;
                let prev_idx = (term_y as usize * self.width as usize + term_x as usize) * 3;
                
                let rgb = &frame.pixels[curr_idx..curr_idx+3];
                let prev_rgb = &self.previous_frame[prev_idx..prev_idx+3];
                
                if rgb != prev_rgb {
                    queue!(
                        out,
                        cursor::MoveTo(term_x, term_y),
                        SetForegroundColor(Color::Rgb { r: rgb[0], g: rgb[1], b: rgb[2] }),
                        Print("█")
                    )?;
                    self.previous_frame[prev_idx..prev_idx+3].copy_from_slice(rgb);
                }
            }
        }
        out.flush()?;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        execute!(
            stdout(),
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        Ok(())
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.width = width;
        self.height = height;
        self.previous_frame = vec![0; (width as usize * height as usize) * 3];
        Ok(())
    }

    fn supported() -> bool {
        true
    }

    fn get_logical_size(&self, term_width: u16, term_height: u16) -> (u16, u16) {
        (term_width, term_height)
    }
}
