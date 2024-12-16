use std::cmp::{max, min};
use core::RidgelineMessage;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Read, Write};
use std::sync::{Arc};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use tokio::task;

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

trait ScreenImpl {
    // Draw the screen from scratch
    fn draw_screen(&self, port: &mut Box<dyn SerialPort>);

    // Update only the parts of the screen that have changed
    // TODO: do we need old state?
    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>);

    fn handle_button(&self, button: u8) -> Box<dyn ScreenImpl>;
}

impl PartialEq for dyn ScreenImpl {
    fn eq(&self, other: &Self) -> bool {
        // TODO: is this right?
        self.eq(other)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HomeScreen {
    left_volume: u8,
    right_volume: u8,
    wifi_status: bool,
    cell_status: bool
}

impl ScreenImpl for HomeScreen {
    fn draw_screen(&self, port: &mut Box<dyn SerialPort>) {
        let mut bytes_to_write: Vec<u8> = Vec::new();

        bytes_to_write.extend("88.7 RIDGELINE RADIO".as_bytes());

        bytes_to_write.extend(set_cursor_bytes(10, 2));
        if self.wifi_status {
            bytes_to_write.extend("WiFi On".as_bytes());
        }
        else {
            bytes_to_write.extend("WiFi Off".as_bytes());
        }

        let _ = port.write_all(bytes_to_write.as_slice());
    }

    fn redraw_screen(&self, _port: &mut Box<dyn SerialPort>) {
        // do nothing just yet
    }

    fn handle_button(&self, _button: u8) -> Box<dyn ScreenImpl> {
        // Any button should start the menu
        Box::new(MenuScreen {
            current_option: 1
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MenuScreen {
    current_option: u8
}

impl ScreenImpl for MenuScreen {
    fn draw_screen(&self, port: &mut Box<dyn SerialPort>) {
        let options = ["Reboot", "Reset Settings", "Antoher Seting", "Another seting"];

        let mut bytes_to_write: Vec<u8> = Vec::new();
        bytes_to_write.extend("* ".as_bytes());

        let mut row = 1;
        for option in options {
            bytes_to_write.extend(option.as_bytes());
            row += 1;
            bytes_to_write.extend(set_cursor_bytes(3, row));
        }

        port.write_all(bytes_to_write.as_slice()).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        for row in 1..5 {
            port.write_all(set_cursor_bytes(1, row).as_slice()).unwrap();
            if row == self.current_option {
                port.write_all("*".as_bytes()).unwrap();
            }
            else {
                port.write_all(" ".as_bytes()).unwrap();
            }
        }
    }

    fn handle_button(&self, button: u8) -> Box<dyn ScreenImpl> {
        match button {
            UP_BUTTON => {
                Box::new(MenuScreen {
                    current_option: max(self.current_option - 1, 0),
                })
            },
            DOWN_BUTTON => {
                Box::new(MenuScreen {
                    current_option: min(self.current_option + 1, 4),
                })
            },
            CENTER_BUTTON => {
                todo!()
            },
            TOP_LEFT_BUTTON | BOTTOM_LEFT_BUTTON => {
                todo!()
            },
            _ => {
                Box::new(self.clone())
            },
        }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Screen {
    Home(HomeScreen),
    Menu(MenuScreen),
}

#[derive(Debug, Copy, Clone)]
struct LCDState {
    current_screen: Screen,
}

fn handle_message(message: RidgelineMessage, write_port: &mut Box<dyn SerialPort>) {
    let mut bytes_to_write: Vec<u8> = Vec::new();
    match message {
        RidgelineMessage::Ticker { count } => {
            bytes_to_write.extend(&[254, 71, 1, 1]);
            bytes_to_write.extend(count.to_string().as_bytes());
            println!("Writing out to display!!");
        }
        RidgelineMessage::NetworkStatusChange { new_state } => {
            if new_state == 100 {
                bytes_to_write.extend(&[254, 71, 10, 2]);
                bytes_to_write.extend("WiFi On".as_bytes());
            }
        }
    }
    let _ = write_port.write_all(bytes_to_write.as_slice());
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
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Could not open serial port")
}

pub async fn start(mut lcd_rx: Receiver<RidgelineMessage>) {
    let mut state = Arc::new(RwLock::new(LCDState {
        current_screen: Screen::Home(HomeScreen {
            left_volume: 0,
            right_volume: 0,
            wifi_status: false,
            cell_status: false,
        }),
    }));

    let mut port = connect();
    // port is safe to clone, but ideally have a read/write clone
    // based on the duplex example

    // This task is responsible for listening to state changes sent to us from
    // the event bus and updating the screens, as appropriate.
    let mut write_port = port.try_clone().expect("Could not create write port");
    let lcd_writer = task::spawn(async move {
        // Setup our display
        reset_screen(&mut write_port).expect("Could not reset screen");
        set_brightness(&mut write_port, 0.5).expect("Could not set brightness"); // Set brightness to 50%

        // Now listen for any events that we need in order to process writes to our screen
        loop {
            let message = lcd_rx.recv().await;
            match message {
                Ok(message) => {
                    handle_message(message, &mut write_port);
                }
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            }
        }

        println!("End of loop reached!");
    });

    // This task is responsible for reading from the USB serial and responding to
    // button presses as needed.
    let read_state = state.clone();
    // let tokio_handle = Handle::current();
    let lcd_reader = task::spawn(async move {
        loop {
            let data_to_process = {
                let mut buf = [0; 2];
                let bytes_read = port.read(buf.as_mut_slice());
                match bytes_read {
                    Ok(bytes_read) => buf[..bytes_read].to_vec(),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                    Err(e) => panic!("{:?}", e),
                }
            };

            handle_button(data_to_process, read_state.clone(), &mut port).await;
        }
    });

    lcd_writer.await.unwrap();
    lcd_reader.await.unwrap();
}

async fn handle_button(
    data_to_process: Vec<u8>,
    rw_state: Arc<RwLock<LCDState>>,
    port: &mut Box<dyn SerialPort>
) {
    let mut state = *(rw_state.write().await);
    let button_pressed = data_to_process[0];

    let new_state = state.current_screen.handle_button(button_pressed);

    // Reload the read state
    if new_state != state {
        println!("New screen is: {:?}", new_state);
        let _ = clear_screen(port);
        new_state.draw_screen(port);
    }
    else {
        new_state.redraw_screen(port);
    }
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
