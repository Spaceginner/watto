#![feature(let_chains)]

use clap::Parser;
use crate::argparser::DeviceId;
use system::kernels::{Cpu, Kernel, Serial};
use system::{DeviceDescription, System};

mod argparser;


fn main() {
    let emu_args: argparser::EmuArgs = argparser::EmuArgs::parse();
    
    let prog = emu_args.prog.read_all().unwrap().into_vec();
    let mut devs = vec![
        DeviceDescription::new(
            0x00,
            Box::new(Cpu::new(emu_args.ram_size, &prog)) as Box<dyn Kernel>,
            emu_args.clock_freq,
            true,
        )
    ];
    
    devs.extend(emu_args.device.into_iter().enumerate().map(|(i, dev)|
        DeviceDescription::new(
            i as u8 + 2,
            match dev {
                DeviceId::SerialPort => Box::new(Serial::new()) as Box<dyn Kernel>
            },
            emu_args.clock_freq,
            false,
        )
    ));
    
    
    let mut system = System::new(devs, emu_args.clock_freq);
    
    system.run(None);
}
