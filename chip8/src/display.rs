use std::cell::RefCell;
use std::rc::Rc;
use crate::emulator::{Emulator, EmulatorComponent};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const FONT_SET_SIZE: usize = 80;
pub const FONT_SET: [u8; FONT_SET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Display {
    emulator: Rc<RefCell<Emulator>>,
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Display {
    pub fn new(emulator: Rc<RefCell<Emulator>>) -> Self {
        let display = Self {
            emulator,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        };
        display
    }

    /// Clear screen buffer
    pub fn op_cls(&mut self) {
        self.reset();
    }

    /// Draws sprite at X Y location
    pub fn op_drw(&mut self, x_coord: usize, y_coord: usize, num_rows: usize) {
        // Keep track if any pixels were flipped
        let mut flipped = false;
        // Iterate over each row of our sprite
        for y_line in 0..num_rows {
            // Determine which memory address our row's data is stored
            let addr = self.emulator.get_mut().get_cpu().get_i_register() + y_line as u16;
            let pixels = self.emulator.get_mut().get_memory().fetch_byte(addr);
            // Iterate over each column in our row
            for x_line in 0..8 {
                // Use a mask to fetch current pixel's bit. Only flip if a 1
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    // Sprites should wrap around screen, so apply modulo
                    let x = (x_coord + x_line) % SCREEN_WIDTH;
                    let y = (y_coord + y_line) % SCREEN_HEIGHT;
                    // Get our pixel's index for our 1D screen array
                    let idx = x + SCREEN_WIDTH * y;
                    // Check if we're about to flip the pixel and set
                    flipped |= self.screen[idx];
                    self.screen[idx] = self.screen[idx] ^ true;
                }
            }
        }
        // Populate VF register
        if flipped {
            self.emulator.get_mut().get_cpu().set_register_value(0xF, 1);
        } else {
            self.emulator.get_mut().get_cpu().set_register_value(0xF, 0);
        }
    }
}

impl EmulatorComponent for Display {
    fn reset(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }
}