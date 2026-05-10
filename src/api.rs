use std::time::Duration;

use anyhow::{Context, Result, bail};
use percent_encoding::percent_decode_str;
use reqwest::header::{ACCEPT, CONTENT_DISPOSITION, HeaderMap, HeaderValue, REFERER};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

// The public API endpoint expects requests that look like the browser-backed
// search page. HTTP/1 plus Referer/User-Agent avoids CDN-side rejections.
const API_LIST_ENDPOINT: &str = "https://www.tele.soumu.go.jp/giteki/list";
const API_FILE_ENDPOINT: &str = "https://www.tele.soumu.go.jp/giteki/file";
const API_WEB_DOC: &str = "https://www.tele.soumu.go.jp/j/sys/equ/tech/webapi/";
const API_SEARCH_PAGE: &str = "https://www.tele.soumu.go.jp/giteki/SearchServlet?pageID=js01";

pub(crate) struct GitekiClient {
    http: reqwest::Client,
}

impl GitekiClient {
    pub(crate) fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json,text/plain,*/*"),
        );
        headers.insert(REFERER, HeaderValue::from_static(API_SEARCH_PAGE));

        let http = reqwest::Client::builder()
            .http1_only()
            .default_headers(headers)
            .user_agent(format!(
                "Mozilla/5.0 AppleWebKit/537.36 Chrome/124.0 Safari/537.36 giteki/{version} ({doc})",
                version = env!("CARGO_PKG_VERSION"),
                doc = API_WEB_DOC,
            ))
            .timeout(Duration::from_secs(20))
            .build()
            .context("failed to initialize HTTP client")?;

        Ok(Self { http })
    }

    pub(crate) async fn search(&self, query: &ListQuery<'_>) -> Result<GitekiListResponse> {
        let body = self.search_raw(query).await?;
        serde_json::from_str(&body).with_context(|| {
            format!(
                "failed to parse 総務省API JSON: {}",
                truncate_for_error(&body)
            )
        })
    }

    pub(crate) async fn search_raw(&self, query: &ListQuery<'_>) -> Result<String> {
        let response = self
            .http
            .get(API_LIST_ENDPOINT)
            .query(query)
            .send()
            .await
            .context("request to 総務省API failed")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read 総務省API response body")?;

        if !status.is_success() {
            bail!(
                "総務省API returned HTTP {status}: {body}",
                body = truncate_for_error(&body)
            );
        }

        Ok(body)
    }

    pub(crate) async fn download_file(&self, query: &FileQuery<'_>) -> Result<FileDownload> {
        let response = self
            .http
            .get(API_FILE_ENDPOINT)
            .header(
                ACCEPT,
                "application/pdf,application/zip,application/octet-stream,*/*",
            )
            .query(query)
            .send()
            .await
            .context("request to 総務省API attachment endpoint failed")?;

        let status = response.status();
        let filename = response
            .headers()
            .get(CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok())
            .and_then(content_disposition_filename);
        let bytes = response
            .bytes()
            .await
            .context("failed to read 総務省API attachment response")?;

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            bail!(
                "総務省API returned HTTP {status}: {body}",
                body = truncate_for_error(&body)
            );
        }

        Ok(FileDownload {
            bytes: bytes.to_vec(),
            filename,
        })
    }
}

#[derive(Debug)]
pub(crate) struct FileDownload {
    pub(crate) bytes: Vec<u8>,
    pub(crate) filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ListQuery<'a> {
    #[serde(rename = "SC")]
    pub(crate) sc: u32,
    #[serde(rename = "DC")]
    pub(crate) dc: u8,
    #[serde(rename = "OF")]
    pub(crate) of: u8,
    #[serde(rename = "NAM", skip_serializing_if = "Option::is_none")]
    pub(crate) nam: Option<&'a str>,
    #[serde(rename = "NUM", skip_serializing_if = "Option::is_none")]
    pub(crate) num: Option<&'a str>,
    #[serde(rename = "TN", skip_serializing_if = "Option::is_none")]
    pub(crate) tn: Option<&'a str>,
    #[serde(rename = "OC", skip_serializing_if = "Option::is_none")]
    pub(crate) oc: Option<&'a str>,
    #[serde(rename = "DS", skip_serializing_if = "Option::is_none")]
    pub(crate) ds: Option<&'a str>,
    #[serde(rename = "DE", skip_serializing_if = "Option::is_none")]
    pub(crate) de: Option<&'a str>,
    #[serde(rename = "AFP", skip_serializing_if = "Option::is_none")]
    pub(crate) afp: Option<u8>,
    #[serde(rename = "BS", skip_serializing_if = "Option::is_none")]
    pub(crate) bs: Option<u8>,
    #[serde(rename = "REC", skip_serializing_if = "Option::is_none")]
    pub(crate) rec: Option<&'a str>,
    #[serde(rename = "TEC", skip_serializing_if = "Option::is_none")]
    pub(crate) tec: Option<&'a str>,
    #[serde(rename = "SK")]
    pub(crate) sk: u8,
}

#[derive(Debug, Serialize)]
pub(crate) struct FileQuery<'a> {
    #[serde(rename = "AFK")]
    pub(crate) afk: &'a str,
    #[serde(rename = "AFT", skip_serializing_if = "Option::is_none")]
    pub(crate) aft: Option<u8>,
    #[serde(rename = "AFN", skip_serializing_if = "Option::is_none")]
    pub(crate) afn: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitekiListResponse {
    #[serde(rename = "gitekiInformation")]
    pub(crate) information: GitekiInformation,
    #[serde(default)]
    pub(crate) giteki: Vec<GitekiItem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitekiInformation {
    #[serde(default, rename = "lastUpdateDate")]
    pub(crate) last_update_date: String,
    #[serde(default, rename = "totalCount", deserialize_with = "deserialize_u32")]
    pub(crate) total_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum GitekiItem {
    // The API has returned both wrapped and direct record objects in practice,
    // so accept either shape and normalize with into_info().
    Wrapped {
        #[serde(rename = "gitekiInfo")]
        giteki_info: GitekiInfo,
    },
    Direct(GitekiInfo),
}

impl GitekiItem {
    pub(crate) fn into_info(self) -> GitekiInfo {
        match self {
            Self::Wrapped { giteki_info } => giteki_info,
            Self::Direct(giteki_info) => giteki_info,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct GitekiInfo {
    #[serde(default, deserialize_with = "deserialize_u32")]
    pub(crate) no: u32,
    #[serde(default, rename = "techCode")]
    pub(crate) tech_code: String,
    #[serde(default)]
    pub(crate) number: String,
    #[serde(default)]
    pub(crate) date: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default, rename = "radioEquipmentCode")]
    pub(crate) radio_equipment_code: String,
    #[serde(default, rename = "typeName")]
    pub(crate) type_name: String,
    #[serde(default, rename = "elecWave")]
    pub(crate) elec_wave: String,
    #[serde(default, rename = "spuriousRules")]
    pub(crate) spurious_rules: String,
    #[serde(default, rename = "bodySar")]
    pub(crate) body_sar: String,
    #[serde(default, rename = "fqMaintainFunc")]
    pub(crate) fq_maintain_func: String,
    #[serde(default)]
    pub(crate) note: String,
    #[serde(default, rename = "organName")]
    pub(crate) organ_name: String,
    #[serde(default, rename = "attachmentFileName")]
    pub(crate) attachment_file_name: String,
    #[serde(default, rename = "attachmentFileKey")]
    pub(crate) attachment_file_key: String,
    #[serde(default, rename = "attachmentFileCntForCd1")]
    pub(crate) attachment_file_cnt_for_cd_1: String,
    #[serde(default, rename = "attachmentFileCntForCd2")]
    pub(crate) attachment_file_cnt_for_cd_2: String,
}

pub(crate) fn decode_attachment_key(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

fn content_disposition_filename(value: &str) -> Option<String> {
    let mut fallback = None;

    for part in value.split(';').map(str::trim) {
        if let Some(raw) = part.strip_prefix("filename*=") {
            if let Some(filename) = decode_rfc5987_filename(raw) {
                return Some(filename);
            }
        } else if let Some(raw) = part.strip_prefix("filename=") {
            fallback = Some(unquote_header_value(raw));
        }
    }

    fallback
}

fn decode_rfc5987_filename(value: &str) -> Option<String> {
    let value = value.trim_matches('"');
    let encoded = value.strip_prefix("UTF-8''").unwrap_or(value);
    percent_decode_str(encoded)
        .decode_utf8()
        .ok()
        .map(|decoded| decoded.into_owned())
}

fn unquote_header_value(value: &str) -> String {
    let value = value.trim();
    let Some(quoted) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return value.to_string();
    };

    let mut output = String::with_capacity(quoted.len());
    let mut chars = quoted.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            if let Some(escaped) = chars.next() {
                output.push(escaped);
            }
        } else {
            output.push(character);
        }
    }
    output
}

fn deserialize_u32<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    // Numeric API fields are often JSON strings, and blank strings should be
    // treated like missing values.
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|raw| u32::try_from(raw).ok())
            .ok_or_else(|| serde::de::Error::custom("number is out of u32 range")),
        Value::String(text) => {
            if text.trim().is_empty() {
                Ok(0)
            } else {
                text.parse::<u32>()
                    .map_err(|error| serde::de::Error::custom(error.to_string()))
            }
        }
        Value::Null => Ok(0),
        other => Err(serde::de::Error::custom(format!(
            "expected number or string: {other}"
        ))),
    }
}

fn truncate_for_error(body: &str) -> String {
    const LIMIT: usize = 300;
    let mut text = body.replace('\n', " ");
    if text.len() > LIMIT {
        text.truncate(LIMIT);
        text.push_str("...");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_attachment_key_from_api_response() {
        assert_eq!(
            decode_attachment_key("020_N_1_240830N020_%E8%AA%8D%E8%A8%BC_51_*****_*****"),
            "020_N_1_240830N020_認証_51_*****_*****"
        );
    }

    #[test]
    fn extracts_filename_from_content_disposition() {
        assert_eq!(
            content_disposition_filename(r#"attachment; filename="022-200057_20200929.zip""#)
                .as_deref(),
            Some("022-200057_20200929.zip")
        );
    }

    #[test]
    fn extracts_encoded_filename_from_content_disposition() {
        assert_eq!(
            content_disposition_filename(
                "attachment; filename*=UTF-8''022-200057_%E8%AA%8D%E8%A8%BC.pdf"
            )
            .as_deref(),
            Some("022-200057_認証.pdf")
        );
    }

    #[test]
    fn deserializes_wrapped_api_response() {
        let json = r#"{
          "gitekiInformation": { "lastUpdateDate": "2026-05-09", "totalCount": "1" },
          "giteki": [
            { "gitekiInfo": {
              "no": "1",
              "techCode": "登録証明機関による工事設計認証",
              "number": "003-180123",
              "date": "2018-07-19",
              "name": "Google LLC",
              "radioEquipmentCode": "第2条第19号",
              "typeName": "G013D",
              "elecWave": "F1D 2402MHz",
              "organName": "(株)ディーエスピーリサーチ"
            } }
          ]
        }"#;

        let response: GitekiListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.information.total_count, 1);
        let info = response.giteki.into_iter().next().unwrap().into_info();
        assert_eq!(info.no, 1);
        assert_eq!(info.number, "003-180123");
    }
}
