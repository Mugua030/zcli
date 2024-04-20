use anyhow::Result;
use clap::Parser;
use csv::Reader;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

// zcli csv -i input.csv -o output.json --header -d ','

#[derive(Debug, Parser)]
#[command(name = "zcli", version, author, about, long_about = None)]
struct Opts {
    #[command(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    #[command(name = "csv", about = "show csv")]
    Csv(CsvOpts),
}

#[derive(Debug, Parser)]
struct CsvOpts {
    #[arg(short, long, value_parser = verify_input_file)]
    input: String,

    // default_value => "output.json".into()
    // default_value_t =>
    #[arg(short, long, default_value = "output.json")]
    output: String,
    #[arg(short, long, default_value_t = ',')]
    delimiter: char,
    #[arg(long, default_value_t = true)]
    header: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let mut reader = Reader::from_path(opts.input)?;
            // to json
            let mut ret = Vec::with_capacity(128);
            for result in reader.deserialize() {
                let record: Player = result?;
                //println!("{:?}", record);
                ret.push(record)
            }

            // for json
            let json = serde_json::to_string_pretty(&ret)?;
            fs::write(opts.output, json)?;
        }
    }
    Ok(())
}

fn verify_input_file(filename: &str) -> Result<String, &'static str> {
    if Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("File does not exist")
    }
}
