use serde::Serialize;

#[allow(dead_code)]

pub struct CINFO {
    pub id: u32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]

pub struct TINFO {
    pub id: u32,
    pub type_code: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]

pub struct SINFO {
    pub id: u32,
    pub type_: String,
    pub code: String,
    pub value: String,
}
#[allow(dead_code)]

pub struct TCOUNT {
    pub title_count: String,
}
#[allow(dead_code)]

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
#[derive(Serialize, Clone)]
pub struct PRGV {
    pub current: u32,
    pub total: u32,
    pub pmax: u32,
}
#[allow(dead_code)]
#[derive(Clone)]
pub struct PRGT {
    pub code: String,
    pub id: u32,
    pub name: String,
}
#[allow(dead_code)]

pub struct PRGC {
    pub code: String,
    pub id: u32,
    pub name: String,
}
#[allow(dead_code)]
#[derive(Clone)]
pub struct MSG {
    pub code: String,
    pub flags: String,
    pub mcount: String,
    pub message: String,
    pub format: String,
    pub params: String,
}
#[allow(dead_code)]

pub struct ParseError {
    pub type_: String,
    pub line: Vec<String>,
}

/// An enum to unify the parsed results.
#[allow(dead_code)]

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
