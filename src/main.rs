use clap::Parser;
use zcli::{
    process_csv, process_decode, process_encode, process_genpwd, Base64SubCommand, Opts, SubCommand,
};

// zcli csv -i input.csv -o output.json --header -d ','

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                //"output.json".into()
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }
        SubCommand::GenPWD(opts) => {
            println!("{:?}", opts);
            process_genpwd(
                opts.length,
                opts.no_upper_case,
                opts.lower_case,
                opts.numbers,
                opts.symbols,
            )?;
        }
        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opts) => {
                //println!("encode: {:?}", opts);
                process_encode(&opts.input, opts.format)?;
            }
            Base64SubCommand::Decode(opts) => {
                //println!("decode: {:?}", opts);
                process_decode(&opts.input, opts.format)?;
            }
        },
    }
    Ok(())
}
