use channel::Channel;

pub struct Dma {
    control: u32,

    irq_en: bool,

    channel_irq_en: u8,
    channel_irq_flags: u8,

    force_irq: bool,

    irq_dummy: u8,

    channels: [Channel; 7],
}

impl Dma {
    pub fn new() -> Dma {
        Dma {
            control: 0x07654321,

            irq_en: false,

            channel_irq_en: 0,
            channel_irq_flags: 0,

            force_irq: false,

            irq_dummy: 0,

            channels: [Channel::new(); 7],
        }
    }

    fn irq(&self) -> bool {
        let channel_irq = self.channel_irq_flags & self.channel_irq_en;

        self.force_irq || (self.irq_en && channel_irq != 0)
    }

    pub fn interrupt(&self) -> u32 {
        let mut r = 0;

        r |= self.irq_dummy as u32;
        r |= (self.force_irq as u32) << 15;
        r |= (self.channel_irq_en as u32) << 16;
        r |= (self.irq_en as u32) << 23;
        r |= (self.channel_irq_flags as u32) << 24;
        r |= (self.irq() as u32) << 31;

        r
    }

    pub fn set_interrupt(&mut self, value: u32) {
        self.irq_dummy = (value & 0x3f) as u8;

        self.force_irq = (value >> 15) & 1 != 0;

        self.channel_irq_en = ((value >> 16) & 0x7f) as u8;

        self.irq_en = (value >> 23) & 1 != 0;

        let ack = ((value >> 24) & 0x3f) as u8;
        self.channel_irq_flags &= !ack;
    }

    pub fn control(&self) -> u32 {
        self.control
    }

    pub fn set_control(&mut self, value: u32) {
        self.control = value;
    }

    pub fn channel(&self, port: Port) -> &Channel {
        &self.channels[port as usize]
    }

    pub fn channel_mut(&mut self, port: Port) -> &mut Channel {
        &mut self.channels[port as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Port {
    MdecIn = 0,
    MdecOut = 1,
    GPU = 2,
    CDROM = 3,
    SPU = 4,
    Pio = 5,
    Otc = 6,
}

impl Port {
    pub fn from_index(index: u32) -> Port {
        match index {
            0 => Port::MdecIn,
            1 => Port::MdecOut,
            2 => Port::GPU,
            3 => Port::CDROM,
            4 => Port::SPU,
            5 => Port::Pio,
            6 => Port::Otc,
            n => panic!("Invalid port {}", n),
        }
    }
}
