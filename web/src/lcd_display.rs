use crate::{SafeState, UnsafeState};
use core::RidgelineMessage;
use rocket::yansi::Paint;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::any::Any;
use std::cmp::{max, min};
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::{select, signal, task};

const SCREEN_WIDTH: u8 = 20;

/**
On input:

A = Top left button
B = Up button
D = Left button
E = Center button
C = Right button
G = Bottom left button
H = Bottom button
**/
const TOP_LEFT_BUTTON: u8 = 65;
const UP_BUTTON: u8 = 66;
const LEFT_BUTTON: u8 = 68;
const CENTER_BUTTON: u8 = 69;
const RIGHT_BUTTON: u8 = 67;
const DOWN_BUTTON: u8 = 72;
const BOTTOM_LEFT_BUTTON: u8 = 71;

trait Screen: Send + Sync + Any {
    // Draw the screen from scratch
    fn draw_screen(&self, port: &mut Box<dyn SerialPort>);

    // Update only the parts of the screen that have changed
    // TODO: do we need old state?
    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>);

    // Handle a button being pressed
    fn handle_button(&self, app_state: &UnsafeState, button: u8) -> Option<Box<dyn Screen>>;

    // Handle an incoming message from the bus
    fn handle_message(
        &self,
        app_state: &UnsafeState,
        message: RidgelineMessage,
    ) -> Option<Box<dyn Screen>>;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone)]
struct HomeScreen {
    wifi_ready: bool,
    cell_ready: bool,
    left_volume: u8,
    right_volume: u8,
}

impl HomeScreen {
    fn new(app_state: &UnsafeState) -> Self {
        HomeScreen {
            wifi_ready: app_state.wifi_on,
            cell_ready: app_state.cell_on,
            left_volume: 0,
            right_volume: 0,
        }
    }
}

fn scale_db_to_display(db: f32) -> u8 {
    // Define the input dB range and the desired output range
    let min_db = -60.0;
    let max_db = 0.0;
    let min_scale = 0.0;
    let max_scale = 80.0;

    // Clamp the input dB value to ensure it falls within the expected range
    let db_clamped = db.clamp(min_db, max_db);

    // Compute the ratio of where db_clamped falls between min_db and max_db
    let ratio = (db_clamped - min_db) / (max_db - min_db);

    // Scale this ratio to the display range (0 to 80)
    let scaled = ratio * (max_scale - min_scale) + min_scale;

    // Round the scaled value and convert to integer
    scaled.round() as u8
}

impl Screen for HomeScreen {
    fn draw_screen(&self, port: &mut Box<dyn SerialPort>) {
        port.write_all("88.7 RIDGELINE RADIO".as_bytes()).unwrap();
        port.write_all("Wi-Fi? ".as_bytes()).unwrap();
        // // TODO(!): we need to figure out how to make global state accessible to the UI.
        // // TODO(!): we can use messaging to trigger updates, but still should be central repo?
        if self.wifi_ready {
            port.write_all("Y".as_bytes()).unwrap();
        } else {
            port.write_all("N".as_bytes()).unwrap();
        }
        port.write_all(" Cell ".as_bytes()).unwrap();
        if self.cell_ready {
            port.write_all("Y".as_bytes()).unwrap();
        } else {
            port.write_all("N".as_bytes()).unwrap();
        }
        // // Draw left
        port.write_all(&*set_cursor_bytes(1, 3)).unwrap();
        port.write_all("L ".as_bytes()).unwrap();

        port.write_all(&*set_cursor_bytes(1, 4)).unwrap();
        port.write_all("R ".as_bytes()).unwrap();

        port.write_all(&[254, 124, 3, 3, 0, self.left_volume])
            .unwrap();
        port.write_all(&[254, 124, 3, 4, 0, self.right_volume])
            .unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        // Do nothing!
        // Set the bar graph values

        // let left_scaled = ((1.0 + left_db / min_db) * 100.0).clamp(0.0, 100.0);
        // let right_scaled = ((1.0 + right_db / min_db) * 100.0).clamp(0.0, 100.0);
        port.write_all(&[254, 124, 3, 3, 0, self.left_volume])
            .unwrap();
        port.write_all(&[254, 124, 3, 4, 0, self.right_volume])
            .unwrap();
        // println!("Left volume {:?} Right Volume {:?}", self.left_volume, self.right_volume);
    }

    fn handle_button(&self, _: &UnsafeState, button: u8) -> Option<Box<dyn Screen>> {
        // All buttons should open the menu
        if button == TOP_LEFT_BUTTON {
            Some(Box::new(MenuScreen::new()))
        } else {
            None
        }
    }

    fn handle_message(
        &self,
        _: &UnsafeState,
        message: RidgelineMessage,
    ) -> Option<Box<dyn Screen>> {
        match message {
            RidgelineMessage::VolumeChange {
                left: left_db,
                right: right_db,
            } => {
                let left_scaled = scale_db_to_display(left_db);
                let right_scaled = scale_db_to_display(right_db);

                Some(Box::new(HomeScreen {
                    wifi_ready: self.wifi_ready,
                    cell_ready: self.cell_ready,
                    left_volume: left_scaled,
                    right_volume: right_scaled,
                }))
            }
            _ => None,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, PartialEq)]
struct MenuScreen {
    current: u8,
}

impl MenuScreen {
    fn new() -> Self {
        MenuScreen { current: 1 }
    }
}

impl Screen for MenuScreen {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn draw_screen(&self, port: &mut Box<dyn SerialPort>) {
        let options = [
            "Reboot",
            "Reset Settings",
            "Another Setting",
            "Another setting",
        ];

        let mut bytes_to_write: Vec<u8> = Vec::new();

        let mut row = 1;
        for option in options {
            if row == self.current {
                bytes_to_write.extend("* ".as_bytes());
            } else {
                bytes_to_write.extend("  ".as_bytes());
            }
            bytes_to_write.extend(option.as_bytes());
            row += 1;
            bytes_to_write.extend(set_cursor_bytes(1, row));
        }

        port.write_all(bytes_to_write.as_slice()).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        for row in 1..5 {
            port.write_all(set_cursor_bytes(1, row).as_slice()).unwrap();
            if row == self.current {
                port.write_all("*".as_bytes()).unwrap();
            } else {
                port.write_all(" ".as_bytes()).unwrap();
            }
        }
    }

    fn handle_button(&self, app_state: &UnsafeState, button: u8) -> Option<Box<dyn Screen>> {
        match button {
            UP_BUTTON => Some(Box::new(MenuScreen {
                current: max(self.current - 1, 1),
            })),
            DOWN_BUTTON => Some(Box::new(MenuScreen {
                current: min(self.current + 1, 4),
            })),
            CENTER_BUTTON => Some(Box::new(HomeScreen::new(app_state))),
            TOP_LEFT_BUTTON | BOTTOM_LEFT_BUTTON => Some(Box::new(HomeScreen::new(app_state))),
            _ => None,
        }
    }

    fn handle_message(&self, _: &UnsafeState, _: RidgelineMessage) -> Option<Box<dyn Screen>> {
        // Do nothing!
        None
    }
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

async fn handle_message(
    current_state: DisplayState,
    app_state: &SafeState,
    message: RidgelineMessage,
    write_port: &mut Box<dyn SerialPort>,
) {
    let mut screen = current_state.write().await;
    let app_state = app_state.read().await;
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
        println!("New screen has been acquired via button!!");
        // print_type_of(&new_screen.as_any());
        // print_type_of(&(*screen).as_any());
        if new_screen.as_any().type_id() != (*screen).as_any().type_id() {
            let _ = clear_screen(port);
            new_screen.draw_screen(port);
        } else {
            new_screen.redraw_screen(port);
        }
        *screen = new_screen;
    }
}

fn connect() -> Box<dyn SerialPort> {
    println!("Connecting to the serial port...");
    let port_name = "/dev/ttyUSB0"; // Replace with your port
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
    mut lcd_rx: Receiver<RidgelineMessage>,
    shared_state: SafeState,
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
    });

    // This task is responsible for reading from the USB serial and responding to
    // button presses as needed.
    let read_state = display_state.clone();
    // let tokio_handle = Handle::current();
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
                            handle_button(read_state.clone(), &shared_state, button_pressed, &mut read_port).await;
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
    set_brightness(&mut port, 0.5).expect("Could not set brightness"); // Set brightness to 50%
                                                                       // Initialize the horizontal bar graphs (page 22 of the LCD manual)
    port.write_all(&[254, 104]).unwrap();

    // Clear the screen and then draw the first screen *we* want to show
    let first_screen = display_state.read().await;
    println!("Writing the first screen...");
    clear_screen(&mut port).expect("Could not clear the screen");
    first_screen.draw_screen(&mut port);

    (lcd_reader, lcd_writer)
}

/******************************************
 Utility functions
******************************************/
const SET_DISPLAY_BRIGHTNESS: &[u8; 2] = &[254, 156];
const SET_BUTTON_BRIGHTNESS: &[u8; 2] = &[254, 153];
const CLEAR_SCREEN: &[u8; 2] = &[254, 88];
const SET_CURSOR: &[u8; 2] = &[254, 71];

fn set_cursor_bytes(column: u8, row: u8) -> Vec<u8> {
    let mut bytes_to_write: Vec<u8> = Vec::new();
    bytes_to_write.extend(SET_CURSOR);
    bytes_to_write.extend(&[column, row]);
    bytes_to_write
}

fn set_brightness(port: &mut Box<dyn SerialPort>, percentage: f32) -> io::Result<()> {
    let brightness = (255.0 * percentage).round() as u8;
    port.write_all(&[254, 156, brightness])?;
    port.write_all(&[254, 153, brightness])?;
    Ok(())
}

fn clear_screen(port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    port.write_all(CLEAR_SCREEN)?;
    Ok(())
}

fn reset_screen(port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    clear_screen(port).expect("Could not clear the screen");
    port.write_all("88.7 RIDGELINE V2".as_bytes())
        .expect("could not write to screen");
    Ok(())
}

// fn go_home(port: &mut dyn SerialPort) -> io::Result<()> {
//     port.write_all(&[254, 72])?;
//     Ok(())
// }
//
// fn place_graph(port: &mut dyn SerialPort, column: u8, row: u8, width: u8) -> io::Result<()> {
//     port.write_all(&[254, 124, column, row, 0, width])?;
//     Ok(())
// }
//
// fn all_lights_off(port: &mut dyn SerialPort) -> io::Result<()> {
//     for i in 1..=6 {
//         port.write_all(&[254, 86, i])?;
//     }
//     Ok(())
// }
//
// fn power_light_on(port: &mut dyn SerialPort) -> io::Result<()> {
//     port.write_all(&[254, 87, 1])?;
//     Ok(())
// }
//
// fn recording_light_off(port: &mut dyn SerialPort) -> io::Result<()> {
//     port.write_all(&[254, 86, 3])?;
//     Ok(())
// }
//
// fn recording_light_on(port: &mut dyn SerialPort) -> io::Result<()> {
//     port.write_all(&[254, 87, 3])?;
//     Ok(())
// }

// fn program_lights(port: &mut dyn SerialPort) -> io::Result<()> {
//     for i in 1..=6 {
//         let value = if i == 2 { 1 } else { 0 };
//         port.write_all(&[254, 195, i, value])?;
//     }
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_equality() {
        let screen_one = MenuScreen { current: 1 };
        let screen_two = MenuScreen { current: 1 };
        assert_eq!(screen_one == screen_two, true);

        let screen_three = MenuScreen { current: 3 };
        assert_eq!(screen_one == screen_three, false);
    }
}
