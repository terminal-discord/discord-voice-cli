use serenity::prelude::Mutex;
use serenity::voice::AudioReceiver;

use cpal::{self, Sample};

use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;

pub struct Receiver(Arc<Mutex<VecDeque<i16>>>);

impl Receiver {
    pub fn new() -> Receiver {
        let buffer: Arc<Mutex<VecDeque<i16>>> = Arc::new(Mutex::new(VecDeque::new()));

        let device = cpal::default_output_device().expect("Failed to get default output device");
        let format = device
            .default_output_format()
            .expect("Failed to get default output format");
        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        event_loop.play_stream(stream_id.clone());

        let local_buf = Arc::clone(&buffer);
        thread::spawn(move || {
            event_loop.run(move |_, data| match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let mut buf = local_buf.lock();
                        for out in sample.iter_mut() {
                            let value = buf.pop_front().unwrap_or(0);
                            *out = value.to_u16();
                        }
                    }
                }
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let mut buf = local_buf.lock();
                        for out in sample.iter_mut() {
                            let value = buf.pop_front().unwrap_or(0);
                            *out = value;
                        }
                    }
                }
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let mut buf = local_buf.lock();
                        for out in sample.iter_mut() {
                            let value = buf.pop_front().unwrap_or(0);
                            *out = value.to_f32();
                        }
                    }
                }
                _ => (),
            });
        });
        Receiver(buffer)
    }
}

impl AudioReceiver for Receiver {
    fn speaking_update(&mut self, _ssrc: u32, _user_id: u64, _speaking: bool) {}

    fn voice_packet(
        &mut self,
        _ssrc: u32,
        _sequence: u16,
        _timestamp: u32,
        _stereo: bool,
        data: &[i16],
    ) {
        self.0.lock().extend(data);
    }
}
