use crate::audio::systems::DrumMachine;
use crate::commands::AudioCommandReceiver;
use crate::events::AudioEventSender;
use cpal::{traits::*, Sample};

pub struct AudioOutput {
    _stream: cpal::Stream,
}

impl AudioOutput {
    pub fn new(
        command_receiver: AudioCommandReceiver,
        event_sender: AudioEventSender,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No output device available")?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;

        println!("Audio device sample rate: {}", sample_rate);

        // Create drum machine with the actual device sample rate and event sender
        let drum_machine = DrumMachine::new(sample_rate, event_sender);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::run::<f32>(
                &device,
                &config.into(),
                drum_machine,
                command_receiver,
            )?,
            cpal::SampleFormat::I16 => Self::run::<i16>(
                &device,
                &config.into(),
                drum_machine,
                command_receiver,
            )?,
            cpal::SampleFormat::U16 => Self::run::<u16>(
                &device,
                &config.into(),
                drum_machine,
                command_receiver,
            )?,
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        Ok(AudioOutput { _stream: stream })
    }

    fn run<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        drum_machine: DrumMachine,
        command_receiver: AudioCommandReceiver,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            config,
            {
                let mut drum_machine = drum_machine;
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    // Process pending commands at the start of the buffer
                    command_receiver.process_commands(|command| {
                        drum_machine.apply_command(command);
                    });

                    // Process all frames
                    for frame in data.chunks_mut(channels) {
                        let (left, right) = drum_machine.next_sample();

                        // Limiting and NaN protection
                        let left = if left.is_finite() {
                            left.clamp(-0.95, 0.95)
                        } else {
                            0.0
                        };
                        let right = if right.is_finite() {
                            right.clamp(-0.95, 0.95)
                        } else {
                            0.0
                        };

                        if channels >= 2 {
                            frame[0] = T::from_sample(left);
                            frame[1] = T::from_sample(right);
                        } else {
                            frame[0] = T::from_sample((left + right) * 0.5);
                        }

                        // Zero out any additional channels
                        for sample in frame.iter_mut().skip(2) {
                            *sample = T::from_sample(0.0);
                        }
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }
}
