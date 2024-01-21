use crate::error::RaptoboError;
use chrono::{DateTime, FixedOffset};
use curl::easy::Easy;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::{Cursor, Read};

pub fn download_xz(url: &str) -> Result<Vec<String>, RaptoboError> {
    let mut content = download_raw(url)?;
    log::debug!("[download_xz] len: {}", content.len());

    let decompressed = lzma::decompress(&mut content)
    .map_err(|e| RaptoboError::new(&e.to_string()))?;
    let data = String::from_utf8(decompressed)
    .map_err(|e| RaptoboError::new(&e.to_string()))?;

    let data = data.split("\n").map(|l| l.to_string()).collect();

    Ok(data)
}

pub fn download_gz(url: &str) -> Result<Vec<String>, RaptoboError> {
    let content = download_raw(url)?;
    let content = Cursor::new(content);
    let mut decoder = GzDecoder::new(content);
    let mut data = String::new();
    let _len = decoder
        .read_to_string(&mut data)
        .map_err(|e| RaptoboError::new(&e.to_string()))?;
    let data = data.split("\n").map(|l| l.to_string()).collect();

    Ok(data)
}

pub fn download_raw(url: &str) -> Result<Vec<u8>, RaptoboError> {
    let mut easy = Easy::new();

    easy.url(&url)
        .map_err(|e| RaptoboError::new(&e.to_string()))?;

    let mut content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                content.extend_from_slice(data);
                Ok(data.len())
            })
            .map_err(|e| RaptoboError::new(&e.to_string()))?;

        transfer
            .perform()
            .map_err(|e| RaptoboError::new(&e.to_string()))?;
    }

    Ok(content)
}

pub fn download(url: &str) -> Result<Vec<String>, RaptoboError> {
    let content = download_raw(url)?;

    let content = String::from_utf8(content).map_err(|e| RaptoboError::new(&e.to_string()))?;

    let content = content.split("\n").map(|l| l.to_string()).collect();

    Ok(content)
}

pub fn parse_metadata(
    content: Vec<String>,
) -> Result<Vec<HashMap<String, Vec<String>>>, RaptoboError> {
    let mut data = Vec::new();
    let mut stanza = HashMap::new();

    let mut key: String = String::from("");
    let mut value: Vec<String> = Vec::new();

    for line in content.into_iter() {
        if line.trim().is_empty() {
            // new stanza
            if !stanza.is_empty() {
                data.push(stanza);
            }

            stanza = HashMap::new();

            continue;
        }

        if line.starts_with(" ") {
            // follow up line
            value.push(line);
        } else {
            if !value.is_empty() {
                stanza.insert(key, value);
                value = Vec::new();
            }

            match line.split_once(":") {
                None => {
                    log::debug!("[parse_metadata] invalid line, missing key: {}", line);
                    key = String::from("")
                }
                Some((k, v)) => {
                    key = String::from(k);
                    value.push(String::from(v));
                }
            }
        }
    }

    Ok(data)
}

pub fn stanza_value(
    key: &str,
    stanza: &HashMap<String, Vec<String>>,
) -> Result<String, RaptoboError> {
    let value = stanza.get(key).ok_or(RaptoboError::new(&format!(
        "[stanza_value] {} not found!",
        &key
    )))?;
    let value = &value[0];
    Ok(value.trim().to_string())
}

pub fn stanza_opt_value(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<String> {
    match stanza_value(key, stanza) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn stanza_list(
    key: &str,
    stanza: &HashMap<String, Vec<String>>,
) -> Result<Vec<String>, RaptoboError> {
    let values = stanza.get(key).ok_or(RaptoboError::new(&format!(
        "[stanza_list] {} not found!",
        &key
    )))?;
    let values = &values[0];
    let values: Vec<String> = values
        .split(" ")
        .map(|v| v.trim().to_string())
        .filter(|v| v.len() > 0)
        .collect();
    Ok(values)
}

pub fn stanza_opt_list(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
    let list = match stanza_list(key, stanza) {
        Ok(list) => list,
        Err(_e) => return None,
    };
    if list.is_empty() {
        None
    } else {
        Some(list)
    }
}

pub fn stanza_text(
    key: &str,
    stanza: &HashMap<String, Vec<String>>,
) -> Result<String, RaptoboError> {
    let values = stanza.get(key).ok_or(RaptoboError::new(&format!(
        "[stanza_text] {} not found!",
        &key
    )))?;

    let values: Vec<&str> = values.into_iter().map(|l| l.trim()).collect();
    let text = values.join("\n");

    Ok(text)
}

pub fn stanza_date(
    key: &str,
    stanza: &HashMap<String, Vec<String>>,
) -> Option<DateTime<FixedOffset>> {
    let value = stanza_value(key, stanza);
    match value {
        Err(_) => None,
        Ok(date) => match DateTime::parse_from_rfc2822(&date) {
            Ok(date) => Some(date),
            Err(e) => {
                log::error!("[stanza_date] parse error: {}", e);
                None
            }
        },
    }
}

pub fn stanza_opt_text(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<String> {
    let text = stanza_text(key, stanza);
    match text {
        Ok(text) => Some(text),
        Err(_) => None,
    }
}

pub fn stanza_lines(
    key: &str,
    stanza: &HashMap<String, Vec<String>>,
    filter_empty: bool,
) -> Result<Vec<String>, RaptoboError> {
    let values = stanza.get(key).ok_or(RaptoboError::new(&format!(
        "[stanza_lines] {} not found!",
        &key
    )))?;

    let values = values.into_iter().map(|l| l.trim().to_string());
    let values: Vec<String> = if filter_empty {
        values.filter(|l| l.len() > 0).collect()
    } else {
        values.collect()
    };

    Ok(values)
}

#[derive(Debug)]
pub struct File {
    pub hash: String,
    pub size: u64,
    pub path: String,
}

pub fn stanza_files(
    key: &str,
    stanza: &HashMap<String, Vec<String>>,
) -> Result<Vec<File>, RaptoboError> {
    let lines = stanza_lines(key, stanza, true)?;

    let mut files: Vec<File> = Vec::new();

    for l in lines.into_iter() {
        let parts: Vec<&str> = l
            .split(" ")
            .map(|v| v.trim())
            .filter(|v| v.len() > 0)
            .collect();

        if parts.len() != 3 {
            return Err(RaptoboError::new(&format!(
                "[stanza_files] invalid file, wrong number of elements: {}",
                l
            )));
        }

        let size = parts[1]
            .parse::<u64>()
            .map_err(|e| RaptoboError::new(&e.to_string()))?;

        files.push(File {
            hash: parts[0].to_string(),
            size,
            path: parts[2].to_string(),
        })
    }

    Ok(files)
}

pub fn stanza_opt_files(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<Vec<File>> {
    let files = stanza_files(key, stanza);
    match files {
        Ok(files) => {
            if files.is_empty() {
                None
            } else {
                Some(files)
            }
        }
        Err(_) => {
            None
        }
    }
}
