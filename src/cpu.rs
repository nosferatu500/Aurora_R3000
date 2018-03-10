use interconnect::Interconnect;
use instruction::Instruction;

pub struct Cpu {
    pc: u32,
    regs: [u32; 32],
    inter: Interconnect,
}

impl Cpu {
    pub fn new(inter: Interconnect) -> Cpu {
        let mut regs = [0xdeadbeef; 32];

        regs[0] = 0;

        Cpu {
            pc: 0xbfc00000,
            regs,
            inter,
        }
    }

    fn reg(&self, index: u32) -> u32 {
        self.regs[index as usize]
    }

    fn set_reg(&mut self, index: u32, value: u32) {
        self.regs[index as usize] = value;
        self.regs[0] = 0;
    }

    pub fn run_next_instruction(&mut self) {
        let pc = self.pc;

        let data = self.load32(pc);

        // Wrapping_add for overflow issue.
        self.pc = pc.wrapping_add(4);

        let instruction = Instruction::new(data);

        self.decode_and_execute(instruction);
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

        match instruction.opcode() {
            0b000000 => {
                match instruction.special_opcode() {
                    0b000000 => self.op_sll(sa, rt, rd),
                    _ => panic!("Unhandled special instruction: {:06b}", instruction.special_opcode())
                }
            },
            0b001001 => self.op_addiu(rs, rt, imm_se),
            0b001101 => self.op_ori(rs, rt, imm),
            0b001111 => self.op_lui(rt, imm),
            0b101011 => self.op_sw(rs, rt, imm_se),
            _ => panic!("Unhandled instruction: {:06b}", instruction.opcode())
        }
    }

    fn op_sll(&mut self, sa: u32, rt: u32, rd: u32) {
        let res = self.reg(rt) << sa;

        self.set_reg(rd, res);
    }

    fn op_addiu(&mut self, rs: u32, rt: u32, imm_se: u32) {
        let res = self.reg(rs).wrapping_add(imm_se);

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
    fn op_sw(&mut self, base: u32, rt: u32, offset: u32) {
        let addr = self.reg(base).wrapping_add(offset);
        let value = self.reg(rt);

        self.store32(addr, value);
    }
}