
#[derive(Debug, Default)]
pub struct TitleInfo {
    pub id: i32,
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
    pub fn new(id: i32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn segment_map(&self) -> Option<Vec<i32>> {
        self.segment_map
            .as_ref()
            .map(|map| map.split(',').filter_map(|s| s.parse().ok()).collect())
    }

    pub fn duration_seconds(&self) -> Option<i32> {
        self.duration.as_ref().and_then(|d| {
            let parts: Vec<&str> = d.split(':').collect();
            if parts.len() == 3 {
                let hours = parts[0].parse::<i32>().ok()?;
                let minutes = parts[1].parse::<i32>().ok()?;
                let seconds = parts[2].parse::<i32>().ok()?;
                Some(hours * 3600 + minutes * 60 + seconds)
            } else {
                None
            }
        })
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
