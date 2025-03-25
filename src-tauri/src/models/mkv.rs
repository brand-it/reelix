use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CINFO {
    pub id: i32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct TINFO {
    pub id: i32,
    pub type_code: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct SINFO {
    pub id: i32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct TCOUNT {
    pub title_count: String,
}
#[allow(dead_code)]
#[derive(Debug)]
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
#[derive(Debug, Serialize, Clone)]
pub struct PRGV {
    pub current: i32,
    pub total: i32,
    pub pmax: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct PRGT {
    pub code: String,
    pub id: i32,
    pub name: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct PRGC {
    pub code: String,
    pub id: i32,
    pub name: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct MSG {
    pub code: String,
    pub flags: String,
    pub mcount: String,
    pub message: String,
    pub format: String,
    pub params: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct ParseError {
    pub type_: String,
    pub line: Vec<String>,
}

/// An enum to unify the parsed results.
#[allow(dead_code)]
#[derive(Debug)]
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
