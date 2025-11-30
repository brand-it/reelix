use serde::Serialize;

/// Disc information output message (CINFO)
/// Represents a disc-level attribute, such as disc name, type, or other metadata.
/// Reference: makemkvcon output, CINFO:id,code,value
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct CINFO {
    pub id: u32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
/// Title information output message (TINFO)
/// Represents a title-level attribute, such as title name, length, or other metadata.
/// Reference: makemkvcon output, TINFO:id,code,value
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct TINFO {
    pub id: u32,
    pub type_code: String,
    pub code: String,
    pub value: String,
}
/// Stream information output message (SINFO)
/// Represents a stream-level attribute, such as audio, video, or subtitle stream details.
/// Reference: makemkvcon output, SINFO:id,code,value
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct SINFO {
    pub id: u32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
/// Title count output message (TCOUNT)
/// Represents the number of titles found on the disc.
/// Reference: makemkvcon output, TCOUT:count
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct TCOUNT {
    pub title_count: String,
}
/// Drive scan message (DRV)
/// Represents information about an optical drive and the disc inserted.
/// Reference: makemkvcon output, DRV:index,visible,enabled,flags,drive name,disc name
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
/// Progress bar values for current and total progress (PRGV)
/// Represents the current, total, and maximum values for a progress bar.
/// Reference: makemkvcon output, PRGV:current,total,max
#[allow(clippy::upper_case_acronyms)]
#[derive(Serialize, Clone)]
pub struct PRGV {
    pub current: u32,
    pub total: u32,
    pub pmax: u32,
}
/// Progress title message (PRGT)
/// Represents the total progress title, including code, id, and name.
/// Reference: makemkvcon output, PRGT:code,id,name
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct PRGT {
    pub code: String,
    pub id: u32,
    pub name: String,
}
/// Progress current message (PRGC)
/// Represents the current progress title, including code, id, and name.
/// Reference: makemkvcon output, PRGC:code,id,name
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

/// Message output (MSG)
/// Represents a general message from makemkvcon, including code, flags, message, and parameters.
/// Reference: makemkvcon output, MSG:code,flags,count,message,format,param0,param1,...
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
/// Parse error message (Error)
/// Represents an error encountered during parsing of makemkvcon output.
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub struct ParseError {
    pub type_: String,
    pub line: Vec<String>,
}

/// An enum to unify the parsed results from makemkvcon output.
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum MkvData {
    /// Disc Information Output (CINFO)
    CINFO(CINFO),
    /// Title Information Output (TINFO)
    TINFO(TINFO),
    /// Stream Information Output (SINFO)
    SINFO(SINFO),
    /// Title Count Output (TCOUNT)
    TCOUNT(TCOUNT),
    /// Drive Scan Message (DRV)
    DRV(DRV),
    /// Progress Bar Values (PRGV)
    PRGV(PRGV),
    /// Progress Title Message (PRGT)
    PRGT(PRGT),
    /// Progress Current Message (PRGC)
    PRGC(PRGC),
    /// General Message Output (MSG)
    MSG(MSG),
    /// Parse Error Message (Error)
    Error(ParseError),
}
