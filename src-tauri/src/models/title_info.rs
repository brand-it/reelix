use serde::Serialize;

#[derive(Default, Serialize, Clone)]
pub struct TitleInfo {
    pub id: u32,
    pub name: Option<String>,
    pub chapter_count: Option<i32>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bytes: Option<String>,
    pub angle: Option<String>,
    pub source_file_name: Option<String>,
    pub segment_count: Option<i32>,
    pub segment_map: Option<String>,
    pub filename: Option<String>,
    pub lang: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
}

impl TitleInfo {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn has_chapters(&self) -> bool {
        self.chapter_count.unwrap_or(0) > 0
    }

    pub fn duration_seconds(&self) -> Option<u64> {
        self.duration.as_ref().and_then(|d| {
            let parts: Vec<&str> = d.split(':').collect();
            if parts.len() == 3 {
                let hours = parts[0].parse::<u64>().ok()?;
                let minutes = parts[1].parse::<u64>().ok()?;
                let seconds = parts[2].parse::<u64>().ok()?;
                Some(hours * 3600 + minutes * 60 + seconds)
            } else {
                None
            }
        })
    }

    pub fn within_range(&self, range: Option<std::ops::Range<u64>>) -> bool {
        let range = match range {
            Some(r) => r,
            None => return false,
        };
        if !self.has_chapters() {
            return false;
        }
        if let Some(duration) = self.duration_seconds() {
            range.contains(&duration)
        } else {
            false
        }
    }

    pub fn set_field(&mut self, field: &str, value: String) {
        match field {
            "name" => self.name = Some(value),
            "chapter_count" => self.chapter_count = value.parse().ok(),
            "duration" => self.duration = Some(value),
            "size" => self.size = Some(value),
            "bytes" => self.bytes = Some(value),
            "angle" => self.angle = Some(value),
            "source_file_name" => self.source_file_name = Some(value),
            "segment_count" => self.segment_count = value.parse().ok(),
            "segment_map" => self.segment_map = Some(value),
            "filename" => self.filename = Some(value),
            "lang" => self.lang = Some(value),
            "language" => self.language = Some(value),
            "description" => self.description = Some(value),
            _ => {}
        }
    }
}
