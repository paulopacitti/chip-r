pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const VREGS_SIZE: usize = 16;
const STACK_SIZE: usize = 16;

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
        Self {
            pc: 0x200,
            sp: 0,
            dt: 0,
            st: 0,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v_reg: [0; VREGS_SIZE],
            i_reg: 0,
            stack: [0; STACK_SIZE],
        }
    }
}
