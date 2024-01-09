use std::collections::HashMap;
use curl::easy::Easy;
use crate::error::RaptoboError;

pub fn download(url: String) -> Result<Vec<String>, RaptoboError> {
    let mut easy = Easy::new();
    
    easy.url(&url)
    .map_err(|e| RaptoboError::new(&e.to_string()))?;
    
    let mut content = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            content.extend_from_slice(data);
            Ok(data.len())
        })
        .map_err(|e| RaptoboError::new(&e.to_string()))?;
    
        transfer.perform()
        .map_err(|e| RaptoboError::new(&e.to_string()))?;
    }

    let content = String::from_utf8(content)
    .map_err(|e| RaptoboError::new(&e.to_string()))?;

    let content = content.split("\n").map(|l| l.to_string()).collect();

    Ok(content)
}

pub fn parse_metadata(content: Vec<String>) -> Result<Vec<HashMap<String, Vec<String>>>, RaptoboError> {
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
                },
                Some((k, v)) => {
                    key = String::from(k);
                    value.push(String::from(v));
                }
            }
        }
    }

    Ok(data)
}

pub fn stanza_value(key: &str, stanza: &HashMap<String, Vec<String>>) -> Result<String, RaptoboError> {
    let value = stanza.get(key).ok_or(RaptoboError::new(&format!("[stanza_value] {} not found!", &key)))?;
    let value = &value[0];
    Ok(value.trim().to_string())
}

pub fn stanza_opt_value(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<String> {
    match stanza_value(key, stanza) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn stanza_list(key: &str, stanza: &HashMap<String, Vec<String>>) -> Result<Vec<String>, RaptoboError> {
    let values = stanza.get(key).ok_or(RaptoboError::new(&format!("[stanza_list] {} not found!", &key)))?;
    let values = &values[0];
    let values: Vec<String> = values.split(" ").map(|v| v.trim().to_string()).filter(|v| v.len() > 0).collect();
    Ok(values)
}

pub fn stanza_text(key: &str, stanza: &HashMap<String, Vec<String>>) -> Result<String, RaptoboError> {
    let values = stanza.get(key).ok_or(RaptoboError::new(&format!("[stanza_text] {} not found!", &key)))?;
    
    let values: Vec<&str> = values.into_iter().map(|l| l.trim()).collect();
    let text = values.join("\n");

    Ok(text)
}

pub fn stanza_lines(key: &str, stanza: &HashMap<String, Vec<String>>, filter_empty: bool) -> Result<Vec<String>, RaptoboError> {
    let values = stanza.get(key).ok_or(RaptoboError::new(&format!("[stanza_lines] {} not found!", &key)))?;
    
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

pub fn stanza_files(key: &str, stanza: &HashMap<String, Vec<String>>) -> Result<Vec<File>, RaptoboError> {
    let lines = stanza_lines(key, stanza, true)?;

    let mut files: Vec<File> = Vec::new();
    
    for l in lines.into_iter() {
        let parts: Vec<&str> = l.split(" ").map(|v| v.trim()).filter(|v| v.len() > 0).collect();
        
        if parts.len() != 3 {
            return Err(RaptoboError::new(&format!("[stanza_files] invalid file, wrong number of elements: {}", l)));
        }

        let size = parts[1].parse::<u64>().map_err(|e| RaptoboError::new(&e.to_string()))?;
        
        files.push(File { hash: parts[0].to_string(), size, path: parts[2].to_string() })
    }

    Ok(files)
}
