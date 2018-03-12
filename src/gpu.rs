pub struct Gpu {
    page_base_x: u8,
    page_base_y: u8,

    semi_transparency: u8,

    texture_depth: TextureDepth,
    
    dithering: bool,

    draw_to_display: bool,

    force_set_mask_bit: bool,

    preserve_masked_pixels: bool,

    field: Field,

    textrure_disable: bool,

    hres: HorisontalRes,
    vres: VerticalRes,

    vmode: VMode,

    display_depth: DisplayDepth,

    interlaced: bool,

    display_disable: bool,

    interrupt: bool,

    dma_direction: DmaDirection,

    rectangle_texture_x_flip: bool,
    rectangle_texture_y_flip: bool,

    texture_window_x_mask: u8,
    texture_window_y_mask: u8,

    texture_window_x_offset: u8,
    texture_window_y_offset: u8,

    drawing_area_left: u16,
    drawing_area_top: u16,
    drawing_area_right: u16,
    drawing_area_bottom: u16,

    drawing_x_offset: i16,
    drawing_y_offset: i16,

    display_vram_x_start: u16,
    display_vram_y_start: u16,

    display_horizontal_start: u16,
    display_horizontal_end: u16,

    display_line_start: u16,
    display_line_end: u16,

    gp0_command: CommandBuffer,
    gp0_command_remaining: u32,
    gp0_command_method: fn(&mut Gpu),

    gp0_mode: Gp0Mode,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            page_base_x: 0,
            page_base_y: 0,

            semi_transparency: 0,

            texture_depth: TextureDepth::T4Bit,
            
            dithering: false,

            draw_to_display: false,

            force_set_mask_bit: false,

            preserve_masked_pixels: false,

            field: Field::Top,

            textrure_disable: false,

            hres: HorisontalRes::from_fields(0, 0),
            vres: VerticalRes::Y240Lines,

            vmode: VMode::NTSC,

            display_depth: DisplayDepth::D15Bits,

            interlaced: false,

            display_disable: true,

            interrupt: false,

            dma_direction: DmaDirection::Off,

            rectangle_texture_x_flip: false,
            rectangle_texture_y_flip: false,

            texture_window_x_mask: 0,
            texture_window_y_mask: 0,

            texture_window_x_offset: 0,
            texture_window_y_offset: 0,

            drawing_area_left: 0,
            drawing_area_top: 0,
            drawing_area_right: 0,
            drawing_area_bottom: 0,

            drawing_x_offset: 0,
            drawing_y_offset: 0,

            display_vram_x_start: 0,
            display_vram_y_start: 0,

            display_horizontal_start: 0,
            display_horizontal_end: 0,

            display_line_start: 0,
            display_line_end: 0,

            gp0_command: CommandBuffer::new(),
            gp0_command_remaining: 0,
            gp0_command_method: Gpu::gp0_nop,

            gp0_mode: Gp0Mode::Command,
        }
    }

    pub fn status(&self) -> u32 {
        let mut r = 0u32;

        r |= (self.page_base_x as u32) << 0;
        r |= (self.page_base_y as u32) << 4;
        r |= (self.semi_transparency as u32) << 5;
        r |= (self.texture_depth as u32) << 7;
        r |= (self.dithering as u32) << 9;
        r |= (self.draw_to_display as u32) << 10;
        r |= (self.force_set_mask_bit as u32) << 11;
        r |= (self.preserve_masked_pixels as u32) << 12;
        r |= (self.field as u32) << 13;
        r |= (self.textrure_disable as u32) << 15;
        r |= self.hres.into_status();
        //r |= (self.vres as u32) << 19;
        r |= (self.vmode as u32) << 20;
        r |= (self.display_depth as u32) << 21;
        r |= (self.interlaced as u32) << 22;
        r |= (self.display_disable as u32) << 23;
        r |= (self.interrupt as u32) << 24;

        r |= 1 << 26;
        r |= 1 << 27;
        r |= 1 << 28;

        r |= (self.dma_direction as u32) << 29;
        r |= 0 << 31;
        
        let dma_request = match self.dma_direction {
            DmaDirection::Off => 0,
            DmaDirection::Fifo => 1,
            DmaDirection::CpuToGpu0 => (r >> 28) & 1,
            DmaDirection::VRamToCpu => (r >> 27) & 1,
        };

        r |= dma_request << 25;

        r
    }

    pub fn gp0(&mut self, value: u32) {
        if self.gp0_command_remaining == 0 {
            let opcode = (value >> 24) & 0xff;

            let (length, method) = match opcode {
                0x00 => (1, Gpu::gp0_nop as fn(&mut Gpu)),
                0x01 => (1, Gpu::gp0_clear_cache as fn(&mut Gpu)),
                0x28 => (5, Gpu::gp0_quad_mono_opaque as fn(&mut Gpu)),
                0x2c => (9, Gpu::gp0_quad_texture_blend_opaque as fn(&mut Gpu)),
                0x30 => (6, Gpu::gp0_triangle_shaded_opaque as fn(&mut Gpu)),
                0x38 => (8, Gpu::gp0_quad_shaded_opaque as fn(&mut Gpu)),
                0xa0 => (3, Gpu::gp0_image_load as fn(&mut Gpu)),
                0xc0 => (3, Gpu::gp0_image_store as fn(&mut Gpu)),
                0xe1 => (1, Gpu::gp0_draw_mode as fn(&mut Gpu)),
                0xe2 => (1, Gpu::gp0_texture_window as fn(&mut Gpu)),
                0xe3 => (1, Gpu::gp0_drawing_area_top_left as fn(&mut Gpu)),
                0xe4 => (1, Gpu::gp0_drawing_area_bottom_right as fn(&mut Gpu)),
                0xe5 => (1, Gpu::gp0_drawing_offset as fn(&mut Gpu)),
                0xe6 => (1, Gpu::gp0_mask_bit_setting as fn(&mut Gpu)),
                _ => panic!("Unhandled GPU0 command {:08x}", value),
            };

            self.gp0_command_remaining = length;
            self.gp0_command_method = method;

            self.gp0_command.clear();
        }
        
        self.gp0_command_remaining -= 1;

        match self.gp0_mode {
            Gp0Mode::Command => {
                self.gp0_command.push_word(value);

                if self.gp0_command_remaining == 0 {
                    (self.gp0_command_method)(self);
                }
            },
            Gp0Mode::ImageLoad => {
                if self.gp0_command_remaining == 0 {
                    self.gp0_mode = Gp0Mode::Command;
                }
            }
        }

        
        
        
    }

    pub fn gp1(&mut self, value: u32) {
        let opcode = (value >> 24) & 0xff;

        match opcode {
            0x00 => self.gp1_reset(value),
            0x01 => self.gp1_reset_command_buffer(),
            0x02 => self.gp1_acknowledge_irq(),
            0x03 => self.gp1_display_enable(value),
            0x04 => self.gp1_dma_direction(value),
            0x05 => self.gp1_display_vram_start(value),
            0x06 => self.gp1_display_horizontal_range(value),
            0x07 => self.gp1_display_vertical_range(value),
            0x08 => self.gp1_display_mode(value),
            _ => panic!("Unhandled GPU1 command {:08x}", opcode),
        }
    }

    fn gp0_nop(&mut self) {

    }

    fn gp0_clear_cache(&mut self) {
        println!("clear_cache!!!");
    }

    fn gp0_image_store(&mut self) {
        let res = self.gp0_command[2];

        let width = res & 0xffff;
        let height = res >> 16;

        println!("image_store!!!");
    }

    fn gp0_image_load(&mut self) {
        let res = self.gp0_command[2];

        let width = res & 0xffff;
        let height = res >> 16;

        let image_size = width * height;

        let image_size = (image_size + 1) & !1;

        self.gp0_command_remaining = image_size / 2;
        self.gp0_mode = Gp0Mode::ImageLoad;
    }

    fn gp0_quad_mono_opaque(&mut self) {
        println!("Draw Mono!!!");
    }

    fn gp0_quad_texture_blend_opaque(&mut self) {
        println!("Draw texture blend!!!");
    }

    fn gp0_triangle_shaded_opaque(&mut self) {
        println!("Draw triangle Shaded!!!");
    }

    fn gp0_quad_shaded_opaque(&mut self) {
        println!("Draw Shaded!!!");
    }

    fn gp0_draw_mode(&mut self) {
        let value = self.gp0_command[0];

        self.page_base_x = (value & 0xf) as u8;
        self.page_base_y = ((value >> 4) & 1) as u8;

        self.semi_transparency = ((value >> 5) & 3) as u8;

        self.texture_depth = match (value >> 7) & 3 {
            0 => TextureDepth::T4Bit,
            1 => TextureDepth::T8Bit,
            2 => TextureDepth::T15Bit,
            n => panic!("Unhandled texture Depth"),
        };

        self.dithering = ((value >> 9) & 1) != 0;
        self.draw_to_display = ((value >> 10) & 1) != 0;
        self.textrure_disable = ((value >> 11) & 1) != 0;
        self.rectangle_texture_x_flip = ((value >> 12) & 1) != 0;
        self.rectangle_texture_y_flip = ((value >> 13) & 1) != 0;
    }

    fn gp0_texture_window(&mut self) {
        let value = self.gp0_command[0];

        self.texture_window_x_mask = (value & 0x1f) as u8;
        self.texture_window_y_mask = ((value >> 5) & 0x1f) as u8;
        self.texture_window_x_offset = ((value >> 10) & 0x1f) as u8;
        self.texture_window_y_offset = ((value >> 15) & 0x1f) as u8;
    }

    fn gp0_drawing_area_top_left(&mut self) {
        let value = self.gp0_command[0];

        self.drawing_area_left = (value & 0x3ff) as u16;
        self.drawing_area_top = ((value  >> 10)& 0x3ff) as u16;
    }

    fn gp0_drawing_area_bottom_right(&mut self) {
        let value = self.gp0_command[0];

        self.drawing_area_right = (value & 0x3ff) as u16;
        self.drawing_area_bottom = ((value  >> 10)& 0x3ff) as u16;
    }

    fn gp0_drawing_offset(&mut self) {
        let value = self.gp0_command[0];

        let x = (value & 0x7ff) as u16;
        let y = ((value  >> 11)& 0x7ff) as u16;

        self.drawing_x_offset = ((x << 5) as i16) >> 5;
        self.drawing_y_offset = ((y << 5) as i16) >> 5;
    }

    fn gp0_mask_bit_setting(&mut self) {
        let value = self.gp0_command[0];

        self.force_set_mask_bit = (value & 1) != 0;
        self.preserve_masked_pixels = (value & 2) != 0;
    }

    fn gp1_reset(&mut self, value: u32) {
            self.page_base_x = 0;
            self.page_base_y = 0;

            self.semi_transparency = 0;

            self.texture_depth = TextureDepth::T4Bit;
            
            self.dithering = false;

            self.draw_to_display = false;

            self.force_set_mask_bit = false;

            self.preserve_masked_pixels = false;

            self.field = Field::Top;

            self.textrure_disable = false;

            self.hres = HorisontalRes::from_fields(0, 0);
            self.vres = VerticalRes::Y240Lines;

            self.vmode = VMode::NTSC;

            self.display_depth = DisplayDepth::D15Bits;

            self.interlaced = false;

            self.display_disable = true;

            self.interrupt = false;

            self.dma_direction = DmaDirection::Off;

            self.rectangle_texture_x_flip = false;
            self.rectangle_texture_y_flip = false;

            self.texture_window_x_mask = 0;
            self.texture_window_y_mask = 0;

            self.texture_window_x_offset = 0;
            self.texture_window_y_offset = 0;

            self.drawing_area_left = 0;
            self.drawing_area_top = 0;
            self.drawing_area_right = 0;
            self.drawing_area_bottom = 0;

            self.drawing_x_offset = 0;
            self.drawing_y_offset = 0;

            self.display_vram_x_start = 0;
            self.display_vram_y_start = 0;

            self.display_horizontal_start = 0;
            self.display_horizontal_end = 0;

            self.display_line_start = 0;
            self.display_line_end = 0;
    }

    fn gp1_reset_command_buffer(&mut self) {
        self.gp0_command.clear();
        self.gp0_command_remaining = 0;
        self.gp0_mode = Gp0Mode::Command;
    }

    fn gp1_acknowledge_irq(&mut self) {
        self.interrupt = false;
    }

    pub fn read(&self) -> u32 {
        0
    }

    fn gp1_display_enable(&mut self, value: u32) {
        self.display_disable = value & 1 != 0;
    }

    fn gp1_display_mode(&mut self, value: u32) {
        let hr1 = (value & 3) as u8;
        let hr2 = ((value >> 6) & 1) as u8;

        self.hres = HorisontalRes::from_fields(hr1, hr2);

        self.vres = match value & 0x4 != 0 {
            false => VerticalRes::Y240Lines,
            true => VerticalRes::Y480Lines,
        };

        self.vmode = match value & 0x8 != 0 {
            false => VMode::NTSC,
            true => VMode::PAL,
        };

        self.display_depth = match value & 0x10 != 0 {
            false => DisplayDepth::D24Bits,
            true => DisplayDepth::D15Bits,
        };

        self.interlaced = value & 0x20 != 0;

        if value & 0x80 != 0 {
            panic!("Unsupported display mode {:08x}", value);
        }
    }

    fn gp1_dma_direction(&mut self, value: u32) {
        self.dma_direction = match value & 3 {
            0 => DmaDirection::Off,
            1 => DmaDirection::Fifo,
            2 => DmaDirection::CpuToGpu0,
            3 => DmaDirection::VRamToCpu,
            _ => unreachable!(),
        }
    }

    fn gp1_display_vram_start(&mut self, value: u32) {
        self.display_vram_x_start = (value & 0x3fe) as u16;
        self.display_vram_y_start = ((value >> 10) & 0x1ff) as u16;
    }

    fn gp1_display_horizontal_range(&mut self, value: u32) {
        self.display_horizontal_start = (value & 0xfff) as u16;
        self.display_horizontal_end = ((value >> 12) & 0xfff) as u16;
    }

    fn gp1_display_vertical_range(&mut self, value: u32) {
        self.display_line_start = (value & 0x3ff) as u16;
        self.display_line_end = ((value >> 10) & 0x3ff) as u16;
    }
}

enum Gp0Mode {
    Command,
    ImageLoad,
}

#[derive(Clone, Copy)]
enum TextureDepth {
    T4Bit = 0,
    T8Bit = 1,
    T15Bit = 2,
}

#[derive(Clone, Copy)]
enum Field {
    Top = 1,
    Bottom = 0,
}

#[derive(Clone, Copy)]
enum DmaDirection {
    Off = 0,
    Fifo = 1,
    CpuToGpu0 = 2,
    VRamToCpu = 3,
}

#[derive(Clone, Copy)]
enum DisplayDepth {
    D15Bits = 0,
    D24Bits = 1,
}

#[derive(Clone, Copy)]
enum VMode {
    NTSC = 0,
    PAL = 1,
}

#[derive(Clone, Copy)]
enum VerticalRes {
    Y240Lines = 0,
    Y480Lines = 1,
}

#[derive(Clone, Copy)]
struct HorisontalRes(u8);

impl HorisontalRes {
    fn from_fields(hr1: u8, hr2: u8) -> HorisontalRes {
        let hr = (hr2 & 1) | ((hr1 & 3) << 1);
        HorisontalRes(hr)
    }

    fn into_status(self) -> u32 {
        let HorisontalRes(hr) = self;
        (hr as u32) << 16
    }

}

struct CommandBuffer {
    buffer: [u32; 12],
    length: u8,
}

impl CommandBuffer {
    fn new() -> CommandBuffer {
        CommandBuffer {
            buffer: [0; 12],
            length: 0,
        }
    }

    fn clear(&mut self) {
        self.length = 0;
    }

    fn push_word(&mut self, value: u32) {
        self.buffer[self.length as usize] = value;
        self.length += 1;
    }
}

impl ::std::ops::Index<usize> for CommandBuffer {
    type Output = u32;

    fn index<'a>(&'a self, index: usize) -> &'a u32 {
        if index >= self.length as usize {
            panic!("Command buffer index out of range");
        }

        &self.buffer[index]
    }
}
