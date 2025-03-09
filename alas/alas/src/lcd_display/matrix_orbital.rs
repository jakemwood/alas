use serialport::SerialPort;
use std::io;
use std::io::Write;

pub const SCREEN_WIDTH: u8 = 20;
pub const SCREEN_HEIGHT: u8 = 4;
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
pub const TOP_LEFT_BUTTON: u8 = 65;
pub const UP_BUTTON: u8 = 66;
const LEFT_BUTTON: u8 = 68;
pub const CENTER_BUTTON: u8 = 69;
const RIGHT_BUTTON: u8 = 67;
pub const DOWN_BUTTON: u8 = 72;
pub const BOTTOM_LEFT_BUTTON: u8 = 71;
const SET_DISPLAY_BRIGHTNESS: &[u8; 2] = &[254, 156];
const SET_BUTTON_BRIGHTNESS: &[u8; 2] = &[254, 153];
const CLEAR_SCREEN: &[u8; 2] = &[254, 88];
const SET_CURSOR: &[u8; 2] = &[254, 71];

pub fn set_cursor_bytes(column: u8, row: u8) -> Vec<u8> {
    let mut bytes_to_write: Vec<u8> = Vec::new();
    bytes_to_write.extend(SET_CURSOR);
    bytes_to_write.extend(&[column, row]);
    bytes_to_write
}

pub fn set_brightness(port: &mut Box<dyn SerialPort>, percentage: f32) -> io::Result<()> {
    let brightness = (255.0 * percentage).round() as u8;
    port.write_all(&[254, 156, brightness])?;
    port.write_all(&[254, 153, brightness])?;
    Ok(())
}

pub fn clear_screen(port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    port.write_all(CLEAR_SCREEN)?;
    Ok(())
}

fn reset_screen(port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    clear_screen(port).expect("Could not clear the screen");
    port.write_all("88.7 RIDGELINE V2".as_bytes()).expect("could not write to screen");
    Ok(())
}

/******************************************
 Utility functions
******************************************/

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
