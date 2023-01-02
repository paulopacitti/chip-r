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
        // TODO: Decode
        // TODO: Execute
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

    // Big Endian words; opcodes has two bytes length
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte; // bitwise concatenation to gen the opcode
        self.pc += 2;
        op
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
