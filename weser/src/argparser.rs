use std::fmt::{Display, Formatter};
use clap::{Args, Parser, ValueEnum};
use clio::ClioPath;

/// watto cpu emulator with full environment support
#[derive(Debug, Clone, Parser)]
#[clap(version, about, long_about = None)]
pub struct EmuArgs {
    /// cpu speed in hz
    #[arg(long = "clk", default_value_t = 1000, value_parser = clap::value_parser!(u32).range(1..))]
    pub clock_freq: u32,

    /// a coefficient for devices clocks frequency (= cpu ÷ coef)
    #[arg(long = "bus-coef", default_value_t = 10)]
    pub devs_clocks_freq_coef: u32,
    
    /// a coefficient for bus clock frequency (= cpu ÷ coef)
    #[arg(long = "devs-coef", default_value_t = 15)]
    pub bus_clock_freq_coef: u32,
    
    /// ram size in bytes
    #[arg(long = "ram", default_value_t = 4096)]
    pub ram_size: u16,
    
    /// print out cpu state each tick
    #[arg(long, short, default_value_t)]
    pub verbose: bool,
    
    /// make the emulator real fast at the cost of 100%'ing cpu core
    #[arg(long, default_value_t)]
    pub kill_cpu: bool,
    
    /// path to the program
    #[arg(value_parser = clap::value_parser!(ClioPath).exists().is_file())]
    pub prog: ClioPath,

    /// list of devices to attach
    pub device: Vec<DeviceId>
}


#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DeviceId {
    // #[value(name = "clock")]
    // RealtimeClock,
    #[value(name = "serial")]
    SerialPort
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // Self::RealtimeClock => write!(f, "clock"),
            Self::SerialPort => write!(f, "serial")
        }
    }
}


// #[derive(Debug, Clone, Args)]
// pub struct Device {
//     /// device name
//     name: DeviceId,
//     /// device's clock speed in hz
//     #[arg(default_value_t = 100)]
//     clock_speed: u16,
//     /// port on the bus (5-bit value, 0th and 1st ports are reserved for cpu and smu)
//     #[arg(value_parser = clap::value_parser!(u16).range(2..=32))]
//     address: Option<u16>,
// }
// 
// impl Display for Device {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}/{}hz@{}", self.name, self.clock_speed, self.address.map(|addr| format!("0x{addr:0>2x}")).unwrap_or(String::from("0x??")))
//     }
// }
