#![feature(let_chains)]

use clap::Parser;
use crate::argparser::DeviceId;
use system::kernels::{Cpu, Serial};
use system::{DeviceDescription, System};

mod argparser;


fn main() {
    let emu_args: argparser::EmuArgs = argparser::EmuArgs::parse();
    
    let prog = emu_args.prog.read_all().unwrap().into_vec();
    let mut devs = vec![
        DeviceDescription::new(
            0x00,
            Cpu::new(emu_args.ram_size, &prog),
            emu_args.clock_freq,
            emu_args.verbose,
        )
    ];
    
    devs.extend(emu_args.device.into_iter().enumerate().map(|(i, dev)|
        DeviceDescription::new(
            i as u8 + 2,
            match dev {
                DeviceId::SerialPort => Serial::new()
            },
            emu_args.clock_freq.div_ceil(emu_args.devs_clocks_freq_coef),
            emu_args.verbose,
        )
    ));
    
    
    let mut system = System::new(devs, emu_args.clock_freq.div_ceil(emu_args.devs_clocks_freq_coef));
    
    if emu_args.kill_cpu {
        system.run_and_kill_cpu(None);
    } else {
        system.run(None);
    };
}
