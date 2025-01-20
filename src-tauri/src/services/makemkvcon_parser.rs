use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CINFO {
    pub id: String,
    pub type_: String,
    pub code: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct TINFO {
    pub id: String,
    pub type_code: String,
    pub code: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct SINFO {
    pub id: String,
    pub type_: String,
    pub code: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct TCOUNT {
    pub title_count: String,
}

#[derive(Debug, Clone)]
pub struct DRV {
    pub index: String,
    pub visible: String,
    pub unknown: String,
    pub enabled: String,
    pub flags: String,
    pub drive_name: String,
    pub disc_name: String,
}

#[derive(Debug, Clone)]
pub struct PRGV {
    pub current: String,
    pub total: String,
    pub pmax: String,
}

#[derive(Debug, Clone)]
pub struct PRGT {
    pub code: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PRGC {
    pub code: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MSG {
    pub code: String,
    pub flags: String,
    pub mcount: String,
    pub message: String,
    pub format: String,
    pub params: String,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub type_: String,
    pub line: Vec<String>,
}

/// An enum to unify the parsed results.
#[derive(Debug, Clone)]
pub enum MkvData {
    CINFO(CINFO),
    TINFO(TINFO),
    SINFO(SINFO),
    TCOUNT(TCOUNT),
    DRV(DRV),
    PRGV(PRGV),
    PRGT(PRGT),
    PRGC(PRGC),
    MSG(MSG),
    Error(ParseError),
}

/// This is analogous to the Ruby hash TINFO_CODE_LEGEND
/// mapping an integer code -> a symbol/string.
fn tinfo_code_legend() -> HashMap<i32, &'static str> {
    let mut map = HashMap::new();
    map.insert(2, "name");
    map.insert(8, "chapter_count");
    map.insert(9, "duration");
    map.insert(10, "size");
    map.insert(11, "bytes");
    map.insert(15, "angle");
    map.insert(16, "source_file_name");
    map.insert(25, "segment_count");
    map.insert(26, "segment_map");
    map.insert(27, "filename");
    map.insert(28, "lang");
    map.insert(29, "language");
    map.insert(30, "description");
    map
}

fn define_type(type_str: &str, fields: Vec<String>) -> MkvData {
    match type_str {
        "CINFO" => {
            // CINFO has 4 fields: (id, type_, code, value)
            let needed = 4;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            // If there are extra fields, combine them into the last field
            let mut final_fields = fields.clone();
            let leftover = if final_fields.len() > needed {
                final_fields.split_off(needed - 1)
            } else {
                vec![]
            };
            let combined_last_field = leftover.join(",");
            let mut cinfo_fields = final_fields;
            if !combined_last_field.is_empty() {
                if cinfo_fields.len() == needed {
                    // Overwrite the last field with leftover
                    cinfo_fields[needed - 1] = format!("{},{}",
                        cinfo_fields[needed - 1], combined_last_field);
                }
            }
            MkvData::CINFO(CINFO {
                id: cinfo_fields[0].clone(),
                type_: cinfo_fields[1].clone(),
                code: cinfo_fields[2].clone(),
                value: cinfo_fields[3].clone(),
            })
        }
        "TINFO" => {
            // TINFO has 4 fields: (id, type_code, code, value)
            let needed = 4;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            let mut final_fields = fields.clone();
            let leftover = if final_fields.len() > needed {
                final_fields.split_off(needed - 1)
            } else {
                vec![]
            };
            let combined_last_field = leftover.join(",");
            let mut tinfo_fields = final_fields;
            if !combined_last_field.is_empty() {
                tinfo_fields[needed - 1] = format!("{},{}",
                    tinfo_fields[needed - 1], combined_last_field);
            }
            MkvData::TINFO(TINFO {
                id: tinfo_fields[0].clone(),
                type_code: tinfo_fields[1].clone(),
                code: tinfo_fields[2].clone(),
                value: tinfo_fields[3].clone(),
            })
        }
        "SINFO" => {
            // SINFO has 4 fields: (id, type_, code, value)
            let needed = 4;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            let mut final_fields = fields.clone();
            let leftover = if final_fields.len() > needed {
                final_fields.split_off(needed - 1)
            } else {
                vec![]
            };
            let combined_last_field = leftover.join(",");
            let mut sinfo_fields = final_fields;
            if !combined_last_field.is_empty() {
                sinfo_fields[needed - 1] = format!("{},{}",
                    sinfo_fields[needed - 1], combined_last_field);
            }
            MkvData::SINFO(SINFO {
                id: sinfo_fields[0].clone(),
                type_: sinfo_fields[1].clone(),
                code: sinfo_fields[2].clone(),
                value: sinfo_fields[3].clone(),
            })
        }
        "TCOUNT" => {
            // TCOUNT has 1 field: (title_count)
            let needed = 1;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            MkvData::TCOUNT(TCOUNT {
                title_count: fields[0].clone(),
            })
        }
        "DRV" => {
            // DRV has 7 fields:
            // (index, visible, unknown, enabled, flags, drive_name, disc_name)
            let needed = 7;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            MkvData::DRV(DRV {
                index: fields[0].clone(),
                visible: fields[1].clone(),
                unknown: fields[2].clone(),
                enabled: fields[3].clone(),
                flags: fields[4].clone(),
                drive_name: fields[5].clone(),
                disc_name: fields[6..].join(","),
                // If there's leftover, it merges into disc_name
            })
        }
        "PRGV" => {
            // PRGV has 3 fields: (current, total, pmax)
            let needed = 3;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            MkvData::PRGV(PRGV {
                current: fields[0].clone(),
                total: fields[1].clone(),
                pmax: fields[2..].join(","),
                // leftover merges into pmax
            })
        }
        "PRGT" => {
            // PRGT has 3 fields: (code, id, name)
            let needed = 3;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            MkvData::PRGT(PRGT {
                code: fields[0].clone(),
                id: fields[1].clone(),
                name: fields[2..].join(","),
            })
        }
        "PRGC" => {
            // PRGC has 3 fields: (code, id, name)
            let needed = 3;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            MkvData::PRGC(PRGC {
                code: fields[0].clone(),
                id: fields[1].clone(),
                name: fields[2..].join(","),
            })
        }
        "MSG" => {
            // MSG has 6 fields: (code, flags, mcount, message, format, params)
            let needed = 6;
            if fields.len() < needed {
                return MkvData::Error(ParseError {
                    type_: type_str.to_string(),
                    line: fields,
                });
            }
            MkvData::MSG(MSG {
                code: fields[0].clone(),
                flags: fields[1].clone(),
                mcount: fields[2].clone(),
                message: fields[3].clone(),
                format: fields[4].clone(),
                params: fields[5..].join(","),
            })
        }
        // Unknown type
        _ => MkvData::Error(ParseError {
            type_: type_str.to_string(),
            line: fields,
        }),
    }
}

/// Parses the MKV string similarly to the Ruby version.
///
/// # Arguments
///
/// * `stdout_str` - the string containing lines to parse
///
/// # Returns
///
/// A vector of `MkvData` enum variants representing the parsed data.
pub fn parse_mkv_string(stdout_str: &str) -> Vec<MkvData> {
    let mut results = Vec::new();

    // split by lines
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // split by commas, remove surrounding quotes/backslashes from each piece
        let mut parts: Vec<String> = trimmed
            .split(',')
            .map(|s| s.trim_matches(|c| c == '"' || c == '\\').to_string())
            .collect();

        if parts.is_empty() {
            continue;
        }

        // The first element is something like "TINFO:2", so split that by ':'
        // The Ruby code does: type, id = line.shift.split(':')
        // Then puts the rest in `line`.
        let first_part = parts.remove(0);
        let mut first_split = first_part.splitn(2, ':');
        let type_str = first_split.next().unwrap_or("").to_string();
        let id_part = first_split.next().unwrap_or("").to_string();

        // Now we want to unify [id_part] + parts
        let mut combined = Vec::with_capacity(parts.len() + 1);
        combined.push(id_part);
        combined.extend(parts);

        // pass to define_type
        let parsed = define_type(&type_str, combined);
        results.push(parsed);
    }

    results
}
