use std::fs;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use clap::Parser;
use zcli::{
    get_content, get_reader, process_csv, process_decode, process_encode, process_genpwd,
    process_text_key_generate, process_text_sign, process_text_verify, Base64SubCommand, Opts,
    SubCommand, TextSubCommand,
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
            let ret = process_genpwd(
                opts.length,
                opts.no_upper_case,
                opts.lower_case,
                opts.numbers,
                opts.symbols,
            )?;
            println!("{}=>len:{}", ret, ret.len());
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
        SubCommand::Text(cmd) => match cmd {
            // zlic text sign --key fixtures/blake3.txt
            TextSubCommand::Sign(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let key = get_content(&opts.key)?;
                let sig = process_text_sign(&mut reader, &key, opts.format)?;

                let encoded = URL_SAFE_NO_PAD.encode(sig);
                println!("{}", encoded);
            }
            TextSubCommand::Verify(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let key = get_content(&opts.key)?;
                let decoded = URL_SAFE_NO_PAD.decode(&opts.sig)?;
                let verified = process_text_verify(&mut reader, &key, &decoded, opts.format)?;
                if verified {
                    println!("siganature verified");
                } else {
                    println!("Signature not verified");
                }
            }
            TextSubCommand::Generate(opts) => {
                let mkey = process_text_key_generate(opts.format)?;
                for (k, v) in mkey {
                    fs::write(opts.output_path.join(k), v)?;
                }
            }
        },
    }
    Ok(())
}
