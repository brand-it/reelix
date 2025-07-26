use serde::Serialize;

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct CINFO {
    pub id: u32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct TINFO {
    pub id: u32,
    pub type_code: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct SINFO {
    pub id: u32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct TCOUNT {
    pub title_count: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct DRV {
    pub index: i32,
    pub visible: i32,
    pub unknown: i32,
    pub enabled: i32,
    pub flags: String,
    pub drive_name: String,
    pub disc_name: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Clone)]
pub struct PRGV {
    pub current: u32,
    pub total: u32,
    pub pmax: u32,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct PRGT {
    pub code: String,
    pub id: u32,
    pub name: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct PRGC {
    pub code: String,
    pub id: u32,
    pub name: String,
}

// Just trying to describe the message codes to help me use that to print
// error messages
// fn describe_msg_code(code: u32) -> &'static str {
//     match code {
//         1002 => "Internal exception or trace log",
//         2023 => "Summary of hash check errors",
//         4004 => "File is corrupt or unreadable at a byte offset",
//         4009 => "Too many AV synchronization issues",
//         5003 => "Failed to save file",
//         5004 => "Title save result summary",
//         5037 => "Copy operation completed (summary)",
//         5076 => "Hash check failed for a file at a given offset",
//         5077 => "Too many hash check failures for one file",
//         _ => "Unknown or uncategorized message code",
//     }
// }

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct MSG {
    pub code: i32,
    pub flags: String,
    pub mcount: String,
    pub message: String,
    pub format: String,
    pub params: String,
}
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct ParseError {
    pub type_: String,
    pub line: Vec<String>,
}

/// An enum to unify the parsed results.
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
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
