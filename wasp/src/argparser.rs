use std::fmt::{Display, Formatter};
use clap::{Parser, ValueEnum};
use clio::ClioPath;


/// an assembler for watto cpu
#[derive(Clone, Debug, Parser)]
pub struct AsmArgs {
    /// perform a dry run (no writing done)
    #[arg(long)]
    pub dry: bool,
    
    /// path to the source file
    #[arg(long, short, value_parser = clap::value_parser!(ClioPath).exists().is_file(), default_value = "-")]
    pub source: ClioPath,
    
    /// path to the output binary
    #[arg(long, short, value_parser = clap::value_parser!(ClioPath), default_value = "-")]
    pub out: ClioPath,
    
    /// output format
    #[arg(long, default_value_t)]
    pub format: Format
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum Format {
    Words,
    Elements,
    Instructs,
    #[default]
    Binary,
}


impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Words => write!(f, "words"),
            Self::Elements => write!(f, "elements"),
            Self::Instructs => write!(f, "instructs"),
            Self::Binary => write!(f, "binary"),
        }
    }
}
