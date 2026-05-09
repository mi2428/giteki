use std::sync::LazyLock;

use regex::Regex;
use terminal_size::{Width, terminal_size};
use textwrap::Options;
use unicode_normalization::UnicodeNormalization;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::api::{GitekiInfo, GitekiInformation};

// Normalize compact API text such as "2402MHz" while preserving interval
// descriptors where the unit belongs to the interval phrase.
static NUMBER_UNIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<prefix>^|[^A-Za-z])(?P<number>\d(?:[\d.]*\d)?)(?P<unit>mW/MHz|W/MHz|GHz|MHz|kHz|mW|W|dBm)(?P<suffix>[^A-Za-z]|$)",
    )
        .expect("valid number-unit regex")
});
static INTERVAL_UNIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<number>\d(?:[\d.]*\d)?) (?P<unit>GHz|MHz|kHz)間隔")
        .expect("valid interval-unit regex")
});

pub(crate) fn print_text(information: &GitekiInformation, records: &[GitekiInfo]) {
    let width = terminal_width();
    println!(
        "API data last updated {}",
        empty_dash(&information.last_update_date)
    );

    if records.is_empty() {
        println!();
        let record = DisplayRecord {
            title: "Search result".to_string(),
            rows: vec![display_row("Result", "No matching 技適 records found.")],
        };
        let layout = table_layout_for_records(std::slice::from_ref(&record), width);
        print_table(&record, &layout);
        return;
    }

    let display_records = records
        .iter()
        .enumerate()
        .map(|(index, item)| build_display_record(index, item))
        .collect::<Vec<_>>();
    let layout = table_layout_for_records(&display_records, width);
    for record in &display_records {
        println!();
        print_table(record, &layout);
    }
}

struct DisplayRow {
    label: &'static str,
    value: String,
    preserve_spacing: bool,
}

struct DisplayRecord {
    title: String,
    rows: Vec<DisplayRow>,
}

struct TableLayout {
    table_width: usize,
    label_width: usize,
    value_width: usize,
}

fn build_display_record(index: usize, item: &GitekiInfo) -> DisplayRecord {
    let title = format!(
        "#{}  {}  {}  {}",
        index + 1,
        empty_dash(&item.number),
        empty_dash(&item.type_name),
        empty_dash(&item.name)
    );
    let mut rows = Vec::new();
    rows.extend([
        display_row("種類", item.tech_code.as_str()),
        display_row("番号", item.number.as_str()),
        display_row("年月日", item.date.as_str()),
        display_row("氏名又は名称", item.name.as_str()),
        display_row("型式又は名称", item.type_name.as_str()),
        display_row("特定無線設備の種別", item.radio_equipment_code.as_str()),
        display_row(
            "電波の型式\n周波数及び空中線電力",
            format_elec_wave(&item.elec_wave),
        )
        .preserve_spacing(),
        display_row("スプリアス規定", item.spurious_rules.as_str()),
        display_row("BODYSAR", item.body_sar.as_str()),
        display_row("周波数維持機能", item.fq_maintain_func.as_str()),
        display_row("備考", item.note.as_str()),
        display_row("認証機関名称", item.organ_name.as_str()),
    ]);
    if has_attachment(item) {
        rows.push(display_row(
            "添付ファイル名",
            item.attachment_file_name.as_str(),
        ));
        rows.push(display_row(
            "添付ファイルキー",
            item.attachment_file_key.as_str(),
        ));
        rows.push(display_row(
            "添付ファイル数",
            format!(
                "外観写真等: {} / 特性試験の結果: {}",
                empty_dash(&item.attachment_file_cnt_for_cd_1),
                empty_dash(&item.attachment_file_cnt_for_cd_2)
            ),
        ));
    }
    rows.retain(|row| !row.value.trim().is_empty());
    DisplayRecord { title, rows }
}

fn display_row(label: &'static str, value: impl Into<String>) -> DisplayRow {
    DisplayRow {
        label,
        value: value.into(),
        preserve_spacing: false,
    }
}

impl DisplayRow {
    fn preserve_spacing(mut self) -> Self {
        self.preserve_spacing = true;
        self
    }
}

fn table_layout_for_records(records: &[DisplayRecord], terminal_width: usize) -> TableLayout {
    let max_table_width = terminal_width.clamp(60, 140);
    let label_width = records
        .iter()
        .flat_map(|record| record.rows.iter())
        .map(|row| max_display_line_width(row.label))
        .max()
        .unwrap_or(8)
        .clamp(8, 28);
    let column_gap = 3;
    let max_value_width = max_table_width
        .saturating_sub(label_width + column_gap)
        .max(20);
    let content_width = records
        .iter()
        .flat_map(|record| record.rows.iter())
        .flat_map(|row| wrap_row_value(row, max_value_width))
        .map(|line| display_width(&line))
        .max()
        .unwrap_or(0);
    let title_width = records
        .iter()
        .flat_map(|record| wrap_value(&record.title, max_table_width))
        .map(|line| display_width(&line))
        .max()
        .unwrap_or(0);
    let table_width = (label_width + column_gap + content_width)
        .max(title_width)
        .clamp(20, max_table_width);
    let value_width = table_width.saturating_sub(label_width + column_gap).max(20);

    TableLayout {
        table_width,
        label_width,
        value_width,
    }
}

fn print_table(record: &DisplayRecord, layout: &TableLayout) {
    let prepared_rows = record
        .rows
        .iter()
        .map(|row| {
            let wrapped_value = wrap_row_value(row, layout.value_width);
            let wrapped_label = wrap_value(row.label, layout.label_width);
            PreparedRow {
                wrapped_label,
                wrapped_value,
            }
        })
        .collect::<Vec<_>>();

    print_top_border(&record.title, layout.table_width);
    print_rule(layout.table_width);

    for row in &prepared_rows {
        let row_height = row.wrapped_label.len().max(row.wrapped_value.len());
        for line_index in 0..row_height {
            let label = row.wrapped_label.get(line_index).map_or("", String::as_str);
            let value = row.wrapped_value.get(line_index).map_or("", String::as_str);
            if label.is_empty() && value.is_empty() {
                continue;
            }
            if value.is_empty() {
                println!("{label}");
            } else {
                println!(
                    "{}{}{}",
                    pad_right(label, layout.label_width),
                    " ".repeat(3),
                    value
                );
            }
        }
    }

    print_rule(layout.table_width);
}

struct PreparedRow {
    wrapped_label: Vec<String>,
    wrapped_value: Vec<String>,
}

fn print_top_border(title: &str, table_width: usize) {
    for line in wrap_value(title, table_width) {
        println!("{line}");
    }
}

fn print_rule(width: usize) {
    println!("{}", "─".repeat(width));
}

fn wrap_value(value: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let normalized = normalize_display_text(value);
    for raw_line in normalized.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let options = if line.starts_with("・ ") {
            Options::new(width)
                .break_words(true)
                .subsequent_indent("  ")
        } else {
            Options::new(width).break_words(true)
        };
        lines.extend(
            textwrap::wrap(line, options)
                .into_iter()
                .map(|line| line.into_owned()),
        );
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn wrap_row_value(row: &DisplayRow, width: usize) -> Vec<String> {
    // Radio wave rows are pre-aligned with spaces, so they need a wrapping path
    // that does not collapse internal spacing.
    if row.preserve_spacing {
        wrap_preserving_spaces(&row.value, width)
    } else {
        wrap_value(&row.value, width)
    }
}

fn wrap_preserving_spaces(value: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let normalized = normalize_display_text(value);
    for raw_line in normalized.lines() {
        let mut remaining = raw_line.trim_end().to_string();
        if remaining.is_empty() {
            lines.push(String::new());
            continue;
        }

        while display_width(&remaining) > width {
            let split_at = split_at_width(&remaining, width).unwrap_or(remaining.len());
            let (head, tail) = remaining.split_at(split_at);
            lines.push(head.trim_end().to_string());
            let tail = tail.trim_start();
            remaining = if raw_line.starts_with("・ ") && !tail.is_empty() {
                format!("  {tail}")
            } else {
                tail.to_string()
            };
            if remaining.is_empty() {
                break;
            }
        }

        if !remaining.is_empty() {
            lines.push(remaining);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn split_at_width(value: &str, width: usize) -> Option<usize> {
    let mut current_width = 0;
    let mut fallback = None;
    for (byte_index, character) in value.char_indices() {
        let character_width = character.width().unwrap_or(0);
        if current_width + character_width > width {
            return fallback.or(Some(byte_index));
        }
        if character.is_whitespace() {
            fallback = Some(byte_index);
        }
        current_width += character_width;
    }
    None
}

fn pad_right(value: &str, width: usize) -> String {
    let padding = width.saturating_sub(display_width(value));
    format!("{value}{}", " ".repeat(padding))
}

fn pad_left(value: &str, width: usize) -> String {
    let padding = width.saturating_sub(display_width(value));
    format!("{}{value}", " ".repeat(padding))
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

fn max_display_line_width(value: &str) -> usize {
    normalize_display_text(value)
        .lines()
        .map(display_width)
        .max()
        .unwrap_or(0)
}

fn format_elec_wave(value: &str) -> String {
    let rows = elec_wave_rows(value);
    // Align each row as: bullet, radio/frequency text, power value, power unit.
    // This keeps the "mW/MHz" column stable even when the value widths differ.
    let prefix_column = rows
        .iter()
        .filter_map(|row| match row {
            ElecWaveRow::WithPower { prefix, .. } => Some(display_width(prefix)),
            ElecWaveRow::Plain(_) => None,
        })
        .max()
        .unwrap_or(0);
    let power_value_column = rows
        .iter()
        .filter_map(|row| match row {
            ElecWaveRow::WithPower { power_value, .. } => Some(display_width(power_value)),
            ElecWaveRow::Plain(_) => None,
        })
        .max()
        .unwrap_or(0);

    rows.into_iter()
        .map(|row| format_elec_wave_row(row, prefix_column, power_value_column))
        .collect::<Vec<_>>()
        .join("\n")
}

fn elec_wave_rows(value: &str) -> Vec<ElecWaveRow> {
    normalize_display_text(value)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(split_elec_wave_power)
        .collect::<Vec<_>>()
}

enum ElecWaveRow {
    WithPower {
        prefix: String,
        power_value: String,
        power_unit: String,
    },
    Plain(String),
}

fn split_elec_wave_power(line: &str) -> ElecWaveRow {
    // The API returns dense plain text. Split only the trailing power component
    // and leave the radio type/frequency side as readable text.
    let tokens = add_space_between_number_and_unit(&add_space_after_commas(line))
        .split_whitespace()
        .collect::<Vec<_>>()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    if tokens.len() >= 3 {
        let number = &tokens[tokens.len() - 2];
        let unit = &tokens[tokens.len() - 1];
        if is_power_number(number) && is_power_unit(unit) {
            return ElecWaveRow::WithPower {
                prefix: tokens[..tokens.len() - 2].join(" "),
                power_value: number.to_string(),
                power_unit: unit.to_string(),
            };
        }
    }

    if let Some((last, prefix)) = tokens.split_last() {
        let prefix = prefix.join(" ");
        if let Some((power_value, power_unit)) = split_compact_power_token(last)
            && !prefix.is_empty()
        {
            ElecWaveRow::WithPower {
                prefix,
                power_value,
                power_unit,
            }
        } else {
            ElecWaveRow::Plain(tokens.join(" "))
        }
    } else {
        ElecWaveRow::Plain(String::new())
    }
}

fn format_elec_wave_row(
    row: ElecWaveRow,
    prefix_column: usize,
    power_value_column: usize,
) -> String {
    match row {
        ElecWaveRow::WithPower {
            prefix,
            power_value,
            power_unit,
        } => {
            let prefix_padding = prefix_column.saturating_sub(display_width(&prefix));
            format!(
                "・ {prefix}{}  {} {power_unit}",
                " ".repeat(prefix_padding),
                pad_left(&power_value, power_value_column)
            )
        }
        ElecWaveRow::Plain(line) => format!("・ {line}"),
    }
}

fn split_compact_power_token(value: &str) -> Option<(String, String)> {
    ["mW/MHz", "W/MHz", "dBm", "mW", "W"]
        .into_iter()
        .find_map(|unit| {
            value.strip_suffix(unit).and_then(|number| {
                if is_power_number(number) {
                    Some((number.to_string(), unit.to_string()))
                } else {
                    None
                }
            })
        })
}

fn is_power_number(value: &str) -> bool {
    value.chars().any(|character| character.is_ascii_digit())
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, '.' | '~' | '-'))
}

fn is_power_unit(value: &str) -> bool {
    matches!(value, "mW/MHz" | "W/MHz" | "mW" | "W" | "dBm")
}

fn add_space_after_commas(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        output.push(character);
        if character == ','
            && chars
                .peek()
                .is_some_and(|next| !next.is_whitespace() && *next != ',')
        {
            output.push(' ');
        }
    }
    output
}

fn add_space_between_number_and_unit(value: &str) -> String {
    let mut current = value.to_string();
    for _ in 0..4 {
        let next = NUMBER_UNIT_RE
            .replace_all(&current, "$prefix$number $unit$suffix")
            .into_owned();
        if next == current {
            return remove_space_before_interval_unit(&current);
        }
        current = next;
    }
    remove_space_before_interval_unit(&current)
}

fn remove_space_before_interval_unit(value: &str) -> String {
    INTERVAL_UNIT_RE
        .replace_all(value, "$number$unit間隔")
        .into_owned()
}

fn normalize_display_text(value: &str) -> String {
    value
        .replace("\\r\\n", "\n")
        .replace("\\n", "\n")
        .replace("\\r", "\n")
        .nfkc()
        .collect()
}

fn has_attachment(item: &GitekiInfo) -> bool {
    !item.attachment_file_name.trim().is_empty() || !item.attachment_file_key.trim().is_empty()
}

fn empty_dash(value: &str) -> &str {
    if value.trim().is_empty() { "-" } else { value }
}

fn terminal_width() -> usize {
    terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .unwrap_or(100)
        .clamp(60, 160)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_fullwidth_display_text() {
        assert_eq!(
            normalize_display_text("Ｇｏｏｇｌｅ　ＬＬＣ\\n５Ｍ00　2412～2472ＭＨｚ"),
            "Google LLC\n5M00 2412~2472MHz"
        );
    }

    #[test]
    fn formats_elec_wave_for_display() {
        assert_eq!(
            format_elec_wave("Ｄ１Ｄ，Ｇ１Ｄ　2412～2472ＭＨz\\n832.5ＭＨz，837.5ＭＨz"),
            "・ D1D, G1D 2412~2472 MHz\n・ 832.5 MHz, 837.5 MHz"
        );
    }

    #[test]
    fn inserts_spaces_between_numbers_and_units() {
        assert_eq!(
            add_space_between_number_and_unit("100~200MHz(20MHz間隔8波) 4.28549mW/MHz 23W 24dBm"),
            "100~200 MHz(20MHz間隔8波) 4.28549 mW/MHz 23 W 24 dBm"
        );
        assert_eq!(
            add_space_between_number_and_unit("2412~2472MHz(5MHz間隔13波)"),
            "2412~2472 MHz(5MHz間隔13波)"
        );
        assert_eq!(add_space_between_number_and_unit("G1X, G7W"), "G1X, G7W");
    }

    #[test]
    fn aligns_power_unit_column_in_elec_wave_rows() {
        let output = format_elec_wave(
            "G1D 5180~5240MHz(20MHz間隔4波) 5.61048mW/MHz\\n\
             G1D 5510~5710MHz(40MHz間隔6波) 4.9204mW/MHz\\n\
             G1D 5.25GHz 0.5358mW/MHz",
        );
        let unit_columns = output
            .lines()
            .map(|line| {
                let byte_index = line.find("mW/MHz").expect("unit exists");
                display_width(&line[..byte_index])
            })
            .collect::<Vec<_>>();

        assert!(unit_columns.iter().all(|column| *column == unit_columns[0]));
        assert!(output.contains("5180~5240 MHz"));
        assert!(output.contains("20MHz間隔"));
    }
}
