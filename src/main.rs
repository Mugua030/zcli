use clap::Parser;
use zcli::{CmdExecutor, Opts};

// zcli csv -i input.csv -o output.json --header -d ','

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    opts.cmd.execute().await?;

    Ok(())
}
