use clap::Parser;
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
}
