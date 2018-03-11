#[derive(Clone, Copy)]
pub struct Channel {
    enable: bool,
    direction: Direction,
    step: Step,
    sync: Sync,
    trigger: bool,
    chop: bool,
    chop_dma_size: u8,
    chop_cpu_size: u8,
    dummy: u8,

    base: u32,

    block_size: u16,
    block_count: u16,
}

impl Channel {
    pub fn new() -> Channel {
        Channel {
            enable: false,
            direction: Direction::ToRam,
            step: Step::Increment,
            sync: Sync::Manual,
            trigger: false,
            chop: false,
            chop_dma_size: 0,
            chop_cpu_size: 0,
            dummy: 0,

            base: 0,

            block_size: 0,
            block_count: 0,
        }
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    pub fn set_base(&mut self, value: u32) {
        self.base = value & 0xffffff
    }

    pub fn block_control(&self) -> u32 {
        let bs = self.block_size as u32;
        let bc = self.block_count as u32;

        (bc << 16) | bs
    }

    pub fn set_block_control(&mut self, value: u32) {
        self.block_size = value as u16;
        self.block_count = (value >> 16) as u16;
    }

    pub fn active(&self) -> bool {
        let trigger = match self.sync {
            Sync::Manual => self.trigger,
            _ => true,
        };

        self.enable && trigger
    }

    pub fn control(&self) -> u32 {
        let mut r = 0;

        r |= (self.direction as u32) << 0;
        r |= (self.step as u32) << 1;
        r |= (self.chop as u32) << 8;
        r |= (self.sync as u32) << 9;
        r |= (self.chop_dma_size as u32) << 16;
        r |= (self.chop_cpu_size as u32) << 20;
        r |= (self.enable as u32) << 24;
        r |= (self.trigger as u32) << 28;
        r |= (self.dummy as u32) << 29;

        r
    }

    pub fn set_control(&mut self, value: u32) {
        self.direction = match value & 1 != 0 {
            true => Direction::FromRam,
            false => Direction::ToRam,
        };

        self.step = match (value >> 1) & 1 != 0 {
            true => Step::Decrement,
            false => Step::Increment,
        };

        self.chop = (value >> 8) & 1 != 0;

        self.sync = match (value >> 9) & 3 {
            0 => Sync::Manual,
            1 => Sync::Request,
            2 => Sync::LinkedList,
            n => panic!("Unknown DMA_SYNC mode. {}", n),
        };

        self.chop_dma_size = ((value >> 16) & 7) as u8;
        self.chop_cpu_size = ((value >> 20) & 7) as u8;

        self.enable = (value >> 24) & 1 != 0;
        self.trigger = (value >> 28) & 1 != 0;
        self.dummy = ((value >> 29) & 3) as u8;
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn sync(&self) -> Sync {
        self.sync
    }

    pub fn transfer_size(&self) -> Option<u32> {
        let bs = self.block_size as u32;
        let bc = self.block_count as u32;

        match self.sync {
            Sync::Manual => Some(bs),
            Sync::Request => Some(bc * bs),
            Sync::LinkedList => None,
        }
    }

    pub fn done(&mut self) {
        self.enable = false;
        self.trigger = false;
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ToRam = 0,
    FromRam = 1,
}
#[derive(Clone, Copy)]
pub enum Step {
    Increment = 0,
    Decrement = 1,
}
#[derive(Clone, Copy)]
pub enum Sync {
    Manual = 0,
    Request = 1,
    LinkedList = 2,
}
