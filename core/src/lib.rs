pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const VREGS_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emulator {
    pc: u16, // program counter
    sp: u16, // stack counter
    dt: u8,  // delay timer
    st: u8,  // sound timer
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
    v_reg: [u8; VREGS_SIZE],
    i_reg: u16,
    stack: [u16; STACK_SIZE],
}

impl Emulator {
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            sp: 0,
            dt: 0,
            st: 0,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v_reg: [0; VREGS_SIZE],
            i_reg: 0,
            stack: [0; STACK_SIZE],
        };

        // copies bitmap fonts set to RAM using slices
        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.sp = 0;
        self.dt = 0;
        self.st = 0;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
        self.v_reg = [0; VREGS_SIZE];
        self.i_reg = 0;
        self.stack = [0; STACK_SIZE];

        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    fn addi(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;
        self.v_reg[r1] = self.v_reg[r1].wrapping_add(value);
    }

    fn and(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] &= self.v_reg[r2];
    }

    // CALL: call subroutine
    fn call(&mut self, op: u16) {
        let address = op & 0xFFF;
        self.push(self.pc);
        self.pc = address;
    }

    fn clean(&mut self) {
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
    }

    fn execute(&mut self, op: u16) {
        let digit_1 = (op & 0xF000) >> 12;
        let digit_2 = (op & 0x0F00) >> 8;
        let digit_3 = (op & 0x00F0) >> 4;
        let digit_4 = op & 0x000F;

        match (digit_1, digit_2, digit_3, digit_4) {
            (0, 0, 0, 0) => return,                      // NOP
            (0, 0, 0xE, 0) => self.clean(),              // CLEAR SCREEN
            (0, 0, 0xE, 0xE) => self.ret(),              // RET
            (1, _, _, _) => self.jmp(op),                // JMP
            (2, _, _, _) => self.call(op),               // CALL
            (3, _, _, _) => self.seq(op, digit_2),       // SEQ
            (4, _, _, _) => self.snq(op, digit_2),       // SNQ
            (5, _, _, 0) => self.seqr(digit_2, digit_3), // SEQR
            (6, _, _, _) => self.set(op, digit_2),       // SET
            (7, _, _, _) => self.addi(op, digit_2),      // ADDI
            (8, _, _, 0) => self.setr(digit_2, digit_3), // SETR
            (8, _, _, 1) => self.or(digit_2, digit_3),   // OR
            (8, _, _, 2) => self.and(digit_2, digit_3),  // AND
            (8, _, _, 3) => self.xor(digit_2, digit_3),  // XOR

            (_, _, _, _) => unimplemented!("Unimplemented op code: { }", op),
        }
    }

    // Big Endian words; opcodes has two bytes length
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte; // bitwise concatenation to gen the opcode
        self.pc += 2;
        op
    }

    // JMP: jump to address encoded in op code
    fn jmp(&mut self, op: u16) {
        let address = op & 0xFFF;
        self.pc = address;
    }

    fn or(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] |= self.v_reg[r2];
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // RET: Return from Subroutine
    fn ret(&mut self) {
        let ret_address = self.pop();
        self.pc = ret_address;
    }

    // SEQ: skip if register equal to value
    fn seq(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFFF) as u8;
        if self.v_reg[r1] == value {
            self.pc += 2;
        }
    }

    fn set(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;
        self.v_reg[r1] = value;
    }

    fn setr(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] = self.v_reg[r2];
    }

    // SEQE: skip if register_1 equal to register_2
    fn seqr(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        if self.v_reg[r1] == self.v_reg[r2] {
            self.pc += 2;
        }
    }

    // SNQ: skip if register not equal to value
    fn snq(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFFF) as u8;
        if self.v_reg[r1] != value {
            self.pc += 2;
        }
    }

    fn xor(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] ^= self.v_reg[r2];
    }
}
