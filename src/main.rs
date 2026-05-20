use clap::Parser;
use anyhow::{Result, Context, anyhow};
use crossbeam_channel::bounded;
use std::time::{Duration, Instant};
use crossterm::{execute, terminal};
use crossterm::event::{self, Event, KeyCode};
use std::sync::{Arc, RwLock};
use std::io::stdout;

mod render;
mod decode;
mod audio;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Video file to play
    file: String,

    /// Rendering mode (ansi, unicode, ascii, kitty)
    #[arg(short, long, default_value = "unicode")]
    mode: String,

    /// Lower resolution for high-performance rendering (Kitty mode)
    #[arg(long, default_value_t = false)]
    low: bool,
}

fn calculate_dimensions(
    vid_w: u32,
    vid_h: u32,
    term_w: u16,
    term_h: u16,
) -> (u16, u16) {
    let aspect = vid_w as f64 / vid_h as f64;
    let target_w = term_w;
    let target_h = (term_w as f64 / aspect) as u16;
    if target_h <= term_h {
        (target_w, target_h)
    } else {
        let target_h = term_h;
        let target_w = (term_h as f64 * aspect) as u16;
        (target_w, target_h)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let (mut term_w, mut term_h) = crossterm::terminal::size()?;
    
    ffmpeg_next::init().context("Failed to initialize FFmpeg")?;
    let ictx = ffmpeg_next::format::input(&args.file).context("Failed to open input file")?;
    let input = ictx.streams().best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| anyhow!("Could not find video stream"))?;
    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
    let decoder = context_decoder.decoder().video()?;
    let vid_w = decoder.width();
    let vid_h = decoder.height();
    drop(decoder);
    drop(ictx);

    let mut renderer = render::create_renderer(&args.mode, args.low)?;
    let (logical_w, logical_h) = renderer.get_logical_size(term_w, term_h);
    let (target_w, mut target_h) = calculate_dimensions(vid_w, vid_h, logical_w, logical_h);
    if args.mode == "unicode" && target_h % 2 != 0 { target_h -= 1; }

    let target_dims = Arc::new(RwLock::new((target_w, target_h)));
    let audio_clock = Arc::new(RwLock::new((0.0, Instant::now())));

    renderer.init(term_w, term_h)?;

    let (v_tx, v_rx) = bounded(20);
    let (a_tx, a_rx) = bounded(100);
    
    let file = args.file.clone();
    let dims_clone = target_dims.clone();
    std::thread::spawn(move || {
        if let Err(e) = decode::decode_pipeline(&file, v_tx, a_tx, dims_clone) {
            eprintln!("Decoder error: {}", e);
        }
    });

    let clock_clone = audio_clock.clone();
    std::thread::spawn(move || {
        if let Err(e) = audio::run_audio_thread(a_rx, clock_clone) {
            eprintln!("Audio error: {}", e);
        }
    });

    let mut pending_video_frame = None;
    let start_time = Instant::now();
    loop {
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        return Ok(());
                    }
                }
                Event::Resize(nw, nh) => {
                    term_w = nw; term_h = nh;
                    let (lw, lh) = renderer.get_logical_size(term_w, term_h);
                    let (tw, mut th) = calculate_dimensions(vid_w, vid_h, lw, lh);
                    if args.mode == "unicode" && th % 2 != 0 { th -= 1; }
                    {
                        let mut dims = target_dims.write().unwrap();
                        *dims = (tw, th);
                    }
                    while v_rx.try_recv().is_ok() {}
                    pending_video_frame = None;
                    renderer.resize(term_w, term_h)?;
                    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
                }
                _ => {}
            }
        }
        
        let mut disconnected = false;
        let frame_opt = if let Some(f) = pending_video_frame.take() {
            Some(f)
        } else {
            match v_rx.try_recv() {
                Ok(frame) => Some(frame),
                Err(crossbeam_channel::TryRecvError::Empty) => None,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    disconnected = true;
                    None
                }
            }
        };

        if disconnected { break; }

        if let Some(frame) = frame_opt {
            let (base_clock, clock_updated_at) = *audio_clock.read().unwrap();
            let elapsed = if base_clock > 0.0 {
                base_clock + clock_updated_at.elapsed().as_secs_f64()
            } else {
                start_time.elapsed().as_secs_f64()
            };
            
            // If we're too early, wait but don't block for more than 10ms at a time 
            // to keep the event loop responsive.
            if frame.timestamp > elapsed {
                let diff = frame.timestamp - elapsed;
                if diff > 0.1 {
                    // Too far in the future, maybe a seek happened or clock is reset
                } else if diff > 0.005 {
                    std::thread::sleep(Duration::from_millis(5));
                    pending_video_frame = Some(frame);
                    continue; 
                }
            }
            
            // Skip late frames (more than 100ms behind)
            if elapsed > frame.timestamp + 0.1 {
                continue;
            }
            
            let (tw, th) = *target_dims.read().unwrap();
            let (lw, lh) = renderer.get_logical_size(term_w, term_h);
            let multiplier_w = std::cmp::max(1, lw / term_w);
            let multiplier_h = std::cmp::max(1, lh / term_h);

            let ox = term_w.saturating_sub(tw / multiplier_w) / 2;
            let oy = term_h.saturating_sub(th / multiplier_h) / 2;
            
            let cells_w = tw / multiplier_w;
            let cells_h = th / multiplier_h;

            renderer.render_frame(&frame, ox, oy, cells_w, cells_h)?;
        } else {
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    renderer.clear()?;
    Ok(())
}
