use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use std::sync::{Arc, RwLock};
use crate::decode::AudioFrame;
use crossbeam_channel::Receiver;

pub fn run_audio_thread(
    rx: Receiver<AudioFrame>,
    clock: Arc<RwLock<Option<(f64, std::time::Instant)>>>,
) -> anyhow::Result<()> {
    let (stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let mut playback_started = false;

    // Keep the stream alive
    let _alive = stream;

    for frame in rx {
        let reset_clock = !playback_started || sink.empty();
        let timestamp = frame.timestamp;
        let source = SamplesBuffer::new(frame.channels, frame.sample_rate, frame.samples);
        sink.append(source);

        if reset_clock {
            if let Ok(mut c) = clock.write() {
                *c = Some((timestamp, std::time::Instant::now()));
            }
            playback_started = true;
        }

        // Prevent the queue from growing too large and creating massive latency
        while sink.len() > 3 {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    sink.sleep_until_end();
    Ok(())
}
