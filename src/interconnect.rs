use bios::Bios;

mod map {
    pub struct Range(u32, u32);

    impl Range {
        pub fn contains(self, addr: u32) -> Option<u32> {
            let Range(start, length) = self;

            if addr >= start && addr < start + length {
                Some(addr - start)
            } else {
                None
            }
        }
    }

    pub const BIOS: Range = Range(0xbfc00000, 512 * 1024);

    pub const MEM_CONTROL: Range = Range(0x1f801000, 36);
}

pub struct Interconnect {
    bios: Bios,
}

impl Interconnect {
    pub fn new(bios: Bios) -> Interconnect {
        Interconnect {
            bios: bios,
        }
    }

    pub fn store32(&mut self, addr: u32, value: u32) {
        if addr % 4 != 0 {
            panic!("Unaligned store 32bit address {:08x}", addr);    
        }

        if let Some(offset) = map::MEM_CONTROL.contains(addr) {
            match offset {
                0 => if value != 0x1f000000 {
                    panic!("Expansion 1 has incorrect address: {:#08x}", value);
                },
                4 => if value != 0x1f802000 {
                    panic!("Expansion 2 has incorrect address: {:#08x}", value);
                },
                _ => println!("Incorrect MEM_CONTROL register: {:#08x}", addr),
            }
            return;
        }

        panic!("Unhandled store 32bit address {:08x}", addr);
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if addr % 4 != 0 {
            panic!("Unaligned load 32bit address {:08x}", addr);    
        }
        
        if let Some(offset) = map::BIOS.contains(addr) {
            return self.bios.load32(offset);
        }

        panic!("Unhandled fetch 32bit address {:08x}", addr);
    }
}