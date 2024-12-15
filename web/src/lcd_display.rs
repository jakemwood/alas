use core::wifi::WiFiObserver;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use tokio::task::{JoinError, JoinHandle, JoinSet};
use tokio::{join, task, try_join};

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
const BOTTOM_BUTTON: u8 = 72;
const BOTTOM_LEFT_BUTTON: u8 = 71;

pub struct LCDServer {
    wifi_observer: Arc<WiFiObserver>,
    serial_port: Option<Box<dyn SerialPort>>,
}

impl LCDServer {
    pub fn new(wifi_observer: Arc<WiFiObserver>) -> Self {
        Self {
            wifi_observer,
            serial_port: None,
        }
    }

    pub fn connect(&mut self) {
        println!("Connecting to the serial port...");
        let port_name = "/dev/ttyUSB0"; // Replace with your port
        let baud_rate = 19200;

        self.serial_port = Some(
            serialport::new(port_name, baud_rate)
                .data_bits(DataBits::Eight)
                .parity(Parity::None)
                .stop_bits(StopBits::One)
                .flow_control(FlowControl::None)
                // .timeout(Duration::from_millis(100))
                .open()
                .expect("Could not open serial port"),
        );
    }

    fn get_port(&self) -> Box<dyn SerialPort> {
        self.serial_port.as_ref().unwrap().try_clone().unwrap()
    }

    pub fn start_writing(&self) -> JoinHandle<()> {
        let mut write_serial = self.get_port();

        let mut rx = self.wifi_observer.sender.subscribe();

        task::spawn(async move {
            loop {
                let msg = *rx.borrow_and_update();
                println!("Received a message! {:?}", msg);
                if let Some(i) = msg {
                    write_serial
                        .write_all(i.to_string().as_bytes())
                        .expect("could not write to port");
                }
                if rx.changed().await.is_err() {
                    break;
                }
            }
        })
    }

    fn handle_button(data_to_process: Vec<u8>) {
        match data_to_process[0] {
            TOP_LEFT_BUTTON => {
                println!("Top left!");
            }
            UP_BUTTON => {
                println!("Up");
            }
            LEFT_BUTTON => {
                println!("Left");
            }
            CENTER_BUTTON => {
                println!("Center");
            }
            RIGHT_BUTTON => {
                println!("Right");
            }
            BOTTOM_LEFT_BUTTON => {
                println!("Bottom left");
            }
            BOTTOM_BUTTON => {
                println!("Down");
            }
            _ => {}
        }
    }

    pub fn start_reading(&self) -> JoinHandle<()> {
        let mut read_serial = self.get_port();

        task::spawn_blocking(move || {
            loop {
                let data_to_process = {
                    let mut buf = [0; 2];
                    let bytes_read = read_serial.read(buf.as_mut_slice());
                    match bytes_read {
                        Ok(bytes_read) => buf[..bytes_read].to_vec(),
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                        Err(e) => panic!("{:?}", e),
                    }
                };

                LCDServer::handle_button(data_to_process);
            }
        })
    }

    pub fn start(&mut self) -> (JoinHandle<()>, JoinHandle<()>) {
        println!("Starting LCD server...");

        let mut serial = self.get_port();

        self.reset_screen(&mut serial).expect("Could not reset screen");
        self.set_brightness(&mut serial, 0.5).expect("Could not set brightness"); // Set brightness to 50%

        let write = self.start_writing();
        let read = self.start_reading();
        (read, write)
    }

    fn set_brightness(&mut self, port: &mut Box<dyn SerialPort>, percentage: f32) -> io::Result<()> {
        let brightness = (255.0 * percentage).round() as u8;
        port.write_all(&[254, 156, brightness])?;
        port.write_all(&[254, 153, brightness])?;
        Ok(())
    }

    fn clear_screen(&mut self, port: &mut Box<dyn SerialPort>) -> io::Result<()> {
        port.write_all(&[254, 88])?;
        Ok(())
    }

    fn reset_screen(&mut self, port: &mut Box<dyn SerialPort>) -> io::Result<()> {
        self.clear_screen(port).expect("Could not clear the screen");
        port.write_all("88.7 RIDGELINE V1".as_bytes())
            .expect("could not write to screen");
        Ok(())
    }
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
