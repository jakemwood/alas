use crate::lcd_display::home_screen::HomeScreen;
use crate::lcd_display::matrix_orbital;
use crate::lcd_display::matrix_orbital::{
    BOTTOM_LEFT_BUTTON, CENTER_BUTTON, DOWN_BUTTON, TOP_LEFT_BUTTON, UP_BUTTON,
};
use crate::lcd_display::screen::Screen;
use alas_lib::state::UnsafeState;
use alas_lib::state::AlasMessage;
use serialport::SerialPort;
use std::any::Any;
use std::cmp::{max, min};
use std::io::Write;

impl Screen for MenuScreen {
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
            bytes_to_write.extend(matrix_orbital::set_cursor_bytes(1, row));
        }

        port.write_all(bytes_to_write.as_slice()).unwrap();
    }

    fn redraw_screen(&self, port: &mut Box<dyn SerialPort>) {
        for row in 1..5 {
            port.write_all(matrix_orbital::set_cursor_bytes(1, row).as_slice())
                .unwrap();
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

    fn handle_message(&self, _: &UnsafeState, _: AlasMessage) -> Option<Box<dyn Screen>> {
        // Do nothing!
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct MenuScreen {
    current: u8,
}

impl MenuScreen {
    pub fn new() -> Self {
        MenuScreen { current: 1 }
    }
}

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
