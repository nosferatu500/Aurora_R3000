use interconnect::Interconnect;
use instruction::Instruction;

pub struct Cpu {
    pc: u32,
    regs: [u32; 32],
    inter: Interconnect,
    next_instruction: Instruction,

    out_regs: [u32; 32],    

    sr: u32,

    load: (u32, u32),
}

impl Cpu {
    pub fn new(inter: Interconnect) -> Cpu {
        let mut regs = [0xdeadbeef; 32];

        regs[0] = 0;

        Cpu {
            pc: 0xbfc00000,
            regs,
            inter,
            next_instruction: Instruction::new(0x0),

            out_regs: regs,

            sr: 0,

            load: (0, 0),
        }
    }

    fn reg(&self, index: u32) -> u32 {
        self.regs[index as usize]
    }

    fn set_reg(&mut self, index: u32, value: u32) {
        self.out_regs[index as usize] = value;
        self.out_regs[0] = 0;
    }

    pub fn run_next_instruction(&mut self) {
        let pc = self.pc;

        //Emulate delay slot.
        let instruction = Instruction::new(self.next_instruction.data);

        let data = self.load32(pc);

        // Wrapping_add for overflow issue.
        self.pc = pc.wrapping_add(4);

        self.next_instruction = Instruction::new(data);

        //Emulate load delay slot.
        let (reg, value) = self.load;
        self.set_reg(reg, value);

        self.load = (0, 0);

        self.decode_and_execute(instruction);

        self.regs = self.out_regs;
    }

    fn store8(&mut self, addr: u32, value: u8) {
        self.inter.store8(addr, value)
    }

    fn load8(&self, addr: u32) -> u8 {
        self.inter.load8(addr)
    }

    fn store16(&mut self, addr: u32, value: u16) {
        self.inter.store16(addr, value)
    }

    fn load16(&self, addr: u32) -> u16 {
        self.inter.load16(addr)
    }

    fn store32(&mut self, addr: u32, value: u32) {
        self.inter.store32(addr, value)
    }

    fn load32(&self, addr: u32) -> u32 {
        self.inter.load32(addr)
    }

    fn decode_and_execute(&mut self, instruction: Instruction) {
        
        let imm = instruction.imm();
        let rt = instruction.rt();
        let rs = instruction.rs();

        let rd = instruction.rd();
        let sa = instruction.sa();

        let imm_se = instruction.imm_se();

        let target = instruction.target(); 

        match instruction.opcode() {
            0b000000 => {
                match instruction.special_opcode() {
                    0b000000 => self.op_sll(sa, rt, rd),
                    0b001000 => self.op_jr(rs),
                    0b001001 => self.op_jalr(rs, rd),
                    0b100000 => self.op_add(rs, rt, rd),
                    0b100100 => self.op_and(rs, rt, rd),
                    0b101011 => self.op_sltu(rs, rt, rd),
                    0b100001 => self.op_addu(rs, rt, rd),
                    0b100101 => self.op_or(rs, rt, rd),
                    _ => panic!("\n\nUnhandled SPECIAL instruction: {:06b}\n\n", instruction.special_opcode())
                }
            },
            0b000001 => {
                match instruction.regimm_condition() {
                    0b00000 => self.op_bltz(rs, rt, imm_se),
                    0b00001 => self.op_bgez(rs, rt, imm_se),
                    0b10000 => self.op_bltzal(rs, rt, imm_se),
                    0b10001 => self.op_bgezal(rs, rt, imm_se),
                    _ => panic!("\n\nUnhandled REGIMM_CONDITION instruction: {:05b}\n\n", instruction.regimm_condition())
                }
            },
            0b010000 => {
                match instruction.cop_opcode() {
                    0b00000 => self.op_mfc0(rt, rd),
                    0b00100 => self.op_mtc0(rt, rd),
                    _ => panic!("\n\nUnhandled COP0 instruction: {:05b}\n\n", instruction.cop_opcode())
                }
            },
            0b000010 => self.op_j(target),
            0b000011 => self.op_jal(target),
            0b000100 => self.op_beq(rs, rt, imm_se),
            0b000101 => self.op_bne(rs, rt, imm_se),
            0b000110 => self.op_blez(rs, rt, imm_se),
            0b000111 => self.op_bgtz(rs, rt, imm_se),
            0b001000 => self.op_addi(rs, rt, imm_se),
            0b001001 => self.op_addiu(rs, rt, imm_se),
            0b001100 => self.op_andi(rs, rt, imm),
            0b001101 => self.op_ori(rs, rt, imm),
            0b001111 => self.op_lui(rt, imm),
            0b100000 => self.op_lb(rs, rt, imm_se),
            0b100011 => self.op_lw(rs, rt, imm_se),
            0b100100 => self.op_lbu(rs, rt, imm_se),
            0b101000 => self.op_sb(rs, rt, imm_se),
            0b101001 => self.op_sh(rs, rt, imm_se),
            0b101011 => self.op_sw(rs, rt, imm_se),
            _ => panic!("\n\nUnhandled COMMON instruction: {:06b}\n\n", instruction.opcode())
        }
    }

    fn op_sll(&mut self, sa: u32, rt: u32, rd: u32) {
        let res = self.reg(rt) << sa;

        self.set_reg(rd, res);
    }

    fn op_jr(&mut self, rs: u32) {
        self.pc = self.reg(rs);
    }

    fn op_jalr(&mut self, rs: u32, rd: u32) {
        let pc = self.pc;
        self.set_reg(rd, pc);
        self.pc = self.reg(rs);
    }

    fn op_add(&mut self, rs: u32, rt: u32, rd: u32) {
        let lhs = self.reg(rs) as i32;
        let rhs = self.reg(rt) as i32;

        let res = match lhs.checked_add(rhs) {
            Some(res) => res as u32,
            None => panic!("ADD is overflow."),
        };

        self.set_reg(rd, res);
    }

    fn op_and(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rs) & self.reg(rt);

        self.set_reg(rd, res as u32);
    }

    fn op_sltu(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rs) < self.reg(rt);

        self.set_reg(rd, res as u32);
    }

    fn op_addu(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rs).wrapping_add(self.reg(rt));

        self.set_reg(rd, res);
    }

    fn op_mfc0(&mut self, rt: u32, rd: u32) {
        let value = match rd {
            12 => self.sr,
            13 => panic!("\n\nTry to write data to CAUSE (only-read register) {:b}\n\n", rd),
            _ => panic!("\n\nUnhandled MTC0 instruction: {:05b}\n\n", rd)
        };

        self.load = (rt, value);
    }

    fn op_mtc0(&mut self, rt: u32, rd: u32) {
        let res = self.reg(rt);

        match rd {
            3 | 5 | 6 | 7 | 9 | 11 => if res != 0 { panic!("\n\nUnhandled MTC0 instruction: {:b}\n\n", res) }, 
            12 => self.sr = res,
            13 => if res != 0 { panic!("\n\nTry to write data to CAUSE (only-read register) {:b}\n\n", res) },
            _ => panic!("\n\nUnhandled MTC0 instruction: {:05b}\n\n", rd)
        }
    }

    fn op_or(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rs) | self.reg(rt);

        self.set_reg(rd, res);
    }

    fn op_j(&mut self, target: u32) {
        self.pc = target << 2 | (self.pc & 0xf0000000);
    }

    fn op_jal(&mut self, target: u32) {
        let pc = self.pc;

        self.pc = target << 2 | (self.pc & 0xf0000000);

        self.set_reg(31, pc);
    }

    fn op_beq(&mut self, rs: u32, rt: u32, imm_se: u32) {
        if self.reg(rs) == self.reg(rt) {
            self.branch(imm_se);
        }
    }

    fn op_bne(&mut self, rs: u32, rt: u32, imm_se: u32) {
        if self.reg(rs) != self.reg(rt) {
            self.branch(imm_se);
        }
    }

    fn op_bgtz(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs) as i32;

        if value != 0 {
            self.branch(imm_se);
        }
    }

    fn op_bltz(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs);

        if value == 1 {
            self.branch(imm_se);
        }
    }

    fn op_bgez(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs);

        if value == 0 {
            self.branch(imm_se);
        }
    }

    fn op_bltzal(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs);
        let pc = self.pc;

        self.set_reg(31, pc);

        if value == 1 {
            self.branch(imm_se);
        }
    }

    fn op_bgezal(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs);
        let pc = self.pc;

        self.set_reg(31, pc);

        if value == 0 {
            self.branch(imm_se);
        }
    }

    fn op_blez(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs) as i32;

        if value == 0 || value == 1 {
            self.branch(imm_se);
        }
    }

    fn op_addi(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let imm = imm_se as i32;
        let reg = self.reg(rs) as i32;

        let res = match reg.checked_add(imm) {
            Some(res) => res as u32,
            None => panic!("ADDI is overflow."),
        };

        self.set_reg(rt, res);
    }

    fn op_addiu(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let res = self.reg(rs).wrapping_add(imm_se);

        self.set_reg(rt, res);
    }

    fn op_andi(&mut self, rs: u32, rt: u32, imm: u32) {
        let res = self.reg(rs) & imm;

        self.set_reg(rt, res);
    }

    fn op_ori(&mut self, rs: u32, rt: u32, imm: u32) {
        let res = self.reg(rs) | imm;

        self.set_reg(rt, res);
    }

    fn op_lui(&mut self, rt: u32, imm: u32) {
        let res = imm << 16;

        self.set_reg(rt, res);
    }

    // Incomplete probably?
    fn op_lb(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring load while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.load8(addr) as i8;

        self.load = (rt, value as u32);
    }

    // Incomplete probably?
    fn op_lbu(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring load while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.load8(addr);

        self.load = (rt, value as u32);
    }

    // Incomplete probably?
    fn op_lw(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring load while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.load32(addr);

        self.load = (rt, value);
    }

    // Incomplete probably?
    fn op_sb(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring store while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.reg(rt);

        self.store8(addr, value as u8);
    }

    // Incomplete probably?
    fn op_sh(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring store while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.reg(rt);

        self.store16(addr, value as u16);
    }

    // Incomplete probably?
    fn op_sw(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring store while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.reg(rt);

        self.store32(addr, value);
    }

    fn branch(&mut self, offset: u32) {
        let offset = offset << 2;

        let mut pc = self.pc;

        pc = pc.wrapping_add(offset);

        //Because we have overhead in run_next_instruction()
        pc = pc.wrapping_sub(4);
        
        self.pc = pc;
    }
}