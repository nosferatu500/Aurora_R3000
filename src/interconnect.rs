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

    pub const SPU: Range = Range(0x1f801c00, 640);    

    pub const EXPANSION_1: Range = Range(0x1f000000, 512 * 1024);    
    pub const EXPANSION_2: Range = Range(0x1f802000, 66);

    pub const INTERRUPT_CONTROL: Range = Range(0x1f801070, 8);   

    pub const TIMERS: Range = Range(0x1f801100, 48);   

    pub const DMA: Range = Range(0x1f801080, 128);   

    pub const GPU: Range = Range(0x1f801810, 8);   
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

    pub fn store8(&mut self, addr: u32, value: u8) {
        let masked_address = map::mask_region(addr);

        if let Some(offset) = map::EXPANSION_2.contains(masked_address) {
            println!("Unimplemented EXPANSION_2 register: {:#08x}", offset);
            return;
        }

        if let Some(offset) = map::RAM.contains(masked_address) {
            return self.ram.store8(offset, value);
        }

        panic!("Unaligned store 8bit address {:08x}", addr);
    }

    pub fn load8(&self, addr: u32) -> u8 {
        let masked_address = map::mask_region(addr);
        
        if let Some(offset) = map::BIOS.contains(masked_address) {
            return self.bios.load8(offset);
        }

        if let Some(offset) = map::RAM.contains(masked_address) {
            return self.ram.load8(offset);
        }
        
        if let Some(offset) = map::EXPANSION_1.contains(masked_address) {
            println!("Unimplemented EXPANSION_1 register: {:#08x}", offset);
            return 0xff;
        }

        panic!("Unhandled fetch 8bit address {:08x}", addr);
    }

    pub fn store16(&mut self, addr: u32, value: u16) {
        if addr % 2 != 0 {
            panic!("Address is not equel for 16bit address {:08x}", addr);    
        }

        let masked_address = map::mask_region(addr);

        if let Some(offset) = map::SPU.contains(masked_address) {
            println!("Unimplemented SPU register: {:#08x}", offset);
            return;
        }

        if let Some(offset) = map::RAM.contains(masked_address) {
            return self.ram.store16(offset, value);
        }

        if let Some(offset) = map::TIMERS.contains(masked_address) {
            println!("Unimplemented TIMERS register: {:#08x}", offset);
            return;
        }

        if let Some(offset) = map::INTERRUPT_CONTROL.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented INTERRUPT_CONTROL yet. Register: {:#08x} : {:08x}", offset, value),
            }
            return;
        }

        panic!("Unaligned store 16bit address {:08x}", masked_address);
    }

    pub fn load16(&self, addr: u32) -> u16 {
        if addr % 2 != 0 {
            panic!("Address is not equel for 16bit address {:08x}", addr);    
        }

        let masked_address = map::mask_region(addr);

        if let Some(offset) = map::RAM.contains(masked_address) {
            return self.ram.load16(offset);
        }

        if let Some(offset) = map::SPU.contains(masked_address) {
            println!("Unimplemented SPU register: {:#08x}", offset);
            return 0;
        }

        if let Some(offset) = map::INTERRUPT_CONTROL.contains(masked_address) {
            println!("Unimplemented INTERRUPT_CONTROL yet. Register: {:#08x}", offset);
            return 0;
        }

        panic!("Unhandled fetch 16bit address {:08x}", addr);
    }

    pub fn store32(&mut self, addr: u32, value: u32) {
        if addr % 4 != 0 {
            panic!("Address is not equel for 32bit address {:08x}", addr);    
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
            return self.ram.store32(offset, value);
        }

        if let Some(offset) = map::RAM_SIZE.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented RAM_SIZE control yet. Register: {:#08x}", offset),
            }
            return;
        }

        if let Some(offset) = map::CACHE_CONTROL.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented CACHE_CONTROL yet. Register: {:#08x}", offset),
            }
            return;
        }

        if let Some(offset) = map::INTERRUPT_CONTROL.contains(masked_address) {
            match offset {
                _ => println!("Unimplemented INTERRUPT_CONTROL yet. Register: {:#08x} : {:08x}", offset, value),
            }
            return;
        }

        if let Some(offset) = map::TIMERS.contains(masked_address) {
            println!("Unimplemented TIMERS register: {:#08x}", offset);
            return;
        }

        if let Some(offset) = map::DMA.contains(masked_address) {
            println!("Unimplemented DMA yet. Register: {:#08x} : {:08x}", offset, value);
            return;
        }

        if let Some(offset) = map::GPU.contains(masked_address) {
            println!("Unimplemented GPU yet. Register: {:#08x} : {:08x}", offset, value);
            return;
        }

        panic!("Unhandled store 32bit address {:08x}", masked_address);
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if addr % 4 != 0 {
            panic!("Address is not equel for 32bit address {:08x}", addr);    
        }

        let masked_address = map::mask_region(addr);
        
        if let Some(offset) = map::BIOS.contains(masked_address) {
            return self.bios.load32(offset);
        }

        if let Some(offset) = map::RAM.contains(masked_address) {
            return self.ram.load32(offset);
        }

        if let Some(offset) = map::INTERRUPT_CONTROL.contains(masked_address) {
            println!("Unimplemented INTERRUPT_CONTROL yet. Register: {:#08x}", offset);
            return 0;
        }

        if let Some(offset) = map::DMA.contains(masked_address) {
            println!("Unimplemented DMA yet. Register: {:#08x}", offset);
            return 0;
        }

        if let Some(offset) = map::GPU.contains(masked_address) {
            return match offset {
                4 => 0x10000000,
                _ => 0,
            }
        }

        panic!("Unhandled fetch 32bit address {:08x}", masked_address);
    }
}