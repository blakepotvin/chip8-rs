use std::cell::RefCell;
use std::rc::Rc;
use crate::emulator::{Emulator, EmulatorComponent};

const NUMBER_OF_KEYS: usize = 16;

pub struct Input {
    emulator: Rc<RefCell<Emulator>>,
    keys: [bool; NUMBER_OF_KEYS],
}

impl Input {
    pub fn new(emulator: Rc<RefCell<Emulator>>) -> Self {
        let input = Self {
            emulator,
            keys: [false; NUMBER_OF_KEYS],
        };
        input
    }

    /// SKP Vx - Skip next instructor if key at index Vx is pressed.
    pub fn op_skp(&mut self, x: usize, reverse: bool) {
        let vx = self.emulator.get_mut().get_cpu().get_register_value(x) as usize;
        let key = self.keys[vx];
        let pc = self.emulator.get_mut().get_cpu().get_program_counter();
        if (!reverse && key) || (reverse && !key) {
            self.emulator.get_mut().get_cpu().set_program_counter(pc + 2);
        }
    }

    /// LD Vx - Loads register Vx with value of key pressed.
    pub fn op_ld_wait(&mut self, x: usize) {
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.emulator.get_mut().get_cpu().set_register_value(x, i as u8);
                pressed = true;
                break;
            }
        }

        if !pressed {
            // redo opcode
            let pc = self.emulator.get_mut().get_cpu().get_program_counter();
            self.emulator.get_mut().get_cpu().set_program_counter(pc - 2);
        }
    }
}

impl EmulatorComponent for Input {
    fn reset(&mut self) {
        self.keys = [false; NUMBER_OF_KEYS];
    }
}