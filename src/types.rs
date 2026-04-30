use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdInstance {
    pub index: i32,
    pub name: String,
    pub adb_serial: String,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub ld_index: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Roi {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}
