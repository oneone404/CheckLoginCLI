use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, atomic::{AtomicBool, AtomicUsize}};
use std::sync::OnceLock;
use crate::types::{Account, Roi};

pub static ROI_CACHE: OnceLock<HashMap<String, Roi>> = OnceLock::new();
pub static RUNNING: AtomicBool = AtomicBool::new(true);
pub static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();

lazy_static::lazy_static! {
    pub static ref ACCOUNTS: Mutex<Vec<Account>> = Mutex::new(Vec::new());
    pub static ref ACTIVE_COUNT: AtomicUsize = AtomicUsize::new(0);
    // Track accounts đã dùng trong session (không cho 2 LD dùng cùng 1 acc)
    pub static ref USED_ACCOUNTS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    // Map LD index -> username (đã claim trong session)
    pub static ref SESSION_CLAIMS: Mutex<HashMap<i32, String>> = Mutex::new(HashMap::new());
}
