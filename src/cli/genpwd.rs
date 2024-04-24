use clap::Parser;

#[derive(Debug, Parser)]
pub struct GenPWDOpts {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,
    #[arg(long)]
    pub no_upper_case: bool,
    #[arg(long, default_value_t = true)]
    pub lower_case: bool,
    #[arg(long, default_value_t = true)]
    pub numbers: bool,
    #[arg(long, default_value_t = true)]
    pub symbols: bool,
}
