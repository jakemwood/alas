use alas_lib::state::AlasMessage;
use alas_lib::state::UnsafeState;
use serialport::SerialPort;
use std::any::Any;
use std::io::Write;

pub trait Screen: Send + Sync + Any {
    // Draw the screen from scratch
    fn draw_screen(&self, port: &mut dyn Write);

    // Update only the parts of the screen that have changed
    // TODO: do we need old state?
    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>);

    // Handle a button being pressed
    fn handle_button(&self, app_state: &UnsafeState, button: u8) -> Option<Box<dyn Screen>>;

    // Handle an incoming message from the bus
    fn handle_message(
        &self,
        app_state: &UnsafeState,
        message: AlasMessage
    ) -> Option<Box<dyn Screen>>;

    fn as_any(&self) -> &dyn Any;
}
