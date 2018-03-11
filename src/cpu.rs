use interconnect::Interconnect;
use instruction::Instruction;

pub struct Cpu {
    pc: u32,
    next_pc: u32,
    regs: [u32; 32],
    inter: Interconnect,

    out_regs: [u32; 32],    

    sr: u32,

    load: (u32, u32),

    hi: u32,
    lo: u32,

    current_pc: u32,

    cause: u32,

    epc: u32,

    branch: bool,
    delay_slot: bool,
}

impl Cpu {
    pub fn new(inter: Interconnect) -> Cpu {
        let mut regs = [0xdeadbeef; 32];

        regs[0] = 0;

        let pc = 0xbfc00000; 

        Cpu {
            pc,
            next_pc: pc.wrapping_add(4),
            regs,
            inter,

            out_regs: regs,

            sr: 0,

            load: (0, 0),

            hi: 0xdeadbeef,
            lo: 0xdeadbeef,

            current_pc: 0xdeadbeef,

            cause: 0xdeadbeef,

            epc: 0xdeadbeef,

            branch: false,
            delay_slot: false,
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
        let instruction = self.load32(self.pc);

        self.current_pc = self.pc;

        if self.current_pc % 4 != 0 {
            self.exception(Exception::LoadAddressError);
            return;
        }

        // Wrapping_add for overflow issue.
        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        //Emulate load delay slot.
        let (reg, value) = self.load;
        self.set_reg(reg, value);

        self.load = (0, 0);

        self.delay_slot = self.branch;
        self.branch = false;

        self.decode_and_execute(Instruction::new(instruction));

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
                    0b000010 => self.op_srl(sa, rt, rd),
                    0b000011 => self.op_sra(sa, rt, rd),
                    0b000100 => self.op_sllv(rs, rt, rd),
                    0b000110 => self.op_srlv(rs, rt, rd),
                    0b000111 => self.op_srav(rs, rt, rd),
                    0b001000 => self.op_jr(rs),
                    0b001001 => self.op_jalr(rs, rd),
                    0b001100 => self.op_syscall(),
                    0b010000 => self.op_mfhi(rd),
                    0b010001 => self.op_mthi(rs),
                    0b010010 => self.op_mflo(rd),
                    0b010011 => self.op_mtlo(rs),
                    0b011001 => self.op_multu(rs, rt),
                    0b011010 => self.op_div(rs, rt),
                    0b011011 => self.op_divu(rs, rt),
                    0b100000 => self.op_add(rs, rt, rd),
                    0b100100 => self.op_and(rs, rt, rd),
                    0b101011 => self.op_sltu(rs, rt, rd),
                    0b100001 => self.op_addu(rs, rt, rd),
                    0b100011 => self.op_subu(rs, rt, rd),
                    0b100101 => self.op_or(rs, rt, rd),
                    0b100111 => self.op_nor(rs, rt, rd),
                    0b101010 => self.op_slt(rs, rt, rd),
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
                    0b000000 => self.op_mfc0(rt, rd),
                    0b000100 => self.op_mtc0(rt, rd),
                    0b010000 => self.op_rfe(instruction.data),
                    _ => panic!("\n\nUnhandled COP0 instruction: {:06b}\n\n", instruction.cop_opcode())
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
            0b001010 => self.op_slti(rs, rt, imm_se),
            0b001011 => self.op_sltiu(rs, rt, imm_se),
            0b001100 => self.op_andi(rs, rt, imm),
            0b001101 => self.op_ori(rs, rt, imm),
            0b001111 => self.op_lui(rt, imm),
            0b100000 => self.op_lb(rs, rt, imm_se),
            0b100001 => self.op_lh(rs, rt, imm_se),
            0b100011 => self.op_lw(rs, rt, imm_se),
            0b100100 => self.op_lbu(rs, rt, imm_se),
            0b100101 => self.op_lhu(rs, rt, imm_se),
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

    fn op_srl(&mut self, sa: u32, rt: u32, rd: u32) {
        let res = (self.reg(rt)) >> sa;

        self.set_reg(rd, res);
    }

    fn op_sra(&mut self, sa: u32, rt: u32, rd: u32) {
        let res = (self.reg(rt) as i32) >> sa;

        self.set_reg(rd, res as u32);
    }

    fn op_sllv(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rt) << (self.reg(rs) & 0x1f);

        self.set_reg(rd, res);
    }

    fn op_srav(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = (self.reg(rt) as i32) >> (self.reg(rs) & 0x1f);

        self.set_reg(rd, res as u32);
    }

    fn op_srlv(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rt) >> (self.reg(rs) & 0x1f);

        self.set_reg(rd, res);
    }

    fn op_jr(&mut self, rs: u32) {
        self.next_pc = self.reg(rs);

        self.branch = true;
    }

    fn op_jalr(&mut self, rs: u32, rd: u32) {
        let pc = self.next_pc;
        self.set_reg(rd, pc);
        self.next_pc = self.reg(rs);

        self.branch = true;
    }

    fn op_syscall(&mut self) {
        self.exception(Exception::SysCall);
    }

    fn op_mfhi(&mut self, rd: u32) {
        let hi = self.hi;
        self.set_reg(rd, hi);
    }

    fn op_mthi(&mut self, rs: u32) {
        self.hi = self.reg(rs)
    }

    fn op_mflo(&mut self, rd: u32) {
        let lo = self.lo;
        self.set_reg(rd, lo);
    }

    fn op_mtlo(&mut self, rs: u32) {
        self.lo = self.reg(rs)
    }

    fn op_multu(&mut self, rs: u32, rt: u32) {
        let a = self.reg(rs) as u64;
        let b = self.reg(rt) as u64;

        let res = a * b;

        self.hi = (res >> 32) as u32;
        self.lo = res as u32;
    }

    fn op_div(&mut self, rs: u32, rt: u32) {
        let n = self.reg(rs) as i32;
        let d = self.reg(rt) as i32;

        if d == 0 {
            self.hi = n as u32;

            if n >= 0 {
                self.lo = 0xffffffff;
            } else {
                self.lo = 1;
            }
        } else if n as u32 == 0x80000000 && d == -1 {
            self.hi = 0;
            self.lo = 0x80000000;
        } else {
            self.hi = (n % d) as u32;
            self.lo = (n / d) as u32;
        }
    }

    fn op_divu(&mut self, rs: u32, rt: u32) {
        let n = self.reg(rs);
        let d = self.reg(rt);

        if d == 0 {
            self.hi = n;
            self.lo = 0xffffffff;
        } else {
            self.hi = n % d;
            self.lo = n / d;
        }
    }

    fn op_add(&mut self, rs: u32, rt: u32, rd: u32) {
        let lhs = self.reg(rs) as i32;
        let rhs = self.reg(rt) as i32;

        match lhs.checked_add(rhs) {
            Some(res) => self.set_reg(rd, res as u32),
            None => self.exception(Exception::Overflow),
        };

        
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

    fn op_subu(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rs).wrapping_sub(self.reg(rt));

        self.set_reg(rd, res);
    }

    fn op_mfc0(&mut self, rt: u32, rd: u32) {
        let value = match rd {
            12 => self.sr,
            13 => self.cause,
            14 => self.epc,
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

    fn op_rfe(&mut self, data: u32) {
        if data & 0x3f != 0b010000 {
            panic!("Invalid COP0_RFE instruction: {}", data);
        }

        //Restore from exception mode.
        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= mode << 2;
    }

    fn op_or(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = self.reg(rs) | self.reg(rt);

        self.set_reg(rd, res);
    }

    fn op_nor(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = !(self.reg(rs) | self.reg(rt));

        self.set_reg(rd, res);
    }

    fn op_slt(&mut self, rs: u32, rt: u32, rd: u32) {
        let res = (self.reg(rs) as i32) < (self.reg(rt) as i32);

        self.set_reg(rd, res as u32);
    }

    fn op_j(&mut self, target: u32) {
        self.next_pc = target << 2 | (self.pc & 0xf0000000);

        self.branch = true;
    }

    fn op_jal(&mut self, target: u32) {
        let pc = self.next_pc;

        self.next_pc = target << 2 | (self.pc & 0xf0000000);

        self.set_reg(31, pc);

        self.branch = true;
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
        let pc = self.next_pc;

        self.set_reg(31, pc);

        if value == 1 {
            self.branch(imm_se);
        }
    }

    fn op_bgezal(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let value = self.reg(rs);
        let pc = self.next_pc;

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
            Some(res) => self.set_reg(rt, res as u32),
            None => self.exception(Exception::Overflow),
        };
    }

    fn op_addiu(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let res = self.reg(rs).wrapping_add(imm_se);

        self.set_reg(rt, res);
    }

    fn op_slti(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let res = (self.reg(rs) as i32) < imm_se as i32;

        self.set_reg(rt, res as u32);
    }

    fn op_sltiu(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let res = (self.reg(rs)) < imm_se;

        self.set_reg(rt, res as u32);
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

    fn op_lh(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring load while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.load16(addr) as i16;

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
    fn op_lhu(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring load while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.load16(addr);

        self.load = (rt, value as u32);
    }

    // Incomplete probably?
    fn op_lw(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring load while cache is isolated");
            return;
        }

        let addr = self.reg(base).wrapping_add(offset);

        if addr % 4 == 0 {
            let value = self.load32(addr);

            self.load = (rt, value);
        } else {
            self.exception(Exception::LoadAddressError)
        }
        
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

        if addr % 2 == 0 {
            let value = self.reg(rt);

            self.store16(addr, value as u16);
        } else {
            self.exception(Exception::StoreAddressError);
        }
    }

    // Incomplete probably?
    fn op_sw(&mut self, base: u32, rt: u32, offset: u32) {
        if self.sr & 0x10000 != 0 {
            println!("Ignoring store while cache is isolated");
            return;
        }
        
        let addr = self.reg(base).wrapping_add(offset);
        
        if addr % 2 == 0 {
            let value = self.reg(rt);
            self.store32(addr, value);
        } else {
            self.exception(Exception::StoreAddressError);
        }
    }

    fn branch(&mut self, offset: u32) {
        let offset = offset << 2;

        self.next_pc = self.pc.wrapping_add(offset);

        self.branch = true;
    }

    fn exception(&mut self, cause: Exception) {
        let handler = match self.sr & (1 << 22) != 0 {
            true => 0xfbc00180,
            false => 0x80000080,
        };

        //Switch to exception mode.
        let mode = self.sr & 0x3f;
        self.sr &= 0x3f;
        self.sr |= (mode << 2) & 0x3f;

        self.cause = (cause as u32) << 2;
        self.epc = self.current_pc;

        if self.delay_slot {
            self.epc = self.epc.wrapping_sub(4);
            self.cause |= 1 << 31;
        }

        self.pc = handler;
        self.next_pc = self.pc.wrapping_add(4);
    }
}

enum Exception {
    SysCall = 0x8,
    Overflow = 0xc,
    LoadAddressError = 0x4,
    StoreAddressError = 0x5,
}
