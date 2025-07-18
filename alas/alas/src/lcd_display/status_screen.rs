use crate::lcd_display::matrix_orbital::{clear_screen};
use crate::lcd_display::screen::Screen;
use alas_lib::state::AlasMessage;
use alas_lib::state::UnsafeState;
use serialport::SerialPort;
use std::any::Any;
use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct StatusScreen {
    message: String,
}

impl Screen for StatusScreen {
    fn draw_screen(&self, port: &mut dyn Write) {
        port.write_all(self.message.as_bytes()).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        clear_screen(port).unwrap();
        self.draw_screen(port);
    }

    fn handle_button(&self, _: &UnsafeState, _: u8) -> Option<Box<dyn Screen>> {
        None
    }

    fn handle_message(&self, _: &UnsafeState, _: AlasMessage) -> Option<Box<dyn Screen>> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl StatusScreen {
    pub fn new(message: String) -> Self {
        StatusScreen { message }
    }

    pub fn shutting_down() -> Self {
        StatusScreen::new("Shutting down...".to_string())
    }

    pub fn rebooting() -> Self {
        StatusScreen::new("Rebooting...".to_string())
    }
}