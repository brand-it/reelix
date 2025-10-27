use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::Read;

use log::debug;
static TITLES_ZIP: &[u8] = include_bytes!("../../data/titles.txt.zip");

fn load_titles() -> Vec<String> {
    let cursor = std::io::Cursor::new(TITLES_ZIP);
    let mut archive = zip::ZipArchive::new(cursor).expect("Failed to read zip archive");

    let mut file = archive
        .by_name("titles.txt")
        .expect("titles.txt not found in zip");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read titles.txt from zip");

    let mut titles: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
    titles.sort_by_key(|t| t.len());
    titles
}

lazy_static::lazy_static! {
    pub static ref TITLE_LIST: Vec<String> = load_titles();
    pub static ref TITLE_INVERTED_INDEX: HashMap<String, Vec<usize>> = {
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();
        for (id, title) in TITLE_LIST.iter().enumerate() {
            for token in title.to_lowercase().split_whitespace() {
                let mut prefix = String::new();
                for character in token.chars() {
                    prefix.push(character);
                    index.entry(prefix.to_string()).or_default().push(id);
                }
            }
        }
        index
    };
}

pub fn suggestion(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let mut found_indexes: Vec<usize> = Vec::new();
    let text_tokens: Vec<String> = tokens(text);
    for text_token in text_tokens.iter() {
        if let Some(indexes) = TITLE_INVERTED_INDEX.get(text_token) {
            if found_indexes.is_empty() {
                found_indexes = indexes.to_vec();
            } else {
                found_indexes = overlapping_vectors(&found_indexes, indexes)
            }
            if found_indexes.is_empty() {
                break;
            }
        };
    }
    if found_indexes.is_empty() {
        None
    } else {
        let lowercase_text = text.to_lowercase();
        let mut results: Vec<String> = found_indexes
            .iter()
            .map(|&i| TITLE_LIST[i].to_lowercase())
            .collect();

        results.retain(|t| t.contains(&lowercase_text));
        if let Some(suggestion) = results.first() {
            let suggestion_parts: Vec<&str> = suggestion.split(&lowercase_text).collect();

            let suggest = suggestion_parts
                .last()
                .map(|suggested_text| suggested_text.to_string());
            debug!("Found Suggestion for {text} {suggestion} {suggest:?}");
            suggest
        } else {
            None
        }
    }
}

fn overlapping_vectors(a: &[usize], b: &[usize]) -> Vec<usize> {
    a.iter()
        .copied()
        .collect::<HashSet<_>>()
        .intersection(&b.iter().copied().collect())
        .copied()
        .collect()
}

fn tokens(text: &str) -> Vec<String> {
    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    text.split_whitespace()
        .flat_map(|t| re.split(t))
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}
