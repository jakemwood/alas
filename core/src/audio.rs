use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use tokio::runtime::Handle;
use tokio::sync::broadcast::Sender;

use crate::RidgelineMessage;
use crate::RidgelineMessage::VolumeChange;
use tokio::task::JoinHandle;
use tokio::{signal, task};

struct AudioState {
    audio_present: bool,
    // audio_last_seen,  # for tracking when to go "off-air"
    is_streaming: bool,
    is_recording: bool,
}

/// Starts the thread for handling audio.
///
/// This closure will hold the state for all things related to audio, including
/// if there is currently audio flowing through the system, how long it has been
/// silent, etc. It will also start and stop the Icecast thread based on these
/// times.
pub fn start(bus: Sender<RidgelineMessage>) -> JoinHandle<Result<(), ()>> {
    let handler = Handle::current();
    task::spawn_blocking(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No default sound card");
        let config = device.default_input_config().unwrap();

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        println!("Default sample rate: {:?}", config.sample_rate());
        println!("Default sample format: {:?}", config.sample_format());
        println!(
            "Default sample size: {:?}",
            config.sample_format().sample_size()
        );

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| handle_samples::<i8, i8>(data, &bus),
                    err_fn,
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::I16 => device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| handle_samples::<i16, i16>(data, &bus),
                    err_fn,
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::I32 => device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| handle_samples::<i32, i32>(data, &bus),
                    err_fn,
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| handle_samples::<f32, f32>(data, &bus),
                    err_fn,
                    None,
                )
                .unwrap(),
            sample_format => panic!("Unsupported sample format: {:?}", sample_format),
        };

        stream.play().expect("Could not play");
        println!("After play!");

        handler.block_on(async {
            signal::ctrl_c()
                .await
                .expect("failed to listen for exit event");
        });
        println!("After sleep!");

        Ok(())
    })
}

fn handle_samples<T, U>(input: &[T], bus: &Sender<RidgelineMessage>)
where
    T: Sample,
    U: Sample,
{
    let (left, right) = calculate_rms_levels(&input, 2);
    bus.send(VolumeChange { left, right })
        .expect("Could not update volume");
}

fn calculate_rms_levels<T>(data: &[T], channels: usize) -> (f32, f32)
where
    T: cpal::Sample,
{
    let mut left_sum = 0.0;
    let mut right_sum = 0.0;
    let mut left_count = 0;
    let mut right_count = 0;

    for (i, sample) in data.iter().enumerate() {
        let value = sample.to_float_sample().to_sample::<f32>();
        if i % channels == 0 {
            left_sum += value * value;
            left_count += 1;
        } else if i % channels == 1 {
            right_sum += value * value;
            right_count += 1;
        }
    }

    let left_rms = (left_sum / left_count as f32).sqrt();
    let right_rms = (right_sum / right_count as f32).sqrt();

    // Convert to decibels
    let min_db = -60.0;
    let left_db = if left_rms > 0.0 {
        20.0 * left_rms.log10()
    } else {
        min_db
    };

    let right_db = if right_rms > 0.0 {
        20.0 * right_rms.log10()
    } else {
        min_db
    };

    (left_db, right_db)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64::consts::PI;

    fn generate_sine_wave_pcm(
        frequency: f64,
        sample_rate: u32,
        duration: f64,
        amplitude: i16,
    ) -> Vec<i16> {
        let num_samples = (sample_rate as f64 * duration) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for n in 0..num_samples {
            let t = n as f64 / sample_rate as f64;
            let sample_value = (amplitude as f64 * (2.0 * PI * frequency * t).sin()).round();
            samples.push(sample_value as i16);
        }

        samples
    }

    #[test]
    fn test_rms() {
        let quiet_samples = generate_sine_wave_pcm(440.0, 44100, 1.0, 32767 / 2);
        let loud_samples = generate_sine_wave_pcm(440.0, 44100, 1.0, 32767);

        let (quiet_rms, _) = calculate_rms_levels(&quiet_samples, 1);
        let (loud_rms, _) = calculate_rms_levels(&loud_samples, 1);

        assert!(quiet_rms > -60.0);
        assert!(loud_rms > -60.0);
        assert!(quiet_rms < loud_rms);
    }
}
