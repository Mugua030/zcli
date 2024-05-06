use super::{verify_file, verify_path, Base64Format};
use crate::{
    get_content, get_reader, process_decode_data, process_encode_data, process_text_decypt,
    process_text_encypt, process_text_key_generate, process_text_sign, process_text_verify,
    CmdExecutor,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use clap::Parser;
use std::{fmt, io::Cursor, path::PathBuf, str::FromStr};
use tokio::fs;

#[derive(Debug, Parser)]
pub enum TextSubCommand {
    #[command(about = "Sign a text")]
    Sign(TextSignOpts),
    #[command(about = "verify a signature")]
    Verify(TextVeriryOpts),
    #[command(about = "generate a random blake3 or 15519 key pair")]
    Generate(KeyGenerateOpts),
    #[command(about = "encrypt text")]
    Encrypt(TextEncryptOpts),
    #[command(about = "decrypt content")]
    Decrypt(TextDecryptOpts),
}

#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
    #[arg(long, default_value = "blake3", value_parser = parse_text_sign_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextVeriryOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
    #[arg(short, long, value_parser = verify_file)]
    pub key: String,
    #[arg(long)]
    pub sig: String,
    #[arg(long, default_value = "blake3", value_parser = parse_text_sign_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct KeyGenerateOpts {
    #[arg(long, default_value = "blake3", value_parser = parse_text_sign_format)]
    pub format: TextSignFormat,
    #[arg(short, long, value_parser = verify_path)]
    pub output_path: PathBuf,
}

#[derive(Debug, Parser)]
pub struct TextEncryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
}

#[derive(Debug, Parser)]
pub struct TextDecryptOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
}

fn parse_text_sign_format(format: &str) -> Result<TextSignFormat, anyhow::Error> {
    format.parse()
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl From<TextSignFormat> for &'static str {
    fn from(format: TextSignFormat) -> Self {
        match format {
            TextSignFormat::Blake3 => "blake3",
            TextSignFormat::Ed25519 => "ed25519",
        }
    }
}

impl fmt::Display for TextSignFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}

// CmdExecutor
impl CmdExecutor for TextSubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            TextSubCommand::Sign(opts) => opts.execute().await,
            TextSubCommand::Verify(opts) => opts.execute().await,
            TextSubCommand::Generate(opts) => opts.execute().await,
            TextSubCommand::Encrypt(opts) => opts.execute().await,
            TextSubCommand::Decrypt(opts) => opts.execute().await,
        }
    }
}

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = get_reader(&self.input)?;
        let key = get_content(&self.key)?;
        let sig = process_text_sign(&mut reader, &key, self.format)?;

        let encoded = URL_SAFE_NO_PAD.encode(sig);
        println!("{}", encoded);

        Ok(())
    }
}

impl CmdExecutor for TextVeriryOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = get_reader(&self.input)?;
        let key = get_content(&self.key)?;
        let decoded = URL_SAFE_NO_PAD.decode(&self.sig)?;
        let verified = process_text_verify(&mut reader, &key, &decoded, self.format)?;
        if verified {
            println!("siganature verified");
        } else {
            println!("Signature not verified");
        }

        Ok(())
    }
}

impl CmdExecutor for KeyGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mkey = process_text_key_generate(self.format)?;
        for (k, v) in mkey {
            fs::write(self.output_path.join(k), v).await?;
        }

        Ok(())
    }
}

impl CmdExecutor for TextEncryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = get_reader(&self.input)?;
        let ret = process_text_encypt(&mut reader)?;

        // use base64 to mix
        let data = process_encode_data(ret, Base64Format::UrlSafe)?;

        println!("{}", data);

        Ok(())
    }
}

impl CmdExecutor for TextDecryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let result = match process_decode_data(&self.input, Base64Format::UrlSafe) {
            Ok(data) => data,
            Err(err) => {
                return Err(anyhow::Error::msg(format!(
                    "process_decode_data error: {}",
                    err
                )))
            }
        };
        let buff: Vec<u8> = result.into_bytes();
        let mut cursor = Cursor::new(buff);

        let ret = process_text_decypt(&mut cursor)?;

        println!("{}", String::from_utf8_lossy(&ret));

        Ok(())
    }
}
