use super::{Renderer, Frame};
use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
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
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
        self.previous_frame = vec![0; (width as usize * height as usize) * 3];
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
            ResetColor,
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
