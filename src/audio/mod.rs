use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use crate::decode::AudioFrame;
use crossbeam_channel::Receiver;

pub fn run_audio_thread(
    rx: Receiver<AudioFrame>,
    clock: Arc<RwLock<(f64, std::time::Instant)>>,
) -> anyhow::Result<()> {
    let (stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let mut pending_frames = VecDeque::new();

    // Keep the stream alive
    let _alive = stream;

    let update_clock = |pending: &mut VecDeque<(f64, f64)>, s: &Sink, c_lock: &Arc<RwLock<(f64, std::time::Instant)>>| {
        // As sink processes frames, remove them from our tracking queue
        // A sink.len() of N means the LAST N frames appended are still in the sink.
        while pending.len() > s.len() {
            pending.pop_front();
        }

        // Update the master clock with the timestamp of the frame currently at the head of the sink
        if let Some((ts, _)) = pending.front() {
            if let Ok(mut c) = c_lock.write() {
                *c = (*ts, std::time::Instant::now());
            }
        }
    };

    for frame in rx {
        let duration = frame.samples.len() as f64 / (frame.channels as f64 * frame.sample_rate as f64);
        pending_frames.push_back((frame.timestamp, duration));
        
        let source = SamplesBuffer::new(frame.channels, frame.sample_rate, frame.samples);
        sink.append(source);
        
        update_clock(&mut pending_frames, &sink, &clock);

        // Prevent the queue from growing too large and creating massive latency
        while sink.len() > 3 {
            std::thread::sleep(std::time::Duration::from_millis(5));
            update_clock(&mut pending_frames, &sink, &clock);
        }
    }

    sink.sleep_until_end();
    Ok(())
}
