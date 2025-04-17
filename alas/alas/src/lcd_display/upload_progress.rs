use std::any::Any;
use std::io::Write;
use serialport::SerialPort;
use alas_lib::state::{AlasMessage, AlasUploadState, AlasUploadStatus, UnsafeState};
use crate::lcd_display::home_screen::HomeScreen;
use crate::lcd_display::matrix_orbital::{set_cursor_bytes, DOWN_BUTTON, UP_BUTTON};
use crate::lcd_display::screen::Screen;

#[derive(Clone)]
pub struct UploadScreen {
    pub progress: u8
}

impl Screen for UploadScreen {
    fn draw_screen(&self, port: &mut dyn Write) {
        port.write_all("Uploading".as_bytes()).unwrap();
        port.write_all(&*set_cursor_bytes(1, 2)).unwrap();
        port.write_all("recording...".as_bytes()).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        port.write_all(&[254, 124, 1, 3, 0, self.progress]).unwrap();
    }

    fn handle_button(&self, app_state: &UnsafeState, button: u8) -> Option<Box<dyn Screen>> {
        if button == UP_BUTTON {
            Some(Box::new(UploadScreen { progress: self.progress + 10 }))
        } else if button == DOWN_BUTTON {
            Some(Box::new(UploadScreen { progress: self.progress - 10 }))
        } else {
            None
        }
    }

    fn handle_message(&self, app_state: &UnsafeState, message: AlasMessage) -> Option<Box<dyn Screen>> {
        match message {
            AlasMessage::UploadStateChange { new_state } => {
                if new_state.state == AlasUploadStatus::Idle {
                    Some(Box::new(HomeScreen::new(&app_state)))
                }
                else {
                    Some(Box::new(UploadScreen { progress: new_state.progress }))
                }
            }
            AlasMessage::RecordingStarted => {
                Some(Box::new(HomeScreen::new(&app_state)))
            }
            _ => {
                None
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}