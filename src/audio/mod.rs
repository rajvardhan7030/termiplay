use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use crate::decode::AudioFrame;
use crossbeam_channel::Receiver;

pub fn run_audio_thread(
    rx: Receiver<AudioFrame>,
    clock: Arc<RwLock<f64>>,
) -> anyhow::Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let mut pending_frames = VecDeque::new();

    for frame in rx {
        let duration = frame.samples.len() as f64 / (frame.channels as f64 * frame.sample_rate as f64);
        pending_frames.push_back((frame.timestamp, duration));
        
        let source = SamplesBuffer::new(frame.channels, frame.sample_rate, frame.samples);
        sink.append(source);
        
        // As sink processes frames, remove them from our tracking queue
        while sink.len() < pending_frames.len() {
            pending_frames.pop_front();
        }

        // Update the master clock with the timestamp of the frame currently at the head of the sink
        if let Some((ts, _)) = pending_frames.front() {
            if let Ok(mut c) = clock.write() {
                *c = *ts;
            }
        }

        // Prevent the queue from growing too large and creating massive latency
        while sink.len() > 3 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    sink.sleep_until_end();
    Ok(())
}
