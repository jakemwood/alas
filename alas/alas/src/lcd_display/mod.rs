use crate::lcd_display::matrix_orbital::clear_screen;
use alas_lib::state::AlasMessage;
use alas_lib::state::AlasState;
use alas_lib::state::SafeState;
use menu_screen::MenuScreen;
use screen::Screen;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::any::Any;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::{select, signal, task};
use udev::Enumerator;

mod home_screen;
mod ip_screen;
mod matrix_orbital;
mod menu_screen;
mod screen;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

async fn handle_message(
    current_state: DisplayState,
    app_state: &SafeState,
    message: AlasMessage,
    write_port: &mut Box<dyn SerialPort>,
) {
    // Some messages are not screen-specific. Handle those here.
    match message {
        AlasMessage::RecordingStarted => {
            let mut state = app_state.write().await;
            (*state).is_recording = true;
            change_on_air_lights(&state, write_port);
        }
        AlasMessage::RecordingStopped => {
            let mut state = app_state.write().await;
            (*state).is_recording = false;
            change_on_air_lights(&state, write_port);
        }
        AlasMessage::StreamingStarted => {
            let mut state = app_state.write().await;
            (*state).is_streaming = true;
            change_on_air_lights(&state, write_port);
        }
        AlasMessage::StreamingStopped => {
            let mut state = app_state.write().await;
            (*state).is_streaming = false;
            change_on_air_lights(&state, write_port);
        }
        _ => {}
    }
    // At this point our write locks should be released, making way for a read lock below.

    let mut screen = current_state.write().await;

    // Clone this state so that we can release our lock quickly
    let app_state = app_state.read().await.clone();

    let new_screen = (*screen).handle_message(&app_state, message);

    if let Some(new_screen) = new_screen {
        if new_screen.as_any().type_id() != (*screen).as_any().type_id() {
            // Clear the screen
            println!("Clearing the screen!! You should not see this message very often!");
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
    if state.is_recording {
        write_port.write_all(&[254, 87, 5]).unwrap();
    } else {
        write_port.write_all(&[254, 86, 5]).unwrap();
    }
    if state.is_streaming {
        write_port.write_all(&[254, 87, 3]).unwrap();
    } else {
        write_port.write_all(&[254, 86, 3]).unwrap();
    }
}

async fn handle_button(
    display_state: DisplayState,
    app_state: &SafeState,
    button_pressed: u8,
    port: &mut Box<dyn SerialPort>,
) {
    println!("Button pressed: {:?}", button_pressed);
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
                            return Some(devnode_str.to_string())
                        }
                    }
                }
            }
        }
    }

    None
}

fn connect() -> Box<dyn SerialPort> {
    println!("Connecting to the serial port...");

    let port_name = find_port_name().expect("Could not find serial port");
    let baud_rate = 19200;

    serialport::new(port_name, baud_rate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Could not open serial port")
}

type DisplayState = Arc<RwLock<Box<dyn Screen>>>;

pub async fn start(
    mut lcd_rx: Receiver<AlasMessage>,
    shared_state: &SafeState,
) -> (JoinHandle<()>, JoinHandle<()>) {
    let mut display_state: DisplayState = Arc::new(RwLock::new(Box::new(MenuScreen::new())));

    let mut port = connect();
    // port is safe to clone, but ideally have a read/write clone
    // based on the duplex example

    // This task is responsible for listening to state changes sent to us from
    // the event bus and updating the screens, as appropriate.
    let mut write_port = port.try_clone().expect("Could not create write port");
    let write_state = display_state.clone();
    let write_shared_state = shared_state.clone();
    let lcd_writer = task::spawn(async move {
        // Now listen for any events that we need in order to process writes to our screen
        loop {
            select! {
                message = lcd_rx.recv() => {
                    match message {
                        Ok(message) => {
                            handle_message(write_state.clone(), &write_shared_state, message, &mut write_port).await;
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            break;
                        }
                    }
                },
                _ = signal::ctrl_c() => {
                    println!("Ctrl+C detected in LCD writer loop!");
                    break;
                }
            }
        }

        println!("End of LCD Writer loop reached!");
        write_port.write_all(&[254, 70]).unwrap(); // turn off screen
        write_port.write_all(&[254, 86, 5]).unwrap(); // turn off gpio leds
        write_port.write_all(&[254, 86, 3]).unwrap(); // turn off gpio leds
        clear_screen(&mut write_port).unwrap();
        write_port
            .write_all("Software shutdown...".as_bytes())
            .unwrap();
    });

    // This task is responsible for reading from the USB serial and responding to
    // button presses as needed.
    let read_state = display_state.clone();
    let read_shared_state = shared_state.clone();
    let mut read_port = port.try_clone().expect("Could not create read port");
    let lcd_reader = task::spawn(async move {
        loop {
            let mut loop_port = read_port
                .try_clone()
                .expect("Could not create response port");
            // let mut response_port = read_port.try_clone().expect("Could not create response port");
            select! {
                result = task::spawn_blocking(move || {
                    let mut buf = [0; 1];
                    // let bytes_read = read_port.read(buf.as_mut_slice());
                    match loop_port.read(buf.as_mut_slice()) {
                        Ok(bytes_read) => Ok(buf[0]),
                        Err(e) => Err(e)
                    }
                }) => {
                    match result {
                        Ok(Ok(button_pressed)) => {
                            handle_button(read_state.clone(), &read_shared_state, button_pressed, &mut read_port).await;
                        },
                        Ok(Err(ref e)) if e.kind() == io::ErrorKind::TimedOut => {
                            continue;
                        },
                        Ok(Err(e)) => panic!("Unexpected LCD Error {:?}", e),
                        Err(e) => panic!("Spawn blocking error {:?}", e),
                    }
                },
                _ = signal::ctrl_c() => {
                    println!("LCD Reader exiting!");
                    break;
                }
            }
        }
        println!("LCD Reader exiting loop");
    });

    // Now that everything is started, send the first screen
    matrix_orbital::set_brightness(&mut port, 0.5).expect("Could not set brightness"); // Set brightness to 50%
                                                                                       // Initialize the horizontal bar graphs (page 22 of the LCD manual)
    port.write_all(&[254, 66, 0]).unwrap(); // turn on display
    port.write_all(&[254, 104]).unwrap(); // load horizontal bars

    // Clear the screen and then draw the first screen *we* want to show
    let first_screen = display_state.read().await;
    println!("Writing the first screen...");
    matrix_orbital::clear_screen(&mut port).expect("Could not clear the screen");
    first_screen.draw_screen(&mut port);

    (lcd_reader, lcd_writer)
}
