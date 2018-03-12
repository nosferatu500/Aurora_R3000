use bios::Bios;
use ram::Ram;
use dma::Dma;
use dma::Port;
use channel::*;
use gpu::Gpu;

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
    dma: Dma,
    gpu: Gpu,
}

impl Interconnect {
    pub fn new(bios: Bios) -> Interconnect {
        Interconnect {
            bios: bios,
            ram: Ram::new(),
            dma: Dma::new(),
            gpu: Gpu::new(),
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
            return self.set_dma_reg(offset, value);
        }

        if let Some(offset) = map::GPU.contains(masked_address) {
            match offset {
                0 => self.gpu.gp0(value),
                4 => self.gpu.gp1(value),
                _ => panic!("GPU write {} {}", offset, value)
            }
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
            return self.dma_reg(offset);
        }

        if let Some(offset) = map::TIMERS.contains(masked_address) {
            println!("Unimplemented TIMERS register: {:#08x}", offset);
            return 0;
        }

        if let Some(offset) = map::GPU.contains(masked_address) {
            return match offset {
                4 => 0x1c000000,
                _ => 0,
            }
        }

        panic!("Unhandled fetch 32bit address {:08x}", masked_address);
    }

    fn dma_reg(&self, offset: u32) -> u32 {
        let major = (offset & 0x70) >> 4;
        let minor = offset & 0xf;

        match major {
            0 ... 6 => {
                let channel = self.dma.channel(Port::from_index(major));

                match minor {
                    8 => channel.control(),
                    _ => panic!("Unhandled DMA read {:x}", offset)
                }
            },
            7 => {
                match minor {
                    0 => self.dma.control(),
                    4 => self.dma.interrupt(),
                    _ => panic!("Unhandled DMA read {:x}", offset)
                }
            },
            _ => panic!("Unhandled DMA read {:x}", offset),
        }
    }

    fn set_dma_reg(&mut self, offset: u32, value: u32) {
        let major = (offset & 0x70) >> 4;
        let minor = offset & 0xf;

        let active_port = match major {
            0 ... 6 => {
                let port = Port::from_index(major);
                let channel = self.dma.channel_mut(port);

                match minor {
                    0 => channel.set_base(value),
                    4 => channel.set_block_control(value),
                    8 => channel.set_control(value),
                    _ => panic!("Unhandled DMA write {:x}", offset)
                }

                if channel.active() {
                    Some(port)
                } else {
                    None
                }
            },
            7 => {
                match minor {
                    0 => self.dma.set_control(value),
                    4 => self.dma.set_interrupt(value),
                    _ => panic!("Unhandled DMA write {:x}", offset)
                }

                None
            },
            _ => panic!("Unhandled DMA write {:x}", offset),
        };

        if let Some(port) = active_port {
            self.do_dma(port);
        }
    }

    fn do_dma(&mut self, port: Port) {
        match self.dma.channel(port).sync() {
            Sync::LinkedList => self.do_dma_linked_list(port),
            _ => self.do_dma_block(port),
        }
    }

    fn do_dma_linked_list(&mut self, port: Port) {
        let channel = self.dma.channel_mut(port);

        let mut addr = channel.base() & 0x1ffffc;

        if channel.direction() == Direction::ToRam {
            panic!("Invalid DMA direction for dma linked list");
        }

        if port != Port::GPU {
            panic!("Attempted linked list DMA. Port: {}", port as u8);
        }

        loop {
            let header = self.ram.load32(addr);

            let mut remsz = header >> 24;

            while remsz > 0 {
                addr = (addr + 4) & 0x1ffffc;

                let command = self.ram.load32(addr);

                self.gpu.gp0(command);

                remsz -= 1;
            }

            if header & 0x800000 != 0 {
                break;
            }

            addr = header & 0x1ffffc;
        }

        channel.done();
    }

    fn do_dma_block(&mut self, port: Port) {
        let channel = self.dma.channel_mut(port);

        let increment: i32 = match channel.step() {
            Step::Increment => 4,
            Step::Decrement => -4,
        };

        let mut addr = channel.base();

        let mut remsz = match channel.transfer_size() {
            Some(n) => n,
            None => panic!("Error DMA block transfer size")
        };

        while remsz > 0 {
            let current_address = addr & 0x1ffffc;

            match channel.direction() {
                Direction::FromRam => {
                    let source_word = self.ram.load32(current_address);

                    match port {
                        Port::GPU => self.gpu.gp0(source_word),
                        _ => panic!("Unhandled DMA destination port {}", port as u8),
                    }
                },
                Direction::ToRam => {
                    let source_word = match port {
                        Port::Otc => match remsz {
                            1 => 0xffffff,
                            _ => addr.wrapping_sub(4) & 0x1fffff,
                        },
                        _ => panic!("Unhandled DMA src port {}", port as u8),
                    };

                    self.ram.store32(current_address, source_word);
                }
            }

            if increment > 0 {
                addr = addr.wrapping_add(4);
            } else {
                addr = addr.wrapping_sub(4);
            }

            remsz -= 1;
        }

        channel.done();
    }
}