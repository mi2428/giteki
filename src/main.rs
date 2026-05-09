mod api;
mod cli;
mod display;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;

use crate::api::{
    FileQuery, GitekiClient, GitekiInfo, GitekiItem, GitekiListResponse, decode_attachment_key,
};
use crate::cli::{ApiFormat, Cli, Commands, FileArgs, SearchArgs};
use crate::display::print_text;

// Keep the entrypoint thin: clap parsing happens in cli.rs, API I/O in api.rs,
// and terminal rendering in display.rs.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli).await
}

async fn run(cli: Cli) -> Result<()> {
    let client = GitekiClient::new()?;

    match cli.command {
        Some(Commands::File(args)) => run_file(&client, args).await,
        None => run_search(&client, cli.search).await,
    }
}

async fn run_search(client: &GitekiClient, args: SearchArgs) -> Result<()> {
    args.validate()?;

    let query = args.query();
    if args.api_format != ApiFormat::Json {
        let body = client.search_raw(&query).await?;
        print!("{body}");
        if !body.ends_with('\n') {
            println!();
        }
        return Ok(());
    }

    let response = client.search(&query).await?;
    let GitekiListResponse {
        information,
        giteki,
    } = response;

    let mut records = giteki
        .into_iter()
        .map(GitekiItem::into_info)
        .collect::<Vec<_>>();
    records.truncate(usize::from(args.limit));

    if args.json {
        let output = SearchOutput {
            last_update_date: &information.last_update_date,
            total_count: information.total_count,
            returned_count: records.len(),
            records: &records,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_text(&information, &records);
    }

    Ok(())
}

async fn run_file(client: &GitekiClient, args: FileArgs) -> Result<()> {
    args.validate()?;

    let attachment_key = decode_attachment_key(&args.attachment_key);
    let query = FileQuery {
        afk: &attachment_key,
        aft: args.attachment_type,
        afn: args.attachment_number,
    };
    let bytes = client.download_file(&query).await?;

    std::fs::write(&args.output, &bytes)
        .with_context(|| format!("failed to save attachment PDF: {}", args.output.display()))?;

    println!(
        "Saved attachment PDF: {} ({} bytes)",
        args.output.display(),
        bytes.len()
    );
    Ok(())
}

#[derive(Debug, Serialize)]
struct SearchOutput<'a> {
    last_update_date: &'a str,
    total_count: u32,
    returned_count: usize,
    records: &'a [GitekiInfo],
}
