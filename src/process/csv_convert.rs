use crate::cli::OutputFormat;
use anyhow::Result;
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

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

pub fn process_csv(input: &str, output: String, format: OutputFormat) -> Result<()> {
    let mut reader = Reader::from_path(input)?;
    // to json
    let mut ret = Vec::with_capacity(128);
    //for result in reader.deserialize() {
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        //let record: Player = result?;
        let record = result?;
        //println!("{:?}", record);
        // => to an tuple by  the  the zip iterator [(head, record),...]
        // use collect => json value
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();

        ret.push(json_value);
    }

    //fs::write(output, json)?;
    let content: String = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
    };
    fs::write(output, content)?;

    Ok(())
}
