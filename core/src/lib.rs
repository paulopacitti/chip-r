use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
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
    keys: [bool; NUM_KEYS],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    stack: [u16; STACK_SIZE],
}

impl Emulator {
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, i: usize, pressed: bool) {
        self.keys[i] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();

        self.ram[start..end].copy_from_slice(data);
    }

    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            sp: 0,
            dt: 0,
            st: 0,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            keys: [false; NUM_KEYS],
            v_reg: [0; NUM_REGS],
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
        self.keys = [false; NUM_KEYS];
        self.v_reg = [0; NUM_REGS];
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

    /// ADD: register_1 += register_2, with carry
    fn add(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;

        let (result, carry) = self.v_reg[r1].overflowing_add(self.v_reg[r2]);
        let vf = if carry { 1 } else { 0 }; // VF is the flag register in CHIP-8, indicating carry (overflow)

        self.v_reg[r1] = result;
        self.v_reg[0xF] = vf;
    }

    /// ADDIW: register_1 += value, wrapped add
    fn addiw(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;
        self.v_reg[r1] = self.v_reg[r1].wrapping_add(value);
    }

    /// AND: register &= register_2
    fn and(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] &= self.v_reg[r2];
    }

    /// CALL: call subroutine
    fn call(&mut self, op: u16) {
        let address = op & 0xFFF;
        self.push(self.pc);
        self.pc = address;
    }

    /// CLEAR => clear screen
    fn clear(&mut self) {
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
    }

    // https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#dxyn-display
    fn draw(&mut self, register_1: u16, register_2: u16, height: u16) {
        // coordinates indicate where the sprite will be drawed
        let x = self.v_reg[register_1 as usize] as u16;
        let y = self.v_reg[register_2 as usize] as u16;

        // Keep track if any pixels were flipped
        let mut flipped = false;
        for row in 0..height {
            // Load pixels from sprite, address is stores on i_reg
            let address = self.i_reg + row as u16;
            let pixels = self.ram[address as usize];

            // In CHIP-8, all sprites are 8 pixels wide
            for column in 0..8 {
                // Use a mask to fetch current pixel's bit, only flip if a 1 (check if is necessary to draw or not)
                if (pixels & (0b1000_0000 >> column)) != 0 {
                    // Sprites should clip around screen, so apply modulo
                    let current_x = (x + column) as usize % SCREEN_WIDTH;
                    let current_y = (y + row) as usize % SCREEN_HEIGHT;

                    // Get our pixel's index for our 1D screen array
                    let idx = SCREEN_WIDTH * current_y + current_x;
                    flipped |= self.screen[idx];
                    self.screen[idx] ^= true;
                }
            }
        }

        // Set flag if any bit was flipped
        if flipped {
            self.v_reg[0xF] = 1;
        } else {
            self.v_reg[0xF] = 0;
        }
    }

    fn execute(&mut self, op: u16) {
        let digit_1 = (op & 0xF000) >> 12;
        let digit_2 = (op & 0x0F00) >> 8;
        let digit_3 = (op & 0x00F0) >> 4;
        let digit_4 = op & 0x000F;

        match (digit_1, digit_2, digit_3, digit_4) {
            (0, 0, 0, 0) => return,                                 // NOP
            (0, 0, 0xE, 0) => self.clear(),                         // CLEAR SCREEN
            (0, 0, 0xE, 0xE) => self.ret(),                         // RET
            (1, _, _, _) => self.jmp(op),                           // JMP
            (2, _, _, _) => self.call(op),                          // CALL
            (3, _, _, _) => self.seq(op, digit_2),                  // SEQ
            (4, _, _, _) => self.snq(op, digit_2),                  // SNQ
            (5, _, _, 0) => self.seqr(digit_2, digit_3),            // SEQR
            (6, _, _, _) => self.ld(op, digit_2),                   // LD
            (7, _, _, _) => self.addiw(op, digit_2),                // ADDIW
            (8, _, _, 0) => self.mv(digit_2, digit_3),              // MV
            (8, _, _, 1) => self.or(digit_2, digit_3),              // OR
            (8, _, _, 2) => self.and(digit_2, digit_3),             // AND
            (8, _, _, 3) => self.xor(digit_2, digit_3),             // XOR
            (8, _, _, 4) => self.add(digit_2, digit_3),             // ADD
            (8, _, _, 5) => self.sub(digit_2, digit_3),             // SUB
            (8, _, _, 6) => self.shr(digit_2),                      // SHR
            (8, _, _, 7) => self.sub2(digit_2, digit_3),            // SUB2
            (8, _, _, 0xE) => self.shl(digit_2),                    // SHL
            (9, _, _, 0) => self.snqr(digit_2, digit_3),            // SNQR
            (0xA, _, _, _) => self.ldi(op),                         // LDI
            (0xB, _, _, _) => self.jmp2(op),                        // JMP2
            (0xC, _, _, _) => self.rnd(op, digit_2),                // RND
            (0xD, _, _, _) => self.draw(digit_2, digit_3, digit_4), // DRAW
            (0xE, _, 9, 0xE) => self.skp(digit_2),                  //SKP
            (0xE, _, 0xA, 1) => self.snp(digit_2),                  // SNP
            (0xF, _, 0, 7) => self.ldt(digit_2),                    // LDT
            (0xF, _, 0, 0xA) => self.wkp(digit_2),                  // WKP
            (0xF, _, 1, 5) => self.sdt(digit_2),                    // SDT
            (0xF, _, 1, 8) => self.sst(digit_2),                    // SST
            (0xF, _, 1, 0xE) => self.iadd(digit_2),                 // IADD
            (0xF, _, 2, 9) => self.ldf(digit_2),                    // LDF
            (0xF, _, 3, 3) => self.sbcd(digit_2),                   // SBCD
            (0xF, _, 5, 5) => self.strr(digit_2),                   // STRR,
            (0xF, _, 6, 5) => self.ldr(digit_2),                    // LDR

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

    /// IADD: increment register I with a offset stored in another register
    fn iadd(&mut self, register: u16) {
        let r1 = register as usize;
        self.i_reg = self.i_reg.wrapping_add(self.v_reg[r1] as u16);
    }

    /// JMP: jump to address encoded in op code
    fn jmp(&mut self, op: u16) {
        let address = op & 0xFFF;
        self.pc = address;
    }

    /// JMP2: jump to address encoded in op code + V0
    fn jmp2(&mut self, op: u16) {
        let address = op & 0xFFF;
        self.pc = (self.v_reg[0] as u16) + address;
    }

    /// LD: The interpreter puts the value into register_1
    fn ld(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;
        self.v_reg[r1] = value;
    }

    /// LDI: The value of register I is set to a value encoded in the opcode.
    fn ldi(&mut self, op: u16) {
        let address = op & 0xFFF;
        self.i_reg = address;
    }

    /// LDF: load font address into register I
    fn ldf(&mut self, register: u16) {
        let r1 = register as usize;
        self.i_reg = 5 * (self.v_reg[r1] as u16);
    }

    /// LDR: load a slice from the memory on the registers
    fn ldr(&mut self, range: u16) {
        for i in 0..=range {
            self.v_reg[i as usize] = self.ram[(self.i_reg as usize) + (i as usize)];
        }
    }

    /// LDT: load delta timer value in a register
    fn ldt(&mut self, register: u16) {
        let r1 = register as usize;
        self.v_reg[r1] = self.dt;
    }

    /// MV: Stores the value of register_2 in register_1
    fn mv(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] = self.v_reg[r2];
    }

    /// OR: register_1 |= register_2
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

    /// RET: Return from Subroutine
    fn ret(&mut self) {
        let ret_address = self.pop();
        self.pc = ret_address;
    }

    /// RND: generate a random number with a bitwise AND base on a register value and store the result in the same register
    fn rnd(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;

        let rng: u8 = random();
        self.v_reg[r1] = rng & value;
    }

    /// SBCD: store the BCD value of a register in memory.
    fn sbcd(&mut self, register: u16) {
        let r1 = register as usize;
        let value = self.v_reg[r1] as f32;

        let hundreds = (value / 100.0).floor() as u8;
        let tens = ((value / 10.0) % 10.0).floor() as u8;
        let ones = (value % 10.0).floor() as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    /// SDT: set/store delta timer.
    fn sdt(&mut self, register: u16) {
        let r1 = register as usize;
        self.dt = self.v_reg[r1];
    }

    /// SEQ: skip if register equal to value.
    fn seq(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;
        if self.v_reg[r1] == value {
            self.pc += 2;
        }
    }

    /// SEQE: skip if register_1 equal to register_2.
    fn seqr(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        if self.v_reg[r1] == self.v_reg[r2] {
            self.pc += 2;
        }
    }

    /// SHL: shift-left value in register, add flag in the VF.
    fn shl(&mut self, register: u16) {
        let r1 = register as usize;

        let msb = (self.v_reg[r1] >> 7) & 1;
        self.v_reg[r1] <<= 1;
        self.v_reg[0xF] = msb; // set flag with the bit shifted
    }

    /// SHL: shift-right value in register, add flag in the VF.
    fn shr(&mut self, register: u16) {
        let r1 = register as usize;

        //  Unfortunately, there isn’t a built-in Rust u8 operator to catch the dropped bit, so we will have to do it ourself
        let lsb = self.v_reg[r1] & 1;
        self.v_reg[r1] >>= 1;
        self.v_reg[0xF] = lsb; // set flag with the bit shifted
    }

    /// SKP: skip if key is pressed.
    fn skp(&mut self, register: u16) {
        let r1 = register as usize;
        let key = self.keys[self.v_reg[r1] as usize];
        if key {
            self.pc += 2;
        }
    }

    /// SNP: skip if key is not pressed.
    fn snp(&mut self, register: u16) {
        let r1 = register as usize;
        let key = self.keys[self.v_reg[r1] as usize];
        if !key {
            self.pc += 2;
        }
    }

    /// SNQ: skip if register not equal to value.
    fn snq(&mut self, op: u16, register: u16) {
        let r1 = register as usize;
        let value = (op & 0xFF) as u8;
        if self.v_reg[r1] != value {
            self.pc += 2;
        }
    }

    /// SNQR: skip if register_1 not equal to register_2.
    fn snqr(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;

        if self.v_reg[r1] != self.v_reg[r2] {
            self.pc += 2;
        }
    }

    /// STRR: store a range of registers values in memory.
    fn strr(&mut self, range: u16) {
        for i in 0..=range {
            self.ram[(self.i_reg as usize) + (i as usize)] = self.v_reg[i as usize];
        }
    }

    /// SDT: set/store sound timer.
    fn sst(&mut self, register: u16) {
        let r1 = register as usize;
        self.st = self.v_reg[r1];
    }

    /// SUB: register_1 -= register_2, with carry.
    fn sub(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;

        let (result, borrow) = self.v_reg[r1].overflowing_sub(self.v_reg[r2]);
        let vf = if borrow { 0 } else { 1 }; // VF is the flag register in CHIP-8, indicating borrow (underflow)

        self.v_reg[r1] = result;
        self.v_reg[0xF] = vf;
    }

    /// SUB: register_2 -= register_1, with carry.
    fn sub2(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;

        let (result, borrow) = self.v_reg[r2].overflowing_sub(self.v_reg[r1]);
        let vf = if borrow { 0 } else { 1 }; // VF is the flag register in CHIP-8, indicating borrow (underflow)

        self.v_reg[r1] = result;
        self.v_reg[0xF] = vf;
    }

    // WKP: wait for key to be pressed.
    fn wkp(&mut self, register: u16) {
        let r1 = register as usize;
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[r1] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            // Redo opcode
            self.pc -= 2;
        }
    }

    /// XOR: register_1 ^= register_2.
    fn xor(&mut self, register_1: u16, register_2: u16) {
        let r1 = register_1 as usize;
        let r2 = register_2 as usize;
        self.v_reg[r1] ^= self.v_reg[r2];
    }
}
