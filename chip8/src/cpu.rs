use std::cell::RefCell;
use std::rc::Rc;
use rand::random;
use crate::emulator::{Emulator, EmulatorComponent};

const NUMBER_OF_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const START_ADDRESS: u16 = 0x200;

pub struct CPU {
    emulator: Rc<RefCell<Emulator>>,
    v_registers: [u8; NUMBER_OF_REGISTERS],
    i_register: u16,
    stack: [u16; STACK_SIZE],
    program_counter: u16,
    stack_pointer: u16,
}

impl CPU {
    pub fn new(emulator: Rc<RefCell<Emulator>>) -> Self {
        let cpu = Self {
            emulator,
            v_registers: [0; NUMBER_OF_REGISTERS],
            i_register: 0,
            stack: [0; STACK_SIZE],
            program_counter: START_ADDRESS,
            stack_pointer: 0,
        };
        cpu
    }

    pub fn get_program_counter(&self) -> u16 {
        self.program_counter
    }

    pub fn set_program_counter(&mut self, value: u16) {
        self.program_counter = value;
    }

    pub fn get_i_register(&self) -> u16 {
        self.i_register
    }

    pub fn set_register_value(&mut self, index: usize, value: u8) {
        self.v_registers[index] = value;
    }

    pub fn get_register_value(&self, index: usize) -> u8 {
        self.v_registers[index]
    }

    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    /// RET - Return from subroutine
    pub fn op_ret(&mut self) {
        let return_address = self.pop();
        self.program_counter = return_address;
    }

    /// CALL - Call subroutine
    pub fn op_call(&mut self, operation: u16) {
        let nnn = operation & 0xFFF;
        self.push(self.program_counter);
        self.program_counter = nnn;
    }

    /// JMP NNN - Move program counter to given address
    pub fn op_jmp(&mut self, operation: u16) {
        let nnn = operation & 0xFFF;
        self.program_counter = nnn;
    }

    /// JMP V0 + NNN - Move program counter to given address
    pub fn op_reg_jmp(&mut self, operation: u16) {
        let nnn = operation & 0xFFF;
        self.program_counter = (self.v_registers[0] as u16) + nnn;
    }

    /// SKIP VX == NN - Skip next instruction if register VX == NN
    pub fn op_se(&mut self, operation: u16, x: usize) {
        let nn = (operation & 0xFF) as u8;
        if self.v_registers[x] == nn {
            self.program_counter += 2;
        }
    }

    /// SKIP VX == VY - Skip next instruction if register VX == NN
    pub fn op_reg_se(&mut self, x: usize, y: usize) {
        if self.v_registers[x] == self.v_registers[y] {
            self.program_counter += 2;
        }
    }

    /// SKIP VX != NN - Skip next instruction if register VX != NN
    pub fn op_sne(&mut self, operation: u16, x: usize) {
        let nn = (operation & 0xFF) as u8;
        if self.v_registers[x] == nn {
            self.program_counter += 2;
        }
    }

    /// LD VX = NN - Load register VX with value NN
    pub fn op_ld(&mut self, operation: u16, x: usize) {
        let nn = (operation & 0xFF) as u8;
        self.v_registers[x] = nn;
    }

    /// LD VX = VY - Load Register VX with value of register VY
    pub fn op_reg_ld(&mut self, x: usize, y: usize) {
        self.v_registers[x] = self.v_registers[y];
    }

    /// ADD VX += NN - Add NN to VX
    pub fn op_add(&mut self, operation: u16, x: usize) {
        let nn = (operation & 0xFF) as u8;
        self.v_registers[x] += nn;
    }

    /// ADD VX += VY - Add VY to VX
    pub fn op_reg_add(&mut self, x: usize, y: usize) {
        let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);
        let new_vf = if carry { 1 } else { 0 };
        self.v_registers[x] = new_vx;
        self.v_registers[0xF] = new_vf;
    }

    /// SUB VX -= VY - Subtract VY to VX
    pub fn op_reg_sub(&mut self, x: usize, y: usize, reverse: bool) {
        let new_vx: u8;
        let borrow: bool;
        if reverse {
            (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
        } else {
            (new_vx, borrow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
        }
        let new_vf = if borrow { 0 } else { 1 };
        self.v_registers[x] = new_vx;
        self.v_registers[0xF] = new_vf;
    }

    /// OR VX |= VY - Bitwise OR between VX and VY
    pub fn op_reg_or(&mut self, x: usize, y: usize) {
        self.v_registers[x] |= self.v_registers[y];
    }

    /// SHR VX >>= 1 - Bitwise shift left or right one
    pub fn op_shift(&mut self, x: usize, right: bool) {
        let bit;
        if right {
            bit = self.v_registers[x] & 1;
            self.v_registers[x] >>= 1;
        } else {
            bit = (self.v_registers[x] >> 7) & 1;
            self.v_registers[x] <<= 1;
        }
        self.v_registers[0xF] = bit;
    }

    /// SNE VX != VY - Skip next instruction if VX != VY
    pub fn op_reg_sne(&mut self, x: usize, y: usize) {
        if self.v_registers[x] != self.v_registers[y] {
            self.program_counter += 2;
        }
    }

    /// LD I = NNN - Load register I with value NNN
    pub fn op_i_ld(&mut self, operation: u16) {
        let nnn = operation & 0xFFF;
        self.i_register = nnn;
    }

    /// RND Vx = Rand & NN
    pub fn op_rnd(&mut self, operation: u16, x: usize) {
        let nn = (operation & 0xFF) as u8;
        let rand: u8 = random();
        self.v_registers[x] = rand & nn;
    }

    /// LD VX = DT - Load VX with Delay Timer value
    pub fn op_ld_dt(&mut self, x: usize) {
        self.v_registers[x] = self.emulator.get_mut().get_memory().get_delay_timer()
    }

    /// ADD I += VX - Add Vx to I
    pub fn op_add_i(&mut self, x: usize) {
        let vx = self.v_registers[x] as u16;
        self.i_register = self.i_register.wrapping_add(vx);
    }

    /// LD I = FONT - Load font into i register
    pub fn op_ld_font(&mut self, x: usize) {
        let c = self.v_registers[x] as u16;
        self.i_register = c * 5;
    }
}

impl EmulatorComponent for CPU {
    fn reset(&mut self) {
        self.v_registers = [0; NUMBER_OF_REGISTERS];
        self.i_register = 0;
        self.stack = [0; STACK_SIZE];
        self.program_counter = START_ADDRESS;
        self.stack_pointer = 0;
    }
}