use crate::lcd_display::matrix_orbital;
use crate::lcd_display::matrix_orbital::{CENTER_BUTTON, TOP_LEFT_BUTTON};
use crate::lcd_display::menu_screen::MenuScreen;
use crate::lcd_display::screen::Screen;
use alas_lib::state::AlasMessage;
use alas_lib::state::UnsafeState;
use serialport::SerialPort;
use std::any::Any;
use std::io::Write;
use alas_lib::wifi::AlasWiFiState;
use crate::lcd_display::upload_progress::UploadScreen;

impl Screen for HomeScreen {
    fn draw_screen(&self, port: &mut dyn Write) {
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
        port.write_all(&*matrix_orbital::set_cursor_bytes(1, 3)).unwrap();
        port.write_all("L ".as_bytes()).unwrap();

        port.write_all(&*matrix_orbital::set_cursor_bytes(1, 4)).unwrap();
        port.write_all("R ".as_bytes()).unwrap();

        port.write_all(&[254, 124, 3, 3, 0, self.left_volume]).unwrap();
        port.write_all(&[254, 124, 3, 4, 0, self.right_volume]).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        // Do nothing!
        // Set the bar graph values
        port.write_all(&[254, 124, 3, 3, 0, self.left_volume]).unwrap();
        port.write_all(&[254, 124, 3, 4, 0, self.right_volume]).unwrap();

        // Draw Wi-Fi and cellular yes/no
        port.write_all(&*matrix_orbital::set_cursor_bytes(8, 2)).unwrap();
        if self.wifi_ready {
            port.write_all(b"Y").unwrap();
        } else {
            port.write_all(b"N").unwrap();
        }

        // Wi-Fi? Y Cell N
        port.write_all(&*matrix_orbital::set_cursor_bytes(15, 2)).unwrap();
        if self.cell_ready {
            port.write_all(b"Y").unwrap();
        } else {
            port.write_all(b"N").unwrap();
        }
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

    fn handle_message(&self, _: &UnsafeState, message: AlasMessage) -> Option<Box<dyn Screen>> {
        match message {
            AlasMessage::VolumeChange { left: left_db, right: right_db } => {
                let left_scaled = scale_db_to_display(left_db);
                let right_scaled = scale_db_to_display(right_db);

                Some(
                    Box::new(HomeScreen {
                        wifi_ready: self.wifi_ready,
                        cell_ready: self.cell_ready,
                        left_volume: left_scaled,
                        right_volume: right_scaled,
                    })
                )
            }
            AlasMessage::NetworkStatusChange { new_state } => {
                if new_state == AlasWiFiState::Connected {
                    Some(
                        Box::new(HomeScreen {
                            wifi_ready: true,
                            ..*self
                        })
                    )
                } else {
                    Some(
                        Box::new(HomeScreen {
                            wifi_ready: false,
                            ..*self
                        })
                    )
                }
            }
            AlasMessage::UploadStateChange { new_state } => {
                Some(
                    Box::new(UploadScreen { progress: new_state.progress })
                )
            }
            AlasMessage::CellularStatusChange { new_state, .. } => {
                Some(
                    Box::new(HomeScreen {
                        cell_ready: new_state == AlasWiFiState::Connected,
                        ..*self
                    })
                )
            }
            _ => None,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct HomeScreen {
    wifi_ready: bool,
    cell_ready: bool,
    left_volume: u8,
    right_volume: u8,
}

impl HomeScreen {
    pub fn new(app_state: &UnsafeState) -> Self {
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
