use std::env::args;

mod bios;
mod interconnect;
mod cpu;
mod instruction;

use bios::*;
use interconnect::*;
use cpu::*;

fn main() {
    let bios_file = args().nth(1).unwrap();

    let bios = Bios::new(&bios_file).unwrap();

    let inter = Interconnect::new(bios);

    let mut cpu = Cpu::new(inter);

    loop {
        cpu.run_next_instruction();
    }
}
