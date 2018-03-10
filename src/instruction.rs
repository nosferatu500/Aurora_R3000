pub struct Instruction {
    pub data: u32,
}

impl Instruction {
    pub fn new(data: u32) -> Instruction {
        Instruction {
            data
        }
    }

    pub fn opcode(&self) -> u32 {
        self.data >> 26
    }

    pub fn special_opcode(&self) -> u32 {
        self.data & 0x3f
    }

    pub fn cop_opcode(&self) -> u32 {
        self.rs()
    }

    pub fn rs(&self) -> u32 {
        (self.data >> 21) & 0x1f
    }

    pub fn rt(&self) -> u32 {
        (self.data >> 16) & 0x1f
    }

    pub fn rd(&self) -> u32 {
        (self.data >> 11) & 0x1f
    }

    pub fn sa(&self) -> u32 {
        (self.data >> 6) & 0x1f
    }

    pub fn imm(&self) -> u32 {
        self.data & 0xffff        
    }

    pub fn imm_se(&self) -> u32 {
        ((self.data & 0xffff) as i16) as u32
    }

    pub fn target(&self) -> u32 {
        self.data & 0x3ffffff
    }
}