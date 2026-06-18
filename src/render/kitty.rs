use super::{Renderer, Frame};
use anyhow::Result;
use crossterm::{execute, terminal, cursor};
use std::io::{stdout, Write, BufWriter};
use base64::{Engine as _, engine::general_purpose};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct KittyRenderer {
    width: u16,
    height: u16,
    low_res: bool,
}

impl KittyRenderer {
    pub fn new(low_res: bool) -> Self {
        Self { width: 0, height: 0, low_res }
    }
}

impl Renderer for KittyRenderer {
    fn init(&mut self, width: u16, height: u16) -> Result<()> {
        self.width = width;
        self.height = height;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
        Ok(())
    }

    fn render_frame(&mut self, frame: &Frame, offset_x: u16, offset_y: u16, cells_w: u16, cells_h: u16) -> Result<()> {
        let mut out = BufWriter::new(stdout());
        
        // Move to centered position to overwrite the previous image
        execute!(out, cursor::MoveTo(offset_x, offset_y))?;
        
        // Use a temporary file (t=t medium)
        // This is extremely fast on Linux as /tmp is usually tmpfs (RAM-backed)
        // and avoids POSIX SHM permission/sandboxing issues.
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temp_path = std::env::temp_dir().join(format!("termiplay_frame_{}.rgb", timestamp));
        std::fs::write(&temp_path, &frame.pixels)?;
        
        let path_str = temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let b64_path = general_purpose::STANDARD.encode(path_str);
        
        // t=t tells Kitty the payload is a file path, and Kitty should DELETE it after reading.
        write!(out, "\x1b_Ga=T,f=24,s={},v={},c={},r={},i=1,q=2,t=t;{}\x1b\\", 
            frame.width, frame.height, cells_w, cells_h, b64_path)?;
        
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
        Ok(())
    }

    fn supported() -> bool {
        std::env::var_os("KITTY_WINDOW_ID").is_some()
            || std::env::var("TERM")
                .map(|term| term.to_ascii_lowercase().contains("kitty"))
                .unwrap_or(false)
    }

    fn get_logical_size(&self, term_width: u16, term_height: u16) -> (u16, u16) {
        if self.low_res {
            // Low resolution: ~480p or tied to cell count
            let target_w = std::cmp::min(640, term_width * 5);
            let target_h = std::cmp::min(360, term_height * 10);
            (target_w, target_h)
        } else {
            // High resolution
            let target_w = std::cmp::min(1280, term_width * 15);
            let target_h = std::cmp::min(720, term_height * 30);
            (target_w, target_h)
        }
    }
}
