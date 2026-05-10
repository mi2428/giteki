use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::api::ListQuery;

// The CLI stays close to clap's derive model: each struct maps directly to the
// command surface, and conversion to API query parameters is kept on SearchArgs.
#[derive(Debug, Parser)]
#[command(
    name = "giteki",
    version,
    about = "Display Giteki (技適) records using MIC equipment certification API (技術基準適合証明等機器検索API)",
    args_conflicts_with_subcommands = true
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    #[command(flatten)]
    pub(crate) search: SearchArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    #[command(about = "Save an attachment file by file key (一覧詳細情報の添付ファイル取得キー)")]
    File(FileArgs),
}

#[derive(Debug, Args)]
pub(crate) struct SearchArgs {
    #[arg(
        value_name = "NUMBER",
        conflicts_with = "number",
        help = "Search by certification number (技術基準適合証明番号, 工事設計認証番号, or 届出番号)"
    )]
    number_arg: Option<String>,

    #[arg(
        short = 'n',
        long,
        value_name = "NUMBER",
        help = "Search by certification number (技術基準適合証明番号, 工事設計認証番号, or 届出番号)"
    )]
    number: Option<String>,

    #[arg(
        long,
        value_name = "NAME",
        help = "Search by applicant name (氏名又は名称), partial match"
    )]
    name: Option<String>,

    #[arg(
        short = 't',
        long,
        alias = "tn",
        value_name = "TYPE_NAME",
        help = "Search by model or type name (型式又は名称), partial match"
    )]
    type_name: Option<String>,

    #[arg(
        long,
        alias = "oc",
        value_name = "CODE",
        help = "Filter by certification body code (認証機関コード)"
    )]
    organ_code: Option<String>,

    #[arg(
        long = "from",
        value_name = "DATE",
        value_parser = parse_date,
        help = "Start date (年月日), YYYYMMDD or YYYY-MM-DD"
    )]
    from_date: Option<String>,

    #[arg(
        long = "to",
        value_name = "DATE",
        value_parser = parse_date,
        help = "End date (年月日), YYYYMMDD or YYYY-MM-DD"
    )]
    to_date: Option<String>,

    #[arg(
        long,
        alias = "rec",
        value_name = "CODE",
        help = "Filter by specified radio equipment code (特定無線設備の種別コード)"
    )]
    radio_equipment_code: Option<String>,

    #[arg(
        long,
        alias = "tec",
        value_name = "CODE",
        help = "Filter by certification type code (技術基準適合証明等の種類コード)"
    )]
    tech_code: Option<String>,

    #[arg(long, help = "Only search records with attachments (添付ファイル)")]
    attachments: bool,

    #[arg(long, help = "Only search Body SAR-supported records (BODYSAR対応)")]
    body_sar: bool,

    #[arg(
        short,
        long,
        default_value_t = 10,
        value_parser = clap::value_parser!(u16).range(1..=1000),
        help = "Maximum records to display. Fetches in API page-size units and truncates locally"
    )]
    pub(crate) limit: u16,

    #[arg(long, default_value_t = 0, help = "Result offset")]
    pub(crate) offset: u32,

    #[arg(
        long,
        default_value_t = 1,
        value_parser = clap::value_parser!(u8).range(1..=22),
        help = "API sort key"
    )]
    sort: u8,

    #[arg(
        long = "api-format",
        value_enum,
        default_value = "json",
        help = "API output format. csv/xml are printed as-is"
    )]
    pub(crate) api_format: ApiFormat,

    #[arg(long, help = "Print pretty JSON")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct FileArgs {
    #[arg(
        value_name = "AFK",
        help = "Attachment file key (添付ファイル取得キー) returned by detail-list API (一覧詳細情報取得API)"
    )]
    pub(crate) attachment_key: String,

    #[arg(
        long = "type",
        alias = "attachment-type",
        value_name = "AFT",
        value_parser = clap::value_parser!(u8).range(1..=2),
        help = "Attachment file type (添付ファイル種別). 1: 外観写真等, 2: 特性試験の結果"
    )]
    pub(crate) attachment_type: Option<u8>,

    #[arg(
        long = "number",
        alias = "attachment-number",
        value_name = "AFN",
        value_parser = clap::value_parser!(u8).range(1..=99),
        help = "Attachment file number (添付ファイル番号). Requires --type"
    )]
    pub(crate) attachment_number: Option<u8>,

    #[arg(
        short,
        long,
        value_name = "DIR",
        help = "Output directory for giteki_<number>.{zip|pdf}; created if missing"
    )]
    pub(crate) output: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ApiFormat {
    Csv,
    Json,
    Xml,
}

impl ApiFormat {
    fn as_code(self) -> u8 {
        match self {
            Self::Csv => 1,
            Self::Json => 2,
            Self::Xml => 3,
        }
    }
}

impl SearchArgs {
    pub(crate) fn validate(&self) -> Result<()> {
        if !self.has_criteria() {
            bail!("At least one search condition is required. Example: giteki --name Ubiquiti");
        }
        Ok(())
    }

    fn has_criteria(&self) -> bool {
        self.number_arg.is_some()
            || self.number.is_some()
            || self.name.is_some()
            || self.type_name.is_some()
            || self.organ_code.is_some()
            || self.from_date.is_some()
            || self.to_date.is_some()
            || self.radio_equipment_code.is_some()
            || self.tech_code.is_some()
            || self.attachments
            || self.body_sar
    }

    pub(crate) fn query(&self) -> ListQuery<'_> {
        // The public API uses short uppercase parameter names. Keep that mapping
        // centralized here so the rest of the program can use readable fields.
        ListQuery {
            sc: self.offset,
            dc: dc_for_limit(self.limit),
            of: self.api_format.as_code(),
            nam: self.name.as_deref(),
            num: self.number.as_deref().or(self.number_arg.as_deref()),
            tn: self.type_name.as_deref(),
            oc: self.organ_code.as_deref(),
            ds: self.from_date.as_deref(),
            de: self.to_date.as_deref(),
            afp: self.attachments.then_some(1),
            bs: self.body_sar.then_some(1),
            rec: self.radio_equipment_code.as_deref(),
            tec: self.tech_code.as_deref(),
            sk: self.sort,
        }
    }
}

impl FileArgs {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.attachment_number.is_some() && self.attachment_type.is_none() {
            bail!("--type is required when --number is set");
        }
        Ok(())
    }
}

fn dc_for_limit(limit: u16) -> u8 {
    // DC is the API page-size code, not the literal number of rows.
    match limit {
        1..=10 => 1,
        11..=20 => 2,
        21..=30 => 3,
        31..=50 => 4,
        51..=100 => 5,
        101..=500 => 6,
        _ => 7,
    }
}

fn parse_date(value: &str) -> std::result::Result<String, String> {
    let compact = value.replace('-', "");
    if compact.len() != 8 || !compact.chars().all(|c| c.is_ascii_digit()) {
        return Err("DATE must be YYYYMMDD or YYYY-MM-DD".to_string());
    }
    Ok(compact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_limit_to_api_count_code() {
        assert_eq!(dc_for_limit(1), 1);
        assert_eq!(dc_for_limit(10), 1);
        assert_eq!(dc_for_limit(11), 2);
        assert_eq!(dc_for_limit(50), 4);
        assert_eq!(dc_for_limit(501), 7);
    }

    #[test]
    fn parses_supported_date_formats() {
        assert_eq!(parse_date("20260509").unwrap(), "20260509");
        assert_eq!(parse_date("2026-05-09").unwrap(), "20260509");
        assert!(parse_date("2026/05/09").is_err());
    }
}
