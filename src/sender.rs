use cpal::{self, Sample};

use serenity::prelude::Mutex;
use serenity::voice::AudioSource;
use serenity::voice::AudioType;

use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;

pub struct Sender(Arc<Mutex<VecDeque<i16>>>);

impl Sender {
    pub fn new() -> Sender {
        let buffer: Arc<Mutex<VecDeque<i16>>> = Arc::new(Mutex::new(VecDeque::new()));

        let device = cpal::default_input_device().expect("Failed to get default input device");
        let format = device
            .default_input_format()
            .expect("Failed to get default input format");
        let discord_format = cpal::Format {
            sample_rate: cpal::SampleRate(48000),
            ..format
        };
        println!("{:#?}", discord_format);
        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop
            .build_input_stream(&device, &discord_format)
            .expect("Failed to build input stream");
        event_loop.play_stream(stream_id);

        let local_buf = Arc::clone(&buffer);
        thread::spawn(move || {
            event_loop.run(move |_, data| match data {
                cpal::StreamData::Input {
                    buffer: cpal::UnknownTypeInputBuffer::U16(buffer),
                } => {
                    local_buf.lock().extend(buffer.iter().map(Sample::to_i16));
                }
                cpal::StreamData::Input {
                    buffer: cpal::UnknownTypeInputBuffer::I16(buffer),
                } => {
                    local_buf.lock().extend(buffer.iter());
                }
                cpal::StreamData::Input {
                    buffer: cpal::UnknownTypeInputBuffer::F32(buffer),
                } => {
                    local_buf.lock().extend(buffer.iter().map(Sample::to_i16));
                }
                _ => (),
            });
        });

        Sender(buffer)
    }
}

impl AudioSource for Sender {
    fn is_stereo(&mut self) -> bool {
        true
    }

    fn get_type(&self) -> AudioType {
        AudioType::Pcm
    }

    fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        let mut buf = self.0.lock();
        let mut c = 0;
        for out in buffer.iter_mut() {
            let shifted_input = if let Some(v) = buf.pop_front() {
                c += 1;
                v
            } else {
                0
            };
            *out = shifted_input;
        }

        if c > 0 {
            Some(c)
        } else {
            None
        }
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        unimplemented!()
    }
}
