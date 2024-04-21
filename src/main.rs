use clap::Parser;
use zcli::{process_csv, Opts, SubCommand};

// zcli csv -i input.csv -o output.json --header -d ','

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => process_csv(&opts.input, &opts.output)?,
    }
    Ok(())
}
