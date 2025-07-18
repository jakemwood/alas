use std::fmt::format;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait };
use cpal::{BufferSize, Sample, StreamConfig, SupportedStreamConfig};
use mp3lame_encoder::{ DualPcm, Encoder, FlushNoGap };
use shout::{ ShoutConn };
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::sync::Arc;
use std::time::{ SystemTime, UNIX_EPOCH };
use tokio::runtime::Handle;
use tokio::sync::broadcast::Sender;

use crate::state::AlasMessage::VolumeChange;
use crate::state::{ AlasMessage, AlasState, SafeState };
use bus::{Bus, BusReader};
use tokio::task::JoinHandle;
use tokio::{ select, task };
use tokio::sync::RwLock;
use crate::dropbox::upload_file_to_dropbox;

/// Starts the thread for handling audio.
///
/// This closure will hold the state for all things related to audio, including
/// if there is currently audio flowing through the system, how long it has been
/// silent, etc. It will also start and stop the Icecast thread based on these
/// times.
pub async fn start(
    bus: Sender<AlasMessage>,
    alas_state: &SafeState
) -> JoinHandle<(JoinHandle<()>, JoinHandle<&'static str>, JoinHandle<&'static str>)> {
    let handler = Handle::current();
    let alas_state = alas_state.clone();

    task::spawn_blocking(move || {
        let mut is_recording = false;
        let mut desire_to_broadcast = Arc::new(AtomicBool::new(false));
        let mut config_reset = Arc::new(AtomicBool::new(false));
        let mut audio_last_seen = UNIX_EPOCH;

        let mut audio_bus = Bus::<Vec<f32>>::new(2204 * 30);

        // Config watch
        let mut subscriber = bus.subscribe();
        let config_reset_watch = config_reset.clone();
        let config_thread = task::spawn(async move {
            loop {
                let msg = subscriber.recv().await;
                if let Ok(msg) = msg {
                    match msg {
                        AlasMessage::StreamingConfigUpdated => {
                            // Switch off the desire to broadcast to kill the loop
                            config_reset_watch.store(true, Ordering::Relaxed);
                        }
                        AlasMessage::Exit => {
                            println!("✅ Exiting config thread!");
                            return;
                        }
                        _ => {}
                    }
                } else {
                    println!("❌ Exiting config loop on an error");
                    break;
                }
            }
            println!("✅ Exited config thread");
        });

        // Icecast streaming thread
        let icecast_rx = audio_bus.add_rx();
        let icecast = start_icecast_thread(
            icecast_rx,
            desire_to_broadcast.clone(),
            alas_state.clone(),
            bus.clone(),
            config_reset.clone()
        );
        //
        // // File saving thread
        let file_rx = audio_bus.add_rx();
        let record = start_file_save_thread(
            file_rx,
            desire_to_broadcast.clone(),
            bus.clone()
        );

        let host = cpal::default_host();
        let device = host.default_input_device().expect("No default sound card");

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let mut exit_bus = bus.subscribe();

        let stream_config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(48_000),
            buffer_size: BufferSize::Default,
        };
        let stream = match device
            .build_input_stream(
                &stream_config,
                move |data, _: &_| {
                    handle_samples::<f32>(
                        data,
                        &bus,
                        &alas_state,
                        &mut desire_to_broadcast,
                        &mut audio_last_seen,
                        &mut audio_bus
                    )
                },
                err_fn,
                None
            ) {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Failed to build input stream: {}", e);
                    panic!("Failed to build input stream: {}", e);
                }
            };

        stream.play().expect("Could not play");
        println!("After play!");

        handler.block_on(async move {
            loop {
                select! {
                    message = exit_bus.recv() => {
                        match message {
                            Ok(AlasMessage::Exit) => {
                                return;
                            },
                            Err(e) => {
                                eprintln!("{:?}", e);
                                return;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
        println!("Received exit message in audio thread...");

        (config_thread, icecast, record)
    })
}

fn start_file_save_thread(
    mut file_rx: BusReader<Vec<f32>>,
    desire_to_broadcast: Arc<AtomicBool>,
    file_bus: Sender<AlasMessage>
) -> JoinHandle<&'static str> {
    let mut is_recording = false;

    task::spawn_blocking(move || {
        // return "Abandoned early!!";
        // TODO(config)
        let mut mp3_encoder = mp3lame_encoder::Builder::new().expect("Could not create LAME");
        mp3_encoder.set_num_channels(2).expect("set channels"); // TODO(config)
        mp3_encoder
            .set_sample_rate(48_000) // TODO(config)
            .expect("set sample rate");
        mp3_encoder
            .set_brate(mp3lame_encoder::Bitrate::Kbps320) // TODO(config
            .expect("set brate");
        let mut mp3_encoder = mp3_encoder.build().expect("Could not init LAME");

        loop {
            let mut input = match file_rx.recv() {
                Ok(input) => input,
                Err(_) => {
                    break;
                }
            };

            if desire_to_broadcast.load(Ordering::Relaxed) {
                let (mut recording_file, file_path) = open_file_named_now();

                while desire_to_broadcast.load(Ordering::Relaxed) {
                    let mp3_buffer = make_mp3_samples(&mut mp3_encoder, &input);
                    match recording_file.write_all(&mp3_buffer) {
                        Ok(_) => {
                            // Transition the state if we're not already set to recording
                            if !&is_recording {
                                is_recording = true;
                                // Send message to bus that we are recording
                                let _ = &file_bus.send(AlasMessage::RecordingStarted).unwrap();
                            }
                        }
                        Err(err) => {
                            eprintln!("Error writing to file: {:?} 174", err);
                            is_recording = false;
                            let _ = &file_bus.send(AlasMessage::RecordingStopped).unwrap();
                        }
                    }

                    input = match file_rx.recv() {
                        Ok(input) => input,
                        Err(_) => {
                            break;
                        }
                    };
                }

                is_recording = false;
                let _ = &file_bus.send(AlasMessage::RecordingStopped);
                println!("Stopped recording");

                // Upload the file to Dropbox.
                upload_file_to_dropbox(file_path, "".to_string(), file_bus.clone());
            }
        }

        let _ = &file_bus.send(AlasMessage::RecordingStopped);
        "✅ Exiting file write thread"
    })
}

fn start_icecast_thread(
    mut icecast_rx: BusReader<Vec<f32>>,
    desire_to_broadcast: Arc<AtomicBool>,
    state: Arc<RwLock<AlasState>>,
    message_bus: Sender<AlasMessage>,
    config_reset: Arc<AtomicBool>
) -> JoinHandle<&'static str> {
    task::spawn_blocking(move || {
        // return "Abandoned early!!";
        // Set up the MP3 encoder.
        let mut mp3_encoder = mp3lame_encoder::Builder::new().expect("Could not create LAME");
        mp3_encoder.set_num_channels(2).expect("set channels"); // TODO(config)
        mp3_encoder
            .set_sample_rate(48_000) // TODO(config)
            .expect("set sample rate");
        mp3_encoder
            .set_brate(mp3lame_encoder::Bitrate::Kbps128) // TODO(config)
            .expect("set brate");
        let mut mp3_encoder = mp3_encoder.build().expect("Could not init LAME");

        loop {
            let mut input = match icecast_rx.recv() {
                Ok(input) => input,
                Err(_) => {
                    break;
                }
            };

            if desire_to_broadcast.load(Ordering::Relaxed) {
                let icecast_connection = connect_to_icecast(&state);
                config_reset.store(false, Ordering::Relaxed);

                while
                    desire_to_broadcast.load(Ordering::Relaxed) &&
                    !config_reset.load(Ordering::Relaxed)
                {
                    let mp3_buffer = make_mp3_samples(&mut mp3_encoder, &input);

                    match icecast_connection.send(&mp3_buffer) {
                        Ok(_) => {
                            if !state.blocking_read().is_streaming {
                                let mut mutable_state = state.blocking_write();
                                mutable_state.is_streaming = true;
                                let _ = message_bus.send(AlasMessage::StreamingStarted);
                            }
                        }
                        Err(_err) => {
                            let mut mutable_state = state.blocking_write();
                            mutable_state.is_streaming = false;
                            let _ = message_bus.send(AlasMessage::StreamingStopped);

                            // Attempt to reconnect
                            match icecast_connection.reconnect() {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("Icecast re-connect error: {:?}", e);
                                }
                            }
                        }
                    }

                    input = match icecast_rx.recv() {
                        Ok(input) => input,
                        Err(_) => {
                            break;
                        }
                    };
                    // TODO: handle graceful exit
                }

                let mut mutable_state = state.blocking_write();
                mutable_state.is_streaming = false;
                let _ = message_bus.send(AlasMessage::StreamingStopped);
            }

            // TODO: handle graceful exit
        }

        let mut mutable_state = state.blocking_write();
        mutable_state.is_streaming = false;
        let _ = message_bus.send(AlasMessage::StreamingStopped);
        println!("Closed Icecast streaming thread");

        "✅ Success! Returned out of Icecast thread!"
    })
}

fn make_mp3_samples<T>(mp3_encoder: &mut Encoder, input: &[T]) -> Vec<u8> where T: Sample {
    let mut left_channel = Vec::new();
    let mut right_channel = Vec::new();

    for (i, sample) in input.iter().enumerate() {
        if i % 2 == 0 {
            left_channel.push(float_to_i16(sample.to_float_sample().to_sample::<f32>()));
        } else {
            right_channel.push(float_to_i16(sample.to_float_sample().to_sample::<f32>()));
        }
    }
    let data = DualPcm {
        left: &*left_channel,
        right: &*right_channel,
    };

    let mut mp3_buffer = Vec::new();
    mp3_buffer.reserve(mp3lame_encoder::max_required_buffer_size(data.left.len()));
    let encoded_size = mp3_encoder.encode(data, mp3_buffer.spare_capacity_mut()).expect("Encode");
    // TODO: surely there is a way to do this safely without offending mp3s?
    unsafe {
        mp3_buffer.set_len(mp3_buffer.len().wrapping_add(encoded_size));
    }
    // mp3_buffer.resize(mp3_buffer.len() + encoded_size, 0);

    let encoded_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_buffer.spare_capacity_mut())
        .expect("to flush");
    unsafe {
        mp3_buffer.set_len(mp3_buffer.len().wrapping_add(encoded_size));
    }

    mp3_buffer
}

fn open_file_named_now() -> (File, String) {
    let formatted_time = chrono::Local::now().format("%Y-%m-%dT%H%M%S.mp3").to_string();
    (File::create(&formatted_time).expect("to open file"), formatted_time)
}

fn connect_to_icecast(state: &SafeState) -> ShoutConn {
    println!("Connection attempt!");
    loop {
        let state = state.blocking_read();
        let config = &state.config.icecast;
        println!("Connecting to {:} {:}", config.hostname, config.mount);
        let connection = shout::ShoutConnBuilder
            ::new()
            .host(config.hostname.clone())
            .port(config.port)
            .user(String::from("source"))
            .password(config.password.clone())
            .mount(config.mount.clone())
            .protocol(shout::ShoutProtocol::HTTP)
            .format(shout::ShoutFormat::MP3)
            .build();
        if let Ok(connection) = connection {
            return connection;
        } else {
            // Sleep for 3 seconds and try re-connecting
            println!("Sleeping for 3 seconds and then re-trying our connection");
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }
}

fn float_to_i16(sample: f32) -> i16 {
    // First clamp to the valid normalized range just in case
    let clamped = sample.clamp(-1.0, 1.0);
    // Map from [-1.0, 1.0] to [-32768, 32767] (i16 range)
    // Multiplying by i16::MAX (32767) handles positive values correctly,
    // and negative values are safely converted as well.
    (clamped * (i16::MAX as f32)) as i16
}

fn handle_samples<T>(
    input: &[T],
    bus: &Sender<AlasMessage>,
    state: &SafeState,
    desire_to_broadcast: &AtomicBool,
    audio_last_seen: &mut SystemTime,
    sender: &mut Bus<Vec<T>>
)
    where T: Sample
{
    let channels = 2;
    let (left, right) = calculate_rms_levels(&input, channels);
    let _ = &bus.send(VolumeChange { left, right }).expect("Could not update volume");

    let read_state = match state.try_read() {
        Ok(guard) => guard.clone(),
        Err(_) => return, // Skip if can't acquire lock
    };

    if
        left > read_state.config.audio.silence_threshold ||
        right > read_state.config.audio.silence_threshold
    {
        // TODO(config)
        if !desire_to_broadcast.load(Ordering::Relaxed) {
            println!("Audio is now available!");
            desire_to_broadcast.store(true, Ordering::Relaxed);

            if let Ok(mut state) = state.try_write() {
                (*state).is_audio_present = true;
            }
        }
        *audio_last_seen = SystemTime::now();
    } else if
        // Before we do anything else, verify that we should still be recording/streaming
        SystemTime::now().duration_since(*audio_last_seen).unwrap().as_secs() >
        (read_state.config.audio.silence_duration_before_deactivation as u64)
    {
        if desire_to_broadcast.load(Ordering::Relaxed) {
            println!("There has been 15 seconds of silence!");
        }
        desire_to_broadcast.store(false, Ordering::Relaxed);
        if let Ok(mut state) = state.try_write() {
            (*state).is_audio_present = false;
        }
    }

    sender.broadcast(input.to_vec().clone());
}

fn calculate_rms_levels<T>(data: &[T], channels: usize) -> (f32, f32) where T: cpal::Sample {
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

    let left_rms = (left_sum / (left_count as f32)).sqrt();
    let right_rms = (right_sum / (right_count as f32)).sqrt();

    // Convert to decibels
    let min_db = -60.0;
    let left_db = if left_rms > 0.0 { 20.0 * left_rms.log10() } else { min_db };

    let right_db = if right_rms > 0.0 { 20.0 * right_rms.log10() } else { min_db };

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
        amplitude: i16
    ) -> Vec<i16> {
        let num_samples = ((sample_rate as f64) * duration) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for n in 0..num_samples {
            let t = (n as f64) / (sample_rate as f64);
            let sample_value = ((amplitude as f64) * (2.0 * PI * frequency * t).sin()).round();
            samples.push(sample_value as i16);
        }

        samples
    }

    #[test]
    fn test_rms() {
        let quiet_samples = generate_sine_wave_pcm(440.0, 48_000, 1.0, 32767 / 2);
        let loud_samples = generate_sine_wave_pcm(440.0, 48_000, 1.0, 32767);

        let (quiet_rms, _) = calculate_rms_levels(&quiet_samples, 1);
        let (loud_rms, _) = calculate_rms_levels(&loud_samples, 1);

        assert!(quiet_rms > -60.0);
        assert!(loud_rms > -60.0);
        assert!(quiet_rms < loud_rms);
    }
}

/*
           cpal::SampleFormat::I8 => device
               .build_input_stream(
                   &config.into(),
                   move |data, _: &_| {
                       handle_samples::<i8>(data, &bus, &mut mp3_encoder, &mut output_file, &conn)
                   },
                   err_fn,
                   None,
               )
               .unwrap(),
           cpal::SampleFormat::I16 => device
               .build_input_stream(
                   &config.into(),
                   move |data, _: &_| {
                       handle_samples::<i16>(data, &bus, &mut mp3_encoder, &mut output_file, &conn)
                   },
                   err_fn,
                   None,
               )
               .unwrap(),
           cpal::SampleFormat::I32 => device
               .build_input_stream(
                   &config.into(),
                   move |data, _: &_| {
                       handle_samples::<i32>(data, &bus, &mut mp3_encoder, &mut output_file, &conn)
                   },
                   err_fn,
                   None,
               )
               .unwrap(),
*/
