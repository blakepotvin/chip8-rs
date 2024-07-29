use std::cell::RefCell;
use std::rc::Rc;
use crate::cpu::CPU;
use crate::display::Display;
use crate::input::Input;
use crate::memory::Memory;

/// Represents the CHIP-8 emulator itself and its internal components
///
/// Constructor will initiate memory with default font set.
pub struct Emulator {
    cpu: Option<Box<CPU>>,
    memory: Option<Box<Memory>>,
    display: Option<Box<Display>>,
    input: Option<Box<Input>>,
}

pub trait EmulatorComponent {
    fn reset(&mut self);
}

impl Emulator {
    /// Constructor
    pub fn new(&mut self) -> Self {
        let emulator = Rc::new(RefCell::new(Emulator { cpu: None, memory: None, display: None, input: None }));
        let emulator_mut = emulator.borrow_mut();
        emulator_mut.cpu = Some(Box::new(CPU::new(emulator.clone())));
        emulator_mut.memory = Some(Box::new(Memory::new(emulator.clone())));
        emulator_mut.display = Some(Box::new(Display::new(emulator.clone())));
        emulator_mut.input = Some(Box::new(Input::new(emulator.clone())));
        emulator
    }

    pub fn get_cpu(&mut self) -> &mut CPU {
        &mut self.cpu
    }

    pub fn get_memory(&self) -> &Memory {
        &self.memory
    }

    /// Split an opcode into 4 nibbles
    fn split_operation(operation: u16) -> (u16, u16, u16, u16) {
        let digit1 = (operation & 0xF000) >> 12;
        let digit2 = (operation & 0x0F00) >> 8;
        let digit3 = (operation & 0x00F0) >> 4;
        let digit4 = operation & 0x000F;
        (digit1, digit2, digit3, digit4)
    }

    fn fetch(&mut self) -> u16 {
        let operation = self.memory.as_ref().fetch_word(self.cpu.get_program_counter());
        self.cpu.set_program_counter(self.cpu.get_program_counter() + 2);
        operation
    }

    fn execute(&mut self, operation: u16) {
        let (digit1, digit2, digit3, digit4) = Emulator::split_operation(operation);
        match (digit1, digit2, digit3, digit4) {
            // NOP - No Operation
            (0, 0, 0, 0) => return,
            // CLS - Clear Screen
            (0, 0, 0xE, 0) => self.display.op_cls(),
            // RET - Return from subroutine
            (0, 0, 0xE, 0xE) => self.cpu.op_ret(),
            // JMP NNN - Move program counter to given address
            (1, _, _, _) => self.cpu.op_jmp(operation),
            // CALL NNN - Goto subroutine
            (2, _, _, _) => self.cpu.op_call(operation),
            // SKIP VX == NN
            (3, _, _, _) => self.cpu.op_se(operation, digit2.into()),
            // SKIP VX != NN
            (4, _, _, _) => self.cpu.op_sne(operation, digit2.into()),
            // SKIP VX == VY
            (5, _, _, 0) => self.cpu.op_reg_se(digit2.into(), digit3.into()),
            // LD VX = NN
            (6, _, _, _) => self.cpu.op_ld(operation, digit2.into()),
            // ADD VX += NN
            (7, _, _, _) => self.cpu.op_add(operation, digit2.into()),
            // OR VX |= VY
            (8, _, _, 1) => self.cpu.op_reg_or(digit2.into(), digit3.into()),
            // ADD VX += VY
            (8, _, _, 4) => self.cpu.op_reg_add(digit2.into(), digit3.into()),
            // SUB VX -= VY
            (8, _, _, 5) => self.cpu.op_reg_sub(digit2.into(), digit3.into(), false),
            // SHR VX
            (8, _, _, 6) => self.cpu.op_shift(digit2.into(), true),
            // SUB VX = VY - VX
            (8, _, _, 7) => self.cpu.op_reg_sub(digit2.into(), digit3.into(), true),
            // SHL VX
            (8, _, _, 0xE) => self.cpu.op_shift(digit2.into(), false),
            // LD VX = VY
            (8, _, _, _) => self.cpu.op_reg_ld(digit2.into(), digit3.into()),
            // SKIP VX != VY
            (9, _, _, _) => self.cpu.op_reg_sne(digit2.into(), digit3.into()),
            // LD I = NNN
            (0xA, _, _, _) => self.cpu.op_i_ld(operation),
            // JMP V0 + NNN
            (0xB, _, _, _) => self.cpu.op_reg_jmp(operation),
            // RND Vx = Rand & NN
            (0xC, _, _, _) => self.cpu.op_rnd(operation, digit2.into()),
            // DRW Vx Vy
            (0xD, _, _, _) => self.display.op_drw(self.cpu.get_register_value(digit2.into()).into(), self.cpu.get_register_value(digit3.into()).into(), digit4.into()),
            // SKP Vx
            (0xE, _, 9, 0xE) => self.input.op_skp(digit2.into(), false),
            // SKNP Vx
            (0xE, _, 0xA, 1) => self.input.op_skp(digit2.into(), true),
            // LD Vx = DT
            (0xF, _, 0, 7) => self.cpu.op_ld_dt(digit2.into()),
            // LD Vx K **BLOCKING**
            (0xF, _, 0, 0xA) => self.input.op_ld_wait(digit2.into()),
            // LD DT = VX
            (0xF, _, 1, 5) => self.memory.op_ld_dt(digit2.into()),
            // LD ST = VX
            (0xF, _, 1, 8) => self.memory.op_ld_st(digit2.into()),
            // ADD I += VX
            (0xF, _, 1, 0xE) => self.cpu.op_add_i(digit2.into()),
            // LD I = Font
            (0xF, _, 2, 9) => self.cpu.op_ld_font(digit2.into()),
            // STR V0 - VX into I
            (0xF, _, 5, 5) => self.memory.op_str(digit2.into()),
            // LD I into V0 - VX
            (0xF, _, 6, 5) => self.memory.op_ld(digit2.into()),
            // Invalid opcode
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {operation}"),
        }
    }

    pub fn tick(&mut self) {
        let operation = self.fetch();
        self.execute(operation);
    }
}

impl EmulatorComponent for Emulator {
    /// Resets the emulator to the initial starting state.
    fn reset(&mut self) {
        self.cpu.reset();
        self.memory.reset();
        self.display.reset();
        self.input.reset();
    }
}
