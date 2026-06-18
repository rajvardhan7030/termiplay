use super::{Renderer, Frame};
use anyhow::Result;
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::io::{stdout, Write, BufWriter};

pub struct UnicodeRenderer {
    width: u16,
    height: u16,
    previous_frame: Vec<u8>, // Raw RGB buffer
}

impl UnicodeRenderer {
    pub fn new() -> Self {
        Self { width: 0, height: 0, previous_frame: Vec::new() }
    }
}

impl Renderer for UnicodeRenderer {
    fn init(&mut self, width: u16, height: u16) -> Result<()> {
        self.width = width;
        self.height = height;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
        self.previous_frame = vec![0; (width as usize * (height * 2) as usize) * 3];
        Ok(())
    }

    fn render_frame(&mut self, frame: &Frame, offset_x: u16, offset_y: u16, cells_w: u16, cells_h: u16) -> Result<()> {
        let mut out = BufWriter::new(stdout());
        
        let available_w = self.width.saturating_sub(offset_x);
        let available_rows = self.height.saturating_sub(offset_y);
        let render_w = frame.width.min(cells_w).min(available_w);
        let render_rows = (frame.height / 2).min(cells_h).min(available_rows);
        
        for y in 0..render_rows {
            for x in 0..render_w {
                let curr_top_idx = ((y * 2) as usize * frame.width as usize + x as usize) * 3;
                let curr_bottom_idx = ((y * 2 + 1) as usize * frame.width as usize + x as usize) * 3;
                
                let term_x = x + offset_x;
                let term_y = y + offset_y;
                let prev_top_idx = ((term_y * 2) as usize * self.width as usize + term_x as usize) * 3;
                let prev_bottom_idx = ((term_y * 2 + 1) as usize * self.width as usize + term_x as usize) * 3;

                let top = &frame.pixels[curr_top_idx..curr_top_idx+3];
                let bottom = &frame.pixels[curr_bottom_idx..curr_bottom_idx+3];
                
                let prev_top = &self.previous_frame[prev_top_idx..prev_top_idx+3];
                let prev_bottom = &self.previous_frame[prev_bottom_idx..prev_bottom_idx+3];

                if top != prev_top || bottom != prev_bottom {
                    queue!(
                        out,
                        cursor::MoveTo(term_x, term_y),
                        SetForegroundColor(Color::Rgb { r: top[0], g: top[1], b: top[2] }),
                        SetBackgroundColor(Color::Rgb { r: bottom[0], g: bottom[1], b: bottom[2] }),
                        Print("▀")
                    )?;
                    self.previous_frame[prev_top_idx..prev_top_idx+3].copy_from_slice(top);
                    self.previous_frame[prev_bottom_idx..prev_bottom_idx+3].copy_from_slice(bottom);
                }
            }
        }
        queue!(out, ResetColor)?;
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
        self.previous_frame = vec![0; (width as usize * (height * 2) as usize) * 3];
        Ok(())
    }

    fn supported() -> bool {
        true
    }

    fn get_logical_size(&self, term_width: u16, term_height: u16) -> (u16, u16) {
        (term_width, term_height * 2)
    }
}
