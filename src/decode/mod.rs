use anyhow::{Result, anyhow, Context};
use crossbeam_channel::Sender;
use crate::render::Frame;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct AudioFrame {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
    pub timestamp: f64,
}

pub fn decode_pipeline(
    path: &str, 
    video_sender: Sender<Frame>, 
    audio_sender: Sender<AudioFrame>,
    target_dims: Arc<RwLock<(u16, u16)>>
) -> Result<()> {
    let mut log = OpenOptions::new().create(true).append(true).open("decoder.log")?;
    let (iw, ih) = *target_dims.read().unwrap();
    writeln!(log, "--- Start Decoding: {} ({}x{}) ---", path, iw, ih)?;
    
    ffmpeg_next::init().context("Failed to initialize FFmpeg")?;
    let mut ictx = ffmpeg_next::format::input(&path).context("Failed to open input file")?;
    
    // Video stream setup
    let video_stream = ictx.streams().best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| anyhow!("Could not find video stream"))?;
    let video_index = video_stream.index();
    let video_time_base: f64 = video_stream.time_base().into();
    let video_context = ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())?;
    let mut video_decoder = video_context.decoder().video()?;
    
    // Audio stream setup (optional)
    let audio_stream = ictx.streams().best(ffmpeg_next::media::Type::Audio);
    let mut audio_data = audio_stream.map(|s| {
        let index = s.index();
        let time_base: f64 = s.time_base().into();
        let context = ffmpeg_next::codec::context::Context::from_parameters(s.parameters()).ok()?;
        let decoder = context.decoder().audio().ok()?;
        Some((index, time_base, decoder))
    }).flatten();

    let (mut current_w, mut current_h) = *target_dims.read().unwrap();
    let mut scaler = ffmpeg_next::software::scaling::context::Context::get(
        video_decoder.format(), video_decoder.width(), video_decoder.height(),
        ffmpeg_next::format::Pixel::RGB24, current_w as u32, current_h as u32,
        ffmpeg_next::software::scaling::flag::Flags::LANCZOS,
    )?;

    // Resampler for audio (to f32 interleaved)
    let mut resampler = if let Some((_, _, ref decoder)) = audio_data {
        Some(ffmpeg_next::software::resampling::context::Context::get(
            decoder.format(), decoder.channel_layout(), decoder.rate(),
            ffmpeg_next::format::Sample::F32(ffmpeg_next::format::sample::Type::Packed),
            ffmpeg_next::util::channel_layout::ChannelLayout::STEREO,
            decoder.rate(),
        )?)
    } else {
        None
    };

    let mut video_decoded = ffmpeg_next::util::frame::video::Video::empty();
    let mut audio_decoded = ffmpeg_next::util::frame::audio::Audio::empty();
    let mut rgb_frame = ffmpeg_next::util::frame::video::Video::empty();

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_index {
            if video_decoder.send_packet(&packet).is_ok() {
                while video_decoder.receive_frame(&mut video_decoded).is_ok() {
                    let timestamp = video_decoded.pts().map(|p| p as f64 * video_time_base).unwrap_or(0.0);
                    
                    let (new_w, new_h) = *target_dims.read().unwrap();
                    if new_w != current_w || new_h != current_h {
                        current_w = new_w; current_h = new_h;
                        scaler = ffmpeg_next::software::scaling::context::Context::get(
                            video_decoder.format(), video_decoder.width(), video_decoder.height(),
                            ffmpeg_next::format::Pixel::RGB24, current_w as u32, current_h as u32,
                            ffmpeg_next::software::scaling::flag::Flags::LANCZOS,
                        )?;
                    }

                    if scaler.run(&video_decoded, &mut rgb_frame).is_ok() {
                        let data = rgb_frame.data(0);
                        let linesize = rgb_frame.stride(0);
                        let mut pixels = Vec::with_capacity((current_w as usize * current_h as usize) * 3);
                        for y in 0..current_h {
                            let row_start = (y as usize) * linesize;
                            pixels.extend_from_slice(&data[row_start..row_start + (current_w as usize * 3)]);
                        }
                        let frame = Frame { width: current_w, height: current_h, pixels, timestamp };
                        if video_sender.send(frame).is_err() { return Ok(()); }
                    }
                }
            }
        } else if let Some((index, time_base, ref mut decoder)) = audio_data {
            if stream.index() == index {
                if decoder.send_packet(&packet).is_ok() {
                    while decoder.receive_frame(&mut audio_decoded).is_ok() {
                        let timestamp = audio_decoded.pts().map(|p| p as f64 * time_base).unwrap_or(0.0);
                        if let Some(ref mut res) = resampler {
                            let mut resampled = ffmpeg_next::util::frame::audio::Audio::empty();
                            if res.run(&audio_decoded, &mut resampled).is_ok() {
                                let data = resampled.data(0);
                                let samples: Vec<f32> = data.chunks_exact(4)
                                    .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                                    .collect();
                                
                                let frame = AudioFrame {
                                    samples,
                                    channels: 2,
                                    sample_rate: decoder.rate(),
                                    timestamp,
                                };
                                if audio_sender.send(frame).is_err() { return Ok(()); }
                            }
                        }
                    }
                }
            }
        }
    }
    
    writeln!(log, "Pipeline finished.")?;
    Ok(())
}
