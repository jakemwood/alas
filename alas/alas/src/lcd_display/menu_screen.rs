use crate::lcd_display::home_screen::HomeScreen;
use crate::lcd_display::ip_screen::IPScreen;
use crate::lcd_display::matrix_orbital::{
    clear_screen,
    BOTTOM_LEFT_BUTTON,
    CENTER_BUTTON,
    DOWN_BUTTON,
    SCREEN_HEIGHT,
    TOP_LEFT_BUTTON,
    UP_BUTTON,
};
use crate::lcd_display::screen::Screen;
use alas_lib::state::AlasMessage;
use alas_lib::state::UnsafeState;
use alas_lib::wifi::create_config_hotspot;
use serialport::SerialPort;
use std::any::Any;
use std::cmp::{ max, min };
use std::io::Write;
use tokio::runtime::Handle;
use tokio::task;

#[derive(Clone, PartialEq)]
pub struct MenuScreen {
    current: u8,
    start_idx: u8,
}

const MENU_OPTIONS: [&str; 7] = [
    "IP Addresses",
    "Reconfigure WiFi",
    "Reboot",
    "Shut Down",
    "Reserved",
    "Reserved",
    "Reserved",
];

impl Screen for MenuScreen {
    fn draw_screen(&self, port: &mut dyn Write) {
        let mut bytes_to_write: Vec<u8> = Vec::new();

        // Starting at start_idx, write them out!
        let destination = min(MENU_OPTIONS.len(), (self.start_idx + SCREEN_HEIGHT) as usize);
        for i in self.start_idx as usize..destination {
            if self.current == (i as u8) {
                bytes_to_write.extend(b"* ");
            } else {
                bytes_to_write.extend(b"  ");
            }
            bytes_to_write.extend(MENU_OPTIONS[i].as_bytes());

            // Only add the carriage return if it's not the last option
            if i + 1 < destination {
                bytes_to_write.extend(b"\r\n");
            }
        }

        port.write_all(bytes_to_write.as_slice()).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        clear_screen(port).unwrap();
        self.draw_screen(port);
    }

    fn handle_button(&self, app_state: &UnsafeState, button: u8) -> Option<Box<dyn Screen>> {
        match button {
            UP_BUTTON => {
                let next_value = {
                    if self.current > 0 { self.current - 1 } else { 0 }
                };
                let mut start_idx = self.start_idx;

                if next_value < self.start_idx {
                    start_idx = next_value;
                }

                Some(
                    Box::new(MenuScreen {
                        current: next_value,
                        start_idx,
                    })
                )
            }
            DOWN_BUTTON => {
                let next_value = min(self.current + 1, (MENU_OPTIONS.len() as u8) - 1);
                let mut start_idx = self.start_idx;

                // If the next_value has scrolled out of view, adjust.
                if start_idx + SCREEN_HEIGHT <= next_value {
                    start_idx += 1;
                }

                Some(
                    Box::new(MenuScreen {
                        current: next_value,
                        start_idx,
                    })
                )
            }
            CENTER_BUTTON => {
                match self.current {
                    0 => Some(Box::new(IPScreen::new())),
                    1 => {
                        // This is ridiculous, but technically we are *currently* inside of an
                        // async runtime, BUT we cannot use the async runtime because we also want
                        // dynamic dispatch to work with our multiple screens. Dynamic dispatch
                        // is incompatible with async functions on trait declarations. Therefore,
                        // the following ridiculous workaround exists.
                        task::spawn_blocking(move || {
                            let handle = Handle::current();
                            handle.block_on(async {
                                create_config_hotspot().await;
                            });
                        });
                        Some(Box::new(HomeScreen::new(app_state)))
                    }
                    _ => Some(Box::new(HomeScreen::new(app_state))),
                }
            }
            TOP_LEFT_BUTTON | BOTTOM_LEFT_BUTTON => Some(Box::new(HomeScreen::new(app_state))),
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

impl MenuScreen {
    pub fn new() -> Self {
        MenuScreen {
            current: 0,
            start_idx: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alas_lib::state::AlasState;
    use std::io::Cursor;

    #[test]
    fn test_screen_equality() {
        let screen_one = MenuScreen {
            current: 0,
            start_idx: 0,
        };
        let screen_two = MenuScreen {
            current: 0,
            start_idx: 0,
        };
        assert_eq!(screen_one == screen_two, true);

        let screen_three = MenuScreen {
            current: 3,
            start_idx: 0,
        };
        assert_eq!(screen_one == screen_three, false);
    }

    #[test]
    fn test_handle_button() {
        let app_state = AlasState::test();
        let mut screen_one = MenuScreen {
            current: 0,
            start_idx: 0,
        };

        let screen_two = screen_one.handle_button(&app_state, DOWN_BUTTON).unwrap();
        let menu_screen_two = screen_two.as_any().downcast_ref::<MenuScreen>().unwrap();
        assert_eq!(menu_screen_two.start_idx, 0);
        assert_eq!(menu_screen_two.current, 1);

        let screen_two = screen_two.handle_button(&app_state, DOWN_BUTTON).unwrap();
        let menu_screen_two = screen_two.as_any().downcast_ref::<MenuScreen>().unwrap();
        assert_eq!(menu_screen_two.start_idx, 0);
        assert_eq!(menu_screen_two.current, 2);

        let screen_two = screen_two.handle_button(&app_state, DOWN_BUTTON).unwrap();
        let menu_screen_two = screen_two.as_any().downcast_ref::<MenuScreen>().unwrap();
        assert_eq!(menu_screen_two.start_idx, 0);
        assert_eq!(menu_screen_two.current, 3);

        let screen_two = screen_two.handle_button(&app_state, DOWN_BUTTON).unwrap();
        let menu_screen_two = screen_two.as_any().downcast_ref::<MenuScreen>().unwrap();
        assert_eq!(menu_screen_two.start_idx, 1);
        assert_eq!(menu_screen_two.current, 4);

        let screen_two = screen_two.handle_button(&app_state, DOWN_BUTTON).unwrap();
        let menu_screen_two = screen_two.as_any().downcast_ref::<MenuScreen>().unwrap();
        assert_eq!(menu_screen_two.start_idx, 2);
        assert_eq!(menu_screen_two.current, 5);
    }

    #[test]
    fn test_draw() {
        let mut screen_one = MenuScreen {
            current: 0,
            start_idx: 0,
        };

        // Use an in-memory buffer to simulate the Write trait
        let mut output_buffer = Cursor::new(Vec::new());

        // Call the draw_screen function
        screen_one.draw_screen(&mut output_buffer);

        // Verify the output written to the buffer
        let written_data = output_buffer.into_inner();
        assert_eq!(
            written_data,
            b"* IP Addresses\r\n  Reconfigure WiFi\r\n  Reboot\r\n  Shut Down"
        );
    }

    #[test]
    fn test_draw_after_up() {
        let mut screen_one = MenuScreen {
            current: 0,
            start_idx: 0,
        };

        // Use an in-memory buffer to simulate the Write trait
        let mut output_buffer = Cursor::new(Vec::new());
        let screen_two = screen_one.handle_button(&AlasState::test(), UP_BUTTON).unwrap();

        screen_two.draw_screen(&mut output_buffer);

        // Verify the output written to the buffer
        let written_data = output_buffer.into_inner();
        assert_eq!(
            written_data,
            b"* IP Addresses\r\n  Reconfigure WiFi\r\n  Reboot\r\n  Shut Down"
        );
    }
}
