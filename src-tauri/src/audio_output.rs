use crate::audio::server::AudioServer;
use crate::audio::systems::{DrumMachineSystem, AuditionerSystem};
use crate::commands::{AudioCommand, AudioCommandReceiver};
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

        // Create audio server with both systems
        let mut audio_server = AudioServer::new(sample_rate);
        
        // Create and add drum machine system
        let drum_machine_system = DrumMachineSystem::new(sample_rate, event_sender.clone());
        audio_server.add_system("drum_machine".to_string(), Box::new(drum_machine_system));
        
        // Create and add auditioner system
        let auditioner_system = AuditionerSystem::new(sample_rate);
        audio_server.add_system("auditioner".to_string(), Box::new(auditioner_system));
        
        // Start with drum machine as default
        audio_server.switch_to_system("drum_machine").unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::run::<f32>(
                &device,
                &config.into(),
                audio_server,
                command_receiver,
            )?,
            cpal::SampleFormat::I16 => Self::run::<i16>(
                &device,
                &config.into(),
                audio_server,
                command_receiver,
            )?,
            cpal::SampleFormat::U16 => Self::run::<u16>(
                &device,
                &config.into(),
                audio_server,
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
        audio_server: AudioServer,
        command_receiver: AudioCommandReceiver,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            config,
            {
                let mut audio_server = audio_server;
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    // Process pending commands at the start of the buffer
                    command_receiver.process_commands(|command| {
                        match command {
                            AudioCommand::SendNodeEvent {
                                system_name,
                                node_name,
                                event_name,
                                parameter,
                            } => {
                                if let Err(e) = audio_server.send_node_event(&system_name, &node_name, &event_name, parameter) {
                                    eprintln!("Error sending node event: {}", e);
                                }
                            }
                            AudioCommand::SwitchSystem(system_name) => {
                                if let Err(e) = audio_server.switch_to_system(&system_name) {
                                    eprintln!("Error switching system: {}", e);
                                }
                            }
                            AudioCommand::SetSequence {
                                system_name,
                                sequence_data,
                            } => {
                                if let Some(system) = audio_server.get_system_mut(&system_name) {
                                    if let Err(e) = system.set_sequence(&sequence_data) {
                                        eprintln!("Error setting sequence: {}", e);
                                    }
                                } else {
                                    eprintln!("System '{}' not found", system_name);
                                }
                            }
                        }
                    });

                    // Process all frames
                    for frame in data.chunks_mut(channels) {
                        let (left, right) = audio_server.process_stereo(0.0, 0.0);

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
