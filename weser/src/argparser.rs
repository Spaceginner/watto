use std::fmt::{Display, Formatter};
use clap::{Args, Parser, ValueEnum};
use clio::ClioPath;

/// watto cpu emulator with full environment support
#[derive(Debug, Clone, Parser)]
#[clap(version, about, long_about = None)]
pub struct EmuArgs {
    /// cpu speed in hz
    #[arg(long = "clk", default_value_t = 200)]
    pub clock_freq: u32,
    
    #[arg(long = "ram", default_value_t = 4096)]
    /// ram size in bytes
    pub ram_size: u16,
    
    #[arg(long, short, default_value_t)]
    /// print out cpu state each tick
    pub verbose: bool,

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
