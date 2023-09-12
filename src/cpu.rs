use crate::display::Display;

#[derive(Clone, Debug)]
pub(crate) struct Cpu {
    pub(crate) v: [u8; 16],
    stack: [u16; 16],
    // stack pointer
    sp: u8,
    // 16 8-bit registers
    i: u16,
    // program counter
    pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            i: 0,
            pc: 0x200,
        }
    }

    pub(crate) fn cycle(&mut self, display: &mut Display, memory: &mut [u8; 4096]) {
        let opcode = self.fetch_opcode(memory);
        // println!("opcode: {:#X} {:#X} {:#X} {:#X} {:#X} ", opcode, b1, b2, b3, b4);
        self.pc += 2;
        self.execute(opcode, display, memory);
    }

    fn fetch_opcode(&self, memory: &[u8; 4096]) -> (u8, u8, u8, u8, u16) {
        let pc = self.pc;
        let opcode = (( memory[pc as usize] as u16) << 8) | memory[(pc + 1) as usize] as u16;
        let b1: u8 = ((opcode & 0xF000) >> 12) as u8;
        let b2: u8 = ((opcode & 0x0F00) >> 8)  as u8;
        let b3: u8 = ((opcode & 0x00F0) >> 4) as u8;
        let b4: u8 = (opcode & 0x000F) as u8;
        (b1, b2, b3, b4, opcode)
    }

    fn execute(&mut self, opcode: (u8, u8, u8, u8, u16), display: &mut Display, memory: &mut [u8; 4096]) {
        match opcode {
            (0x00, 0x00, 0x00, 0x00, _) => { self.op_0nnn_call(opcode); }
            (0x00, 0x00, 0x0E, 0x00, _) => { self.op_00e0_display(display); }
            (0x00, 0x00, 0x0E, 0x0E, _) => { self.op_00ee_flow(); }
            (0x01, _, _, _, _) => { self.op_1nnn_flow(opcode); }
            (0x02, _, _, _, _) => { self.op_2nnn_flow(opcode); }
            (0x03, _, _, _, _) => { self.op_3xnn_cond(opcode); }
            (0x04, _, _, _, _) => { self.op_4xnn_cond(opcode); }
            (0x05, _, _, 0x00, _) => { self.op_5xy0_cond(opcode); }
            (0x06, _, _, _, _) => { self.ld_v(opcode); }
            (0x07, _, _, _, _) => { self.add_x(opcode); }
            (0x08, _, _, 0x00, _) => { self.op_8xy4_assig(opcode); }
            (0x08, _, _, 0x02, _) => { self.op_8xy2_bitop(opcode); }
            (0x08, _, _, 0x03, _) => { self.op_8xy3_bitop(opcode); }
            (0x08, _, _, 0x04, _) => { self.op_8xy4_math(opcode); }
            (0x08, _, _, 0x05, _) => { self.op_8xy5_math(opcode); }
            (0x08, _, _, 0x0E, _) => { self.op_8xye_bitop(opcode); }
            (0x09, _, _, 0x00, _) => { self.op_9xy0_cond(opcode); }
            (0x0A, _, _, _, _) => { self.ld_i(opcode); }
            (0x0B, _, _, _, _) => { self.bnnn_flow(opcode); }
            (0x0C, _, _, _, _) => { self.rnd_v(opcode); }
            (0x0D, _, _, _, _) => { self.drw(opcode, display, memory); }
            (0x0F, _, 0x01, 0x05, _) => { self.op_fx15_timer(opcode); }
            (0x0F, _, 0x01, 0x0E, _) => { self.op_fx1e_mem(opcode); }
            (0x0F, _, 0x02, 0x09, _) => { self.op_fx29_mem(opcode); }
            (0x0F, _, 0x05, 0x05, _) => { self.op_fx55_mem(opcode, memory); }
            (0x0F, _, 0x06, 0x05, _) => { self.op_fx65_mem(opcode, memory); }
            _ => {
                println!("Unknown opcode: {:#06X}", opcode.4);
                panic!();
            }
        }
    }

    /**
     * 6XNN - LD Vx, byte
     * Set Vx = NN.
     * The interpreter puts the value NN into register Vx.
     */
    fn ld_v(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let register = opcode.1 as usize;
        let value = (opcode.2 << 4) | opcode.3;
        // println!("LD V{}, {}", register, value);
        self.v[register] = value;
    }

    /**
     * Set I = nnn.
     *
     * The value of register I is set to nnn.
     */
    fn ld_i(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let value = ((opcode.1 as u16) << 8) | ((opcode.2 as u16) << 4) | opcode.3 as u16;
        // println!("LD I, {}", value);
        self.i = value;
    }

    /**
     Cxkk - RND Vx, byte
    Set Vx = random byte AND kk.

    The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
     */
    fn rnd_v(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let register = opcode.1 as usize;
        let rand = rand::random::<u8>();
        let kk = ((opcode.2) << 4) | opcode.3;
        let value = rand & ((opcode.2 << 4) | opcode.3) & kk;
        // println!("RND V{}, {}", register, value);
        self.v[register] = value;
    }

    /**
    Skip next instruction if Vx = kk.

    The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
     */
    fn op_3xnn_cond(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let nn = ((opcode.2) << 4) | opcode.3;
        // println!("SE V{}, {}", x, kk);
        if self.v[x] == nn {
            // println!("Skip next instruction");
            self.pc += 2;
        }
    }

    fn op_4xnn_cond(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let nn = ((opcode.2) << 4) | opcode.3;
        // println!("SE V{}, {}", x, kk);
        if self.v[x] != nn {
            // println!("Skip next instruction");
            self.pc += 2;
        }
    }


    /**
    Skip next instruction if Vx = kk.

    The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
     */
    fn op_5xy0_cond(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let y = opcode.2 as usize;
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    /**
    Dxyn - DRW Vx, Vy, nibble
    Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.

    Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    Each row of 8 pixels is read as bit-coded starting from memory location I;
    I value does not change after the execution of this instruction. As described above,
    VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is
    drawn, and to 0 if that does not happen.
    */
    fn drw(&mut self, opcode: (u8, u8, u8, u8, u16), display: &mut Display, memory: &[u8; 4096]) {
        let x = self.v[opcode.1 as usize];
        let y = self.v[opcode.2 as usize];
        let n = opcode.3;
        println!("DRW x{}, y{}, n{}", x, y, n);
        self.v[0x0f] = 0;
        // height of n pixels
        for i in 0..n {
            let value = memory[(self.i + i as u16) as usize];
            let y = y + i;
            // width of 8 pixels
            for j in 0..8 {
                let x = x + j;
                // decode the color from the memory value
                let pixel = ((value >> (7 - j)) & 0x01) == 1;
                if display.set_pixel_xor(x, y, pixel) {
                    self.v[0x0f] = 1;
                }
            }
        }
    }

    /**
    7xkk - ADD Vx, byte
    Set Vx = Vx + kk.

    Adds the value kk to the value of register Vx, then stores the result in Vx.
    */
    fn add_x(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let kk = ((opcode.2) << 4) | opcode.3;
        self.v[x] += kk;
    }

    /// Goto nnn
    fn op_1nnn_flow(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let nnn = ((opcode.1 as u16) << 8) | ((opcode.2 as u16) << 4) | opcode.3 as u16;
        // println!("GOTO {:#X}", nnn);
        self.pc = nnn;
    }

    /// Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
    fn op_8xy4_math(&mut self, opcodes: (u8, u8, u8, u8, u16)) {
        let x = opcodes.1 as usize;
        let y = opcodes.2 as usize;
        println!("8XY4 X={}, Y={}", x, y);
        self.v[x] += self.v[y];
    }

    /// Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
    fn op_8xy5_math(&mut self, opcodes: (u8, u8, u8, u8, u16)) {
        let x = opcodes.1 as usize;
        let y = opcodes.2 as usize;
        println!("8XY5 X={}, Y={}", x, y);
        self.v[x] -= self.v[y];
    }

    fn op_2nnn_flow(&mut self, opcodes: (u8, u8, u8, u8, u16)) {
        let nnn = ((opcodes.1 as u16) << 8) | ((opcodes.2 as u16) << 4) | opcodes.3 as u16;
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
        println!("2NNN nnn={:#X}", nnn)
    }

    fn op_fx15_timer(&mut self, opcode: (u8, u8, u8, u8, u16)) {
      let x = opcode.1 as usize;
        self.i += x as u16;
    }

    ///Adds VX to I. VF is not affected.
    fn op_fx1e_mem(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        println!("FX1E X={}", x);
        self.i += self.v[x] as u16;
    }

    ///The first 0x200 bytes of memory contains hexadecimal digits and all of them are 5 byte
    fn op_fx29_mem(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        println!("FX29 X={}", x);
        self.i += (self.v[x] * 5) as u16;
    }

    fn bnnn_flow(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let nnn = ((opcode.1 as u16) << 8) | ((opcode.2 as u16) << 4) | opcode.3 as u16;
        self.pc = nnn + self.v[0] as u16;
    }
    fn op_8xy4_assig(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let y = opcode.2 as usize;
        self.v[x] = self.v[y];
    }
    fn op_8xy2_bitop(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let y = opcode.2 as usize;
        self.v[x] &= self.v[y];
    }
    fn op_8xy3_bitop(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let y = opcode.2 as usize;
        self.v[x] ^= self.v[y];
    }

    fn op_8xye_bitop(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let y = opcode.2 as usize;
        let most_significant_bit = self.v[y] >> 7;
        self.v[0x0f] = most_significant_bit;
        self.v[x] <<= 1;
    }

    fn op_9xy0_cond(&mut self, opcode: (u8, u8, u8, u8, u16)) {
        let x = opcode.1 as usize;
        let y = opcode.2 as usize;
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }
    fn op_00e0_display(&self, display: &mut Display) {
        display.clear();
    }
    fn op_00ee_flow(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    fn op_0nnn_call(&self, opcode: (u8, u8, u8, u8, u16)) {
        let nnn = ((opcode.1 as u16) << 8) | ((opcode.2 as u16) << 4) | opcode.3 as u16;
        println!("0NNN nnn={:#X}", nnn);
        // todo : what to do ?
    }

    fn op_fx55_mem(&self, opcode: (u8, u8, u8, u8, u16), memory: &mut [u8; 4096]) {
        let x = opcode.1 as usize;
        for i in 0..x {
            memory[self.i as usize + i] = self.v[i];
        }
        println!("FX55 X={}", x)
    }

    fn op_fx65_mem(&mut self, opcode: (u8, u8, u8, u8, u16), memory: &[u8; 4096]) {
        let x = opcode.1 as usize;
        for i in 0..x {
            self.v[i] = memory[self.i as usize + i];
        }
        println!("FX65 X={}", x)
    }
}