use crate::lcd_display::matrix_orbital::{ BOTTOM_LEFT_BUTTON, TOP_LEFT_BUTTON };
use crate::lcd_display::menu_screen::MenuScreen;
use crate::lcd_display::screen::Screen;
use alas_lib::state::AlasMessage;
use alas_lib::state::UnsafeState;
use get_if_addrs::get_if_addrs;
use serialport::SerialPort;
use std::any::Any;
use std::io::Write;
use std::net::IpAddr;

impl Screen for IPScreen {
    fn draw_screen(&self, port: &mut dyn Write) {
        match get_if_addrs() {
            Ok(interfaces) => {
                for iface in interfaces {
                    match iface.ip() {
                        IpAddr::V4(ip) => {
                            port.write_all(ip.to_string().as_bytes()).unwrap();
                            port.write_all("\r\n".as_bytes()).unwrap();
                            println!("Interface: {}, IP: {}", iface.name, ip);
                        }
                        IpAddr::V6(ip) => {
                            println!("IPV6 interface: {}, {}", iface.name, ip);
                        }
                    }
                }
            }
            Err(e) => {
                port.write_all("Could not get IPs\r\n".as_bytes()).unwrap();
                port.write_all(e.to_string().as_bytes()).unwrap();
            }
        }
    }

    fn redraw_screen(&self, _port: &mut Box<dyn SerialPort>) {}

    fn handle_button(&self, _: &UnsafeState, button: u8) -> Option<Box<dyn Screen>> {
        match button {
            TOP_LEFT_BUTTON | BOTTOM_LEFT_BUTTON => Some(Box::new(MenuScreen::new())),
            _ => None,
        }
    }

    fn handle_message(&self, _: &UnsafeState, _: AlasMessage) -> Option<Box<dyn Screen>> {
        // Do nothing!
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct IPScreen {}

impl IPScreen {
    pub fn new() -> Self {
        IPScreen {}
    }
}
