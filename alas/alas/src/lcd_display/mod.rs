use crate::lcd_display::home_screen::HomeScreen;
use crate::lcd_display::matrix_orbital::clear_screen;
use alas_lib::state::AlasMessage;
use alas_lib::state::AlasState;
use alas_lib::state::SafeState;
use rocket::futures::AsyncWriteExt;
use rocket::yansi::Paint;
use screen::Screen;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::{join, select, signal, task};
use udev::Enumerator;

mod home_screen;
mod ip_screen;
mod matrix_orbital;
mod menu_screen;
mod screen;
mod upload_progress;
// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>());
// }

fn random_number() -> u16 {
    use rand::Rng;
    rand::rng().random_range(1000..9999)
}

async fn handle_message(
    current_state: DisplayState,
    app_state: SafeState,
    message: AlasMessage,
    write_port: &mut Box<dyn SerialPort>
) {
    // Some messages are not screen-specific. Handle those here.
    match message {
        AlasMessage::RecordingStarted => {
            let (is_recording, is_streaming) = {
                let mut state = app_state.write().await;
                (*state).is_recording = true;
                (state.is_recording, state.is_streaming)
            }; // Write lock released here
            change_on_air_lights_direct(is_recording, is_streaming, write_port);
        }
        AlasMessage::RecordingStopped => {
            let (is_recording, is_streaming) = {
                let mut state = app_state.write().await;
                (*state).is_recording = false;
                (state.is_recording, state.is_streaming)
            }; // Write lock released here
            change_on_air_lights_direct(is_recording, is_streaming, write_port);
        }
        AlasMessage::StreamingStarted => {
            let (is_recording, is_streaming) = {
                let mut state = app_state.write().await;
                (*state).is_streaming = true;
                (state.is_recording, state.is_streaming)
            }; // Write lock released here
            change_on_air_lights_direct(is_recording, is_streaming, write_port);
        }
        AlasMessage::StreamingStopped => {
            let (is_recording, is_streaming) = {
                let mut state = app_state.write().await;
                (*state).is_streaming = false;
                (state.is_recording, state.is_streaming)
            }; // Write lock released here
            change_on_air_lights_direct(is_recording, is_streaming, write_port);
        }
        _ => {}
    }
    // At this point our write locks should be released, making way for a read lock below.

    let random = random_number();
    let mut screen = current_state.write().await;

    // Clone this state so that we can release our lock quickly
    let app_state = app_state.read().await.clone();

    let new_screen = (*screen).handle_message(&app_state, message);

    if let Some(new_screen) = new_screen {
        if new_screen.as_any().type_id() != (*screen).as_any().type_id() {
            // Clear the screen
            println!("📺 Clearing the screen!! You should not see this message very often!");
            // print_type_of(&new_screen.as_any());
            // print_type_of(&(*screen).as_any());
            clear_screen(write_port).unwrap();
            new_screen.draw_screen(write_port);
        } else {
            // If the two states are identical, we do not need to redraw the screen
            new_screen.redraw_screen(write_port);
        }
        *screen = new_screen;
    }
}

fn change_on_air_lights(state: &AlasState, write_port: &mut Box<dyn SerialPort>) {
    change_on_air_lights_direct(state.is_recording, state.is_streaming, write_port);
}

fn change_on_air_lights_direct(is_recording: bool, is_streaming: bool, write_port: &mut Box<dyn SerialPort>) {
    if is_recording {
        write_port.write_all(&[254, 87, 5]).unwrap();
    } else {
        write_port.write_all(&[254, 86, 5]).unwrap();
    }
    if is_streaming {
        write_port.write_all(&[254, 87, 3]).unwrap();
    } else {
        write_port.write_all(&[254, 86, 3]).unwrap();
    }
}

async fn handle_button(
    display_state: DisplayState,
    app_state: &SafeState,
    button_pressed: u8,
    port: &mut Box<dyn SerialPort>
) {
    println!("📺 Button pressed: {:?}", button_pressed);
    let mut screen = display_state.write().await;
    let app_state = app_state.read().await;
    let new_screen = (*screen).handle_button(&app_state, button_pressed);
    if let Some(new_screen) = new_screen {
        // print_type_of(&new_screen.as_any());
        // print_type_of(&(*screen).as_any());
        if new_screen.as_any().type_id() != (*screen).as_any().type_id() {
            let _ = matrix_orbital::clear_screen(port);
            new_screen.draw_screen(port);
        } else {
            new_screen.redraw_screen(port);
        }
        *screen = new_screen;
    }
}

fn find_port_name() -> Option<String> {
    let mut enumerator = Enumerator::new().ok().expect("Failed to create enumerator");
    enumerator.match_subsystem("tty").ok().expect("Failed to match");

    // Iterate over each device
    for device in enumerator.scan_devices().ok().expect("Failed to scan devices") {
        // Check if the device node exists and starts with "/dev/ttyUSB"
        if let Some(devnode) = device.devnode() {
            if let Some(devnode_str) = devnode.to_str() {
                if devnode_str.starts_with("/dev/ttyUSB") {
                    // Check the device property for the vendor name
                    if let Some(vendor) = device.property_value("ID_VENDOR") {
                        if vendor.to_str() == Some("MO") {
                            return Some(devnode_str.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

async fn find_port_name_stubborn() -> String {
    loop {
        if let Some(port_name) = find_port_name() {
            return port_name;
        }
        else {
            println!("📺 Sleeping for 10 seconds to find display...");
            select! {
                _ = sleep(Duration::from_secs(10)) => {},
                _ = signal::ctrl_c() => {
                    panic!("Exiting while stubbornly looking for port name!");
                }
            }
        }
    }
}

async fn connect() -> serialport::Result<Box<dyn SerialPort>> {
    println!("Connecting to the serial port...");

    let port_name = find_port_name_stubborn().await;
    let baud_rate = 19200;

    serialport
        ::new(port_name, baud_rate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(100))
        .open()
}

async fn connect_stubborn() -> Box<dyn SerialPort> {
    loop {
        match connect().await {
            Ok(port) => break port,
            Err(e) => {
                eprintln!("📺 Error connecting to display port: {:?}", e);
                select! {
                    _ = sleep(Duration::from_secs(15)) => {},
                    _ = signal::ctrl_c() => {
                        panic!("Exiting while stubbornly looking for display!");
                    }
                }
            }
        }
    }
}

type DisplayState = Arc<RwLock<Box<dyn Screen>>>;

async fn thread_start(
    bus: Sender<AlasMessage>,
    shared_state: SafeState
) {
    loop {
        if try_and_connect_to_serial_port(&bus, &shared_state).await {
            println!("✅ Done with the display!");
            break;
        }
    }
}

async fn try_and_connect_to_serial_port(bus: &Sender<AlasMessage>, shared_state: &SafeState) -> bool {
    let copy_state = shared_state.read().await.clone();
    let display_state: DisplayState = Arc::new(RwLock::new(Box::new(
        HomeScreen::new(&copy_state)
    )));

    let mut port = connect_stubborn().await;
    // port is safe to clone, but ideally have a read/write clone
    // based on the duplex example

    // This task is responsible for listening to state changes sent to us from
    // the event bus and updating the screens, as appropriate.
    let write_state = display_state.clone();
    let write_shared_state = shared_state.clone();
    let lcd_writer = start_writer_thread(
        bus.clone(),
        write_state,
        write_shared_state,
        &mut port,
    );

    // This task is responsible for reading from the USB serial and responding to
    // button presses as needed.
    let read_state = display_state.clone();
    let read_shared_state = shared_state.clone();
    let lcd_reader = start_reader_thread(
        read_state,
        read_shared_state,
        &mut port,
    );

    // Now that everything is started, send the first screen
    matrix_orbital::set_brightness(&mut port, 0.5).expect("Could not set brightness"); // Set brightness to 50%
    // Initialize the horizontal bar graphs (page 22 of the LCD manual)
    port.write_all(&[254, 66, 0]).unwrap(); // turn on display
    port.write_all(&[254, 104]).unwrap(); // load horizontal bars

    // Clear the screen and then draw the first screen *we* want to show
    {
        let first_screen = display_state.read().await;
        println!("Writing the first screen...");
        matrix_orbital::clear_screen(&mut port).expect("Could not clear the screen");
        first_screen.draw_screen(&mut port);
    }

    println!("📖✏️ Waiting for reader and writer threads...");
    let (write_results, read_results) = join!(lcd_writer, lcd_reader);
    println!("📖✏️ Joined on our threads!");

    if write_results.is_ok() && read_results.is_ok() {
        println!("✅ LCD writer and reader exited okay!");
        true
    } else {
        println!("❌ LCD writer/reader crashed: {:?} {:?}", write_results.is_ok(), read_results.is_ok());
        false
    }
}

pub async fn start(
    bus: Sender<AlasMessage>,
    shared_state: &SafeState
) -> JoinHandle<()> {
    let shared_state = shared_state.clone();
    tokio::spawn(async move {
        println!("📺 Starting LCD thread...");
        thread_start(bus, shared_state.clone()).await
    })
}

fn start_reader_thread(
    read_state: DisplayState,
    read_shared_state: SafeState,
    port: &mut Box<dyn SerialPort>
) -> JoinHandle<()> {
    let mut read_port = port.try_clone().expect("Could not create read port");
    task::spawn(async move {
        loop {
            let mut loop_port = read_port.try_clone().expect("Could not create response port");
            select! {
                result = task::spawn_blocking(move || {
                    let mut buf = [0; 1];
                    match loop_port.read(buf.as_mut_slice()) {
                        Ok(_bytes_read) => Ok(buf[0]),
                        Err(e) => Err(e)
                    }
                }) => {
                    match result {
                        Ok(Ok(button_pressed)) => {
                            handle_button(
                                read_state.clone(),
                                &read_shared_state,
                                button_pressed,
                                &mut read_port
                            ).await;
                        },
                        Ok(Err(ref e)) if e.kind() == io::ErrorKind::TimedOut => {
                            // This is expected, this just means the user hasn't pressed
                            // the button in awhile.
                            continue;
                        },
                        Ok(Err(e)) => panic!("Unexpected LCD Error {:?}", e),
                        Err(e) => panic!("Spawn blocking error {:?}", e),
                    }
                },
                _ = signal::ctrl_c() => {
                    println!("📺📖 LCD Reader exiting from ctrl+c!");
                    break;
                }
            }
        }
        println!("📺📖 LCD Reader exiting loop");
    })
}

fn start_writer_thread(
    bus: Sender<AlasMessage>,
    write_state: DisplayState,
    write_shared_state: SafeState,
    port: &mut Box<dyn SerialPort>,
) -> JoinHandle<()> {
    let mut write_port = port.try_clone().expect("Could not create write port");
    let mut lcd_rx = bus.subscribe();
    task::spawn(async move {
        // Now listen for any events that we need in order to process writes to our screen
        println!("📺✏️ LCD writer thread has started...");
        loop {
            select! {
                message = lcd_rx.recv() => {
                    match message {
                        Ok(AlasMessage::Exit) => {
                            println!("📺✏️ LCD writer received exit message...");
                            break;
                        }
                        Ok(message) => {
                            if !matches!(message, AlasMessage::VolumeChange { .. }) {
                                println!("📺✏️ Handling message... {:?}", message);
                            }
                            handle_message(
                                write_state.clone(),
                                write_shared_state.clone(),
                                message,
                                &mut write_port
                            ).await;
                            // println!("📺✅️ Handling message...");
                        }
                        Err(e) => {
                            println!("📺❌ LCD writer error: {:?}", e);
                            break;
                        }
                    }
                },
                _ = signal::ctrl_c() => {
                    println!("📺✏️ LCD writer received Ctrl+c...");
                    break;
                }
            }
        }

        println!("📺✏️ End of LCD Writer loop reached!");
        write_port.write_all(&[254, 70]).unwrap(); // turn off screen
        write_port.write_all(&[254, 86, 5]).unwrap(); // turn off gpio leds
        write_port.write_all(&[254, 86, 3]).unwrap(); // turn off gpio leds
        clear_screen(&mut write_port).unwrap();
        write_port.write_all("Software shutdown...".as_bytes()).unwrap();
    })
}
