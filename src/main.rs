mod api;
mod cli;
mod display;

use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;

use crate::api::{
    FileDownload, FileQuery, GitekiClient, GitekiInfo, GitekiItem, GitekiListResponse,
    decode_attachment_key,
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
    ensure_output_dir(&args.output)?;

    let attachment_key = decode_attachment_key(&args.attachment_key);
    let query = FileQuery {
        afk: &attachment_key,
        aft: args.attachment_type,
        afn: args.attachment_number,
    };
    let download = client.download_file(&query).await?;
    let output_path = args.output.join(attachment_output_filename(&download)?);

    std::fs::write(&output_path, &download.bytes)
        .with_context(|| format!("failed to save attachment file: {}", output_path.display()))?;

    println!(
        "Saved attachment file: {} ({} bytes)",
        output_path.display(),
        download.bytes.len()
    );
    Ok(())
}

fn attachment_output_filename(download: &FileDownload) -> Result<String> {
    let giteki_number = download
        .filename
        .as_deref()
        .and_then(giteki_number_from_filename)
        .context("failed to determine giteki number from attachment response filename")?;
    let extension = attachment_extension(&download.bytes, download.filename.as_deref())?;
    Ok(format!("giteki_{giteki_number}.{extension}"))
}

fn ensure_output_dir(path: &Path) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            anyhow::bail!("--output must be a directory: {}", path.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(path)
        .with_context(|| format!("failed to create output directory: {}", path.display()))
}

fn giteki_number_from_filename(filename: &str) -> Option<String> {
    let filename = Path::new(filename).file_name()?.to_str()?;
    let stem = filename.rsplit_once('.').map_or(filename, |(stem, _)| stem);
    let number = stem.split('_').next()?.trim();
    let sanitized = sanitize_filename_component(number);
    (!sanitized.is_empty()).then_some(sanitized)
}

fn sanitize_filename_component(value: &str) -> String {
    value
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                Some(character)
            } else if character.is_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn attachment_extension(bytes: &[u8], filename: Option<&str>) -> Result<&'static str> {
    if bytes.starts_with(b"%PDF-") {
        return Ok("pdf");
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return Ok("zip");
    }

    match filename.and_then(|filename| Path::new(filename).extension()?.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("pdf") => Ok("pdf"),
        Some(extension) if extension.eq_ignore_ascii_case("zip") => Ok("zip"),
        _ => anyhow::bail!("attachment response is neither PDF nor ZIP"),
    }
}

#[derive(Debug, Serialize)]
struct SearchOutput<'a> {
    last_update_date: &'a str,
    total_count: u32,
    returned_count: usize,
    records: &'a [GitekiInfo],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_zip_output_name_from_response_filename() {
        let download = FileDownload {
            bytes: b"PK\x03\x04archive".to_vec(),
            filename: Some("022-200057_20200929.zip".to_string()),
        };

        let filename = attachment_output_filename(&download).unwrap();

        assert_eq!(filename, "giteki_022-200057.zip");
    }

    #[test]
    fn derives_pdf_output_name_from_response_filename() {
        let download = FileDownload {
            bytes: b"%PDF-1.5\n".to_vec(),
            filename: Some("022-200057_01_003.pdf".to_string()),
        };

        let filename = attachment_output_filename(&download).unwrap();

        assert_eq!(filename, "giteki_022-200057.pdf");
    }
}
