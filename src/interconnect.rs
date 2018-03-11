use bios::Bios;
use ram::Ram;

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

    const REGION_MASK: [u32; 8] = [
        //KUSEG: 2048MB
        0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
        //KSEG0: 512MB
        0x7fffffff,
        //KSEG1: 512MB
        0x1fffffff,
        //KSEG2: 1024MB
        0xffffffff, 0xffffffff,
    ];

    pub fn mask_region(addr: u32) -> u32 {
        let index = (addr >> 29) as usize;
        addr & REGION_MASK[index]
    }

    pub const BIOS: Range = Range(0x1fc00000, 512 * 1024);

    pub const RAM: Range = Range(0x00000000, 2 * 1024 * 1024);

    pub const MEM_CONTROL: Range = Range(0x1f801000, 36);

    pub const RAM_SIZE: Range = Range(0x1f801060, 4);

    pub const CACHE_CONTROL: Range = Range(0xfffe0130, 4);    
}

pub struct Interconnect {
    bios: Bios,
    ram: Ram,
}

impl Interconnect {
    pub fn new(bios: Bios) -> Interconnect {
        Interconnect {
            bios: bios,
            ram: Ram::new(),
        }
    }

    pub fn store32(&mut self, addr: u32, value: u32) {
        if addr % 4 != 0 {
            panic!("Unaligned store 32bit address {:08x}", addr);    
        }

        let masked_address = map::mask_region(addr);

        if let Some(offset) = map::MEM_CONTROL.contains(masked_address) {
            match offset {
                0 => if value != 0x1f000000 {
                    panic!("Expansion 1 has incorrect address: {:#08x}", value);
                },
                4 => if value != 0x1f802000 {
                    panic!("Expansion 2 has incorrect address: {:#08x}", value);
                },
                _ => println!("Unimplemented MEM_CONTROL register: {:#08x}", masked_address),
            }
            return;
        }

        if let Some(offset) = map::RAM.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented RAM control yet. Register: {:#08x}", masked_address),
            }
            return;
        }

        if let Some(offset) = map::RAM_SIZE.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented RAM_SIZE control yet. Register: {:#08x}", masked_address),
            }
            return;
        }

        if let Some(offset) = map::CACHE_CONTROL.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented CACHE_CONTROL yet. Register: {:#08x}", masked_address),
            }
            return;
        }

        panic!("Unhandled store 32bit address {:08x}", masked_address);
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if addr % 4 != 0 {
            panic!("Unaligned load 32bit address {:08x}", addr);    
        }

        let masked_address = map::mask_region(addr);
        
        if let Some(offset) = map::BIOS.contains(masked_address) {
            return self.bios.load32(offset);
        }

        if let Some(offset) = map::RAM.contains(masked_address) {
            return self.ram.load32(offset);
        }

        panic!("Unhandled fetch 32bit address {:08x}", masked_address);
    }
}