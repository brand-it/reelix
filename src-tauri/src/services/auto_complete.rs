use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

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

fn build_index(titles: &[String]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (id, title) in titles.iter().enumerate() {
        for token in title.to_lowercase().split_whitespace() {
            let chars: Vec<char> = token.chars().collect();
            let token_len = chars.len();
            if token_len == 0 {
                continue;
            }

            for prefix_len in 1..=token_len.min(2) {
                let prefix: String = chars.iter().take(prefix_len).collect();
                index.entry(prefix).or_default().push(id);
            }

            let mut prefix_len = 3;
            while prefix_len <= token_len {
                let prefix: String = chars.iter().take(prefix_len).collect();
                index.entry(prefix).or_default().push(id);
                prefix_len += 3;
            }

            if token_len % 3 != 0 {
                let full_token: String = chars.iter().collect();
                index.entry(full_token).or_default().push(id);
            }
        }
    }

    for indexes in index.values_mut() {
        indexes.sort_unstable();
        indexes.dedup();
    }

    index
}

fn grouped_lookup_token(token: &str) -> Option<String> {
    let chars: Vec<char> = token.chars().collect();
    if chars.is_empty() {
        return None;
    }

    if chars.len() < 3 {
        return Some(chars.into_iter().collect());
    }

    let grouped_len = (chars.len() / 3) * 3;
    Some(chars.into_iter().take(grouped_len).collect())
}

fn next_completion_chunk(suggestion: &str, query: &str) -> Option<String> {
    let start = suggestion.find(query)? + query.len();
    let suffix = suggestion.get(start..)?;

    let trimmed_suffix = suffix.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
    if trimmed_suffix.is_empty() {
        return None;
    }

    let next_word_fragment: String = trimmed_suffix
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .take(3)
        .collect();

    if next_word_fragment.is_empty() {
        None
    } else {
        Some(next_word_fragment)
    }
}

static TITLE_LIST: OnceLock<Vec<String>> = OnceLock::new();
static TITLE_INVERTED_INDEX: OnceLock<HashMap<String, Vec<usize>>> = OnceLock::new();
static INIT_STARTED: AtomicBool = AtomicBool::new(false);

pub fn init_background() {
    if INIT_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        let titles = load_titles();
        let index = build_index(&titles);
        let _ = TITLE_LIST.set(titles);
        let _ = TITLE_INVERTED_INDEX.set(index);
    });
}

pub fn is_ready() -> bool {
    TITLE_LIST.get().is_some() && TITLE_INVERTED_INDEX.get().is_some()
}

pub fn suggestion(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let titles = TITLE_LIST.get()?;
    let inverted = TITLE_INVERTED_INDEX.get()?;
    let mut found_indexes: Vec<usize> = Vec::new();
    let text_tokens: Vec<String> = tokens(text);
    for text_token in text_tokens.iter() {
        if let Some(lookup_token) = grouped_lookup_token(text_token) {
            if let Some(indexes) = inverted.get(&lookup_token) {
                if found_indexes.is_empty() {
                    found_indexes = indexes.to_vec();
                } else {
                    found_indexes = overlapping_vectors(&found_indexes, indexes)
                }
                if found_indexes.is_empty() {
                    break;
                }
            }
        }
    }

    if found_indexes.is_empty() {
        None
    } else {
        let lowercase_text = text.to_lowercase();
        let mut results: Vec<String> = found_indexes
            .iter()
            .map(|&i| titles[i].to_lowercase())
            .collect();
        results.sort_by_key(|t| t.len());

        results.retain(|t| t.contains(&lowercase_text));
        if let Some(suggestion) = results.first() {
            let suggest = next_completion_chunk(suggestion, &lowercase_text);
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
