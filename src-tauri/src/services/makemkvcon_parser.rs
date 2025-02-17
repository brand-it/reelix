use crate::models::mkv::{
    MkvData, ParseError, CINFO, DRV, MSG, PRGC,
    PRGT, PRGV, SINFO, TCOUNT, TINFO,
};

fn define_type<I: IntoIterator<Item = String>>(type_str: &str, fields: I) -> MkvData {
    match type_str {
        "CINFO" => {
            let mut iter = fields.into_iter();
            MkvData::CINFO(CINFO {
                id: iter.next().unwrap(),
                type_: iter.next().unwrap(),
                code: iter.next().unwrap(),
                value: iter.collect::<Vec<String>>().join(","),
            })
        }
        "TINFO" => {
            let mut iter = fields.into_iter();
            MkvData::TINFO(TINFO {
                id: iter.next().unwrap(),
                type_code: iter.next().unwrap(),
                code: iter.next().unwrap(),
                value: iter.collect::<Vec<String>>().join(","),
            })
        }
        "SINFO" => {
            let mut iter = fields.into_iter();
            MkvData::SINFO(SINFO {
                id: iter.next().unwrap(),
                type_: iter.next().unwrap(),
                code: iter.next().unwrap(),
                value: iter.collect::<Vec<String>>().join(","),
            })
        }
        "TCOUNT" => MkvData::TCOUNT(TCOUNT {
            title_count: fields.into_iter().next().unwrap(),
        }),
        "DRV" => {
            let mut iter = fields.into_iter();
            MkvData::DRV(DRV {
                index: iter.next().unwrap(),
                visible: iter.next().unwrap(),
                unknown: iter.next().unwrap(),
                enabled: iter.next().unwrap(),
                flags: iter.next().unwrap(),
                drive_name: iter.next().unwrap(),
                disc_name: iter.collect::<Vec<String>>().join(","),
            })
        }
        "PRGV" => {
            let mut iter = fields.into_iter();
            MkvData::PRGV(PRGV {
                current: iter.next().unwrap(),
                total: iter.next().unwrap(),
                pmax: iter.collect::<Vec<String>>().join(","),
            })
        }
        "PRGT" => {
            let mut iter = fields.into_iter();
            MkvData::PRGT(PRGT {
                code: iter.next().unwrap(),
                id: iter.next().unwrap(),
                name: iter.collect::<Vec<String>>().join(","),
            })
        }
        "PRGC" => {
            let mut iter = fields.into_iter();
            MkvData::PRGC(PRGC {
                code: iter.next().unwrap(),
                id: iter.next().unwrap(),
                name: iter.collect::<Vec<String>>().join(","),
            })
        }
        "MSG" => {
            let mut iter = fields.into_iter();
            MkvData::MSG(MSG {
                code: iter.next().unwrap(),
                flags: iter.next().unwrap(),
                mcount: iter.next().unwrap(),
                message: iter.next().unwrap(),
                format: iter.next().unwrap(),
                params: iter.collect::<Vec<String>>().join(","),
            })
        }
        // Unknown type
        _ => MkvData::Error(ParseError {
            type_: type_str.to_string(),
            line: fields.into_iter().collect::<Vec<String>>(),
        }),
    }
}

pub fn parse_mkv_string(stdout_str: &str) -> Vec<MkvData> {
    let mut results: Vec<MkvData> = Vec::new();

    // split by lines
    for line in stdout_str.lines() {
        let trimmed: &str = line.trim();
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
        let first_part: String = parts.remove(0);
        let mut first_split: std::str::SplitN<'_, char> = first_part.splitn(2, ':');
        let type_str: String = first_split.next().unwrap_or("").to_string();
        let id_part: String = first_split.next().unwrap_or("").to_string();

        // Now we want to unify [id_part] + parts
        let mut combined: Vec<String> = Vec::with_capacity(parts.len() + 1);
        combined.push(id_part);
        combined.extend(parts);

        // pass to define_type
        let parsed: MkvData = define_type(&type_str, combined);
        results.push(parsed);
    }

    results
}
