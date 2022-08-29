mod csv_record;

use clap::Parser;
use csv::{Error, Trim};
use csv_record::TransactionRecord;
use ledger_rs::Ledger;
use std::{collections::BTreeMap, io};
use tracing::{debug, warn, Level};

#[derive(Debug, Parser)]
#[clap(author, version, about = "CSV transaction processor")]
struct Cli {
    /// Input file
    #[clap(value_parser)]
    path: String,
    /// Debug mode
    #[clap(short, long, value_parser, default_value = "false")]
    debug: bool,
    /// Sort output accounts on ClientId
    #[clap(short, long, value_parser, default_value = "false")]
    sort: bool,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    if cli.debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    let ledger = process_file(&cli.path)?;

    let accounts = ledger.get_accounts();
    let mut wtr = csv::Writer::from_writer(io::stdout());
    match cli.sort {
        true => {
            let mut btree = BTreeMap::new();
            for account in accounts {
                btree.insert(account.id, account);
            }
            for account in btree.values() {
                wtr.serialize(account)?;
            }
            wtr.flush()?;
        }
        false => {
            for account in accounts {
                wtr.serialize(account)?;
            }
            wtr.flush()?;
        }
    }

    Ok(())
}

fn process_file(path: &str) -> Result<Ledger, Error> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_path(&path)?;

    let mut ledger = Ledger::new();

    while let Some(result) = rdr.deserialize::<TransactionRecord>().next() {
        match result {
            Ok(record) => match record.try_into() {
                Ok(tx) => {
                    debug!("Attempting to process {tx:#?}");
                    if let Err(e) = ledger.process_tx(tx) {
                        warn!("{e}")
                    }
                }
                Err(e) => warn!("{e}"),
            },
            Err(e) => warn!("{e}"),
        }
    }
    Ok(ledger)
}
