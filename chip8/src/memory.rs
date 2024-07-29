use std::cell::RefCell;
use std::rc::Rc;
use crate::display::{FONT_SET, FONT_SET_SIZE};
use crate::emulator::{Emulator, EmulatorComponent};

const RAM_SIZE: usize = 0x1000; // 4096 bytes

pub struct Memory {
    emulator: Rc<RefCell<Emulator>>,
    ram: [u8; RAM_SIZE],
    delay_timer: u8,
    sound_timer: u8,
}

impl Memory {
    pub fn new(emulator: Rc<RefCell<Emulator>>) -> Self {
        let mut memory = Self {
            emulator,
            ram: [0; RAM_SIZE],
            delay_timer: 0,
            sound_timer: 0,
        };
        memory.initialize_font_set();
        memory
    }

    /// Initializes the font set within the first 80 bytes of memory.
    pub fn initialize_font_set(&mut self) {
        self.ram[..FONT_SET_SIZE].copy_from_slice(&FONT_SET);
    }

    /// Fetch byte
    pub fn fetch_byte(&self, index: u16) -> u8 {
        self.ram[index as usize]
    }

    /// Fetches word at index
    pub fn fetch_word(&self, index: u16) -> u16 {
        (self.ram[index as usize] as u16) << 8 | (self.ram[(index + 1) as usize] as u16)
    }

    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    pub fn op_ld_dt(&mut self, x: usize) {
        self.delay_timer = self.emulator.get_mut().get_cpu().get_register_value(x);
    }

    pub fn op_ld_st(&mut self, x: usize) {
        self.sound_timer = self.emulator.get_mut().get_cpu().get_register_value(x);
    }

    pub fn op_ld_bcd(&mut self, x: usize) {
        let vx = self.emulator.get_mut().get_cpu().get_register_value(x) as f32;

        // Fetch the hundreds digit by dividing by 100 and tossing the decimal
        let hundreds = (vx / 100.0).floor() as u8;
        // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        // Fetch the ones digit by tossing the hundreds and the tens
        let ones = (vx % 10.0) as u8;

        let i_reg = self.emulator.get_mut().get_cpu().get_i_register();
        self.ram[i_reg as usize] = hundreds;
        self.ram[(i_reg + 1) as usize] = tens;
        self.ram[(i_reg + 2) as usize] = ones;
    }

    pub fn op_str(&mut self, x: usize) {
        let i = self.emulator.get_mut().get_cpu().get_i_register() as usize;
        for idx in 0..=x {
            self.ram[i + idx] = self.emulator.get_mut().get_cpu().get_register_value(idx);
        }
    }

    pub fn op_ld(&mut self, x: usize) {
        let i = self.emulator.get_mut().get_cpu().get_i_register() as usize;
        for idx in 0..=x {
            self.emulator.get_mut().get_cpu().set_register_value(idx, self.ram[i + idx]);
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                unimplemented!("Beeping not implemented yet.")
            }
            self.sound_timer -= 1;
        }
    }
}

impl EmulatorComponent for Memory {
    fn reset(&mut self) {
        self.ram = [0; RAM_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.initialize_font_set();
    }
}

