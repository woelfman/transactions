//! Payment processor.
//!
//! Usage: transactions <FILE>
use csv::Writer;
use eyre::Result;

use transactions::Record;
#[cfg(feature = "clap")]
mod cli;

fn main() -> Result<()> {
    #[cfg(feature = "env_logger")]
    env_logger::init();

    cfg_if::cfg_if! {
        if #[cfg(feature = "clap")] {
            let args = cli::parse_args();
            let file = args.file;
        } else {
            let args: Vec<String> = std::env::args().collect();
            let file = &args[1];
        }
    }

    let records = Record::from_path(file)?;

    let clients = Record::process(&records)?;

    let mut wtr = Writer::from_writer(vec![]);
    clients
        .values()
        .try_for_each(|client| wtr.serialize(client))?;

    print!("{}", String::from_utf8(wtr.into_inner()?)?);

    Ok(())
}
