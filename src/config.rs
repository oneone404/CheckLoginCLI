use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::OnceLock;
use crate::utils::{log_system, log_error, log_warning};

const DEFAULT_MAX_CONCURRENT: usize = 5;
const DEFAULT_CHECK_INTERVAL_MIN_SEC: u64 = 10;
const DEFAULT_CHECK_INTERVAL_MAX_SEC: u64 = 15;
const DEFAULT_RESTART_MINUTES: u64 = 240;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_restart_minutes")]
    pub restart_minutes: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default = "default_check_interval_min")]
    pub check_interval_min_sec: u64,
    #[serde(default = "default_check_interval_max")]
    pub check_interval_max_sec: u64,
    #[serde(default = "default_config_nph_delay_ms")]
    pub config_nph_delay_ms: u64,
    #[serde(default = "default_login_half_delay_sec")]
    pub login_half_delay_sec: u64,

    // Auto-start LDs config
    #[serde(default = "default_auto_start_enabled")]
    pub auto_start_enabled: bool,
    #[serde(default = "default_auto_start_lds")]
    pub auto_start_lds: Vec<i32>,
    #[serde(default = "default_auto_sort_after_start")]
    pub auto_sort_after_start: bool,
    #[serde(default = "default_auto_sort_delay_sec")]
    pub auto_sort_delay_sec: u64,
    #[serde(default = "default_auto_open_nph_enabled")]
    pub auto_open_nph_enabled: bool,

    // Login Coordinates
    #[serde(default = "default_login_username_x")]
    pub login_username_x: i32,
    #[serde(default = "default_login_username_y")]
    pub login_username_y: i32,
    #[serde(default = "default_login_password_x")]
    pub login_password_x: i32,
    #[serde(default = "default_login_password_y")]
    pub login_password_y: i32,
    #[serde(default = "default_login_btn_x")]
    pub login_btn_x: i32,
    #[serde(default = "default_login_btn_y")]
    pub login_btn_y: i32,
    #[serde(default = "default_sort_columns")]
    pub sort_columns: i32,
    #[serde(default = "default_nph_active_x")]
    pub nph_active_x: i32,
    #[serde(default = "default_nph_active_y")]
    pub nph_active_y: i32,
    #[serde(default = "default_nph_refresh_x")]
    pub nph_refresh_x: i32,
    #[serde(default = "default_nph_refresh_y")]
    pub nph_refresh_y: i32,
    #[serde(default = "default_nph_scroll_x")]
    pub nph_scroll_x: i32,
    #[serde(default = "default_nph_scroll_y1")]
    pub nph_scroll_y1: i32,
    #[serde(default = "default_nph_scroll_y2")]
    pub nph_scroll_y2: i32,
}

fn default_restart_minutes() -> u64 { DEFAULT_RESTART_MINUTES }
fn default_max_concurrent() -> usize { DEFAULT_MAX_CONCURRENT }
fn default_check_interval_min() -> u64 { DEFAULT_CHECK_INTERVAL_MIN_SEC }
fn default_check_interval_max() -> u64 { DEFAULT_CHECK_INTERVAL_MAX_SEC }
fn default_config_nph_delay_ms() -> u64 { 40 }
fn default_login_half_delay_sec() -> u64 { 60 }
fn default_auto_start_enabled() -> bool { false }
fn default_auto_start_lds() -> Vec<i32> { Vec::new() }
fn default_auto_sort_after_start() -> bool { false }
fn default_auto_sort_delay_sec() -> u64 { 5 }
fn default_auto_open_nph_enabled() -> bool { false }
fn default_nph_active_x() -> i32 { 910 }
fn default_nph_active_y() -> i32 { 125 }
fn default_nph_refresh_x() -> i32 { 515 }
fn default_nph_refresh_y() -> i32 { 320 }
fn default_nph_scroll_x() -> i32 { 980 }
fn default_nph_scroll_y1() -> i32 { 200 }
fn default_nph_scroll_y2() -> i32 { 600 }

fn default_login_username_x() -> i32 { 480 }
fn default_login_username_y() -> i32 { 213 }
fn default_login_password_x() -> i32 { 480 }
fn default_login_password_y() -> i32 { 261 }
fn default_login_btn_x() -> i32 { 480 }
fn default_login_btn_y() -> i32 { 316 }
fn default_sort_columns() -> i32 { 5 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            restart_minutes: DEFAULT_RESTART_MINUTES,
            max_concurrent: DEFAULT_MAX_CONCURRENT,
            check_interval_min_sec: DEFAULT_CHECK_INTERVAL_MIN_SEC,
            check_interval_max_sec: DEFAULT_CHECK_INTERVAL_MAX_SEC,
            config_nph_delay_ms: 40,
            login_half_delay_sec: 60,
            auto_start_enabled: false,
            auto_start_lds: Vec::new(),
            auto_sort_after_start: false,
            auto_sort_delay_sec: 5,
            auto_open_nph_enabled: false,
            login_username_x: 480,
            login_username_y: 213,
            login_password_x: 480,
            login_password_y: 261,
            login_btn_x: 480,
            login_btn_y: 316,
            sort_columns: 5,
            nph_active_x: 910,
            nph_active_y: 125,
            nph_refresh_x: 515,
            nph_refresh_y: 320,
            nph_scroll_x: 980,
            nph_scroll_y1: 200,
            nph_scroll_y2: 600,
        }
    }
}

pub fn load_config() -> AppConfig {
    let mut possible_paths = Vec::new();
    possible_paths.push("config.json".to_string());
    
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let config_next_to_exe = exe_dir.join("config.json");
            if let Some(path_str) = config_next_to_exe.to_str() {
                possible_paths.push(path_str.to_string());
            }
        }
    }

    for path in possible_paths {
        if let Ok(mut content) = fs::read_to_string(&path) {
            if content.starts_with('\u{feff}') {
                content = content.replace('\u{feff}', "");
            }
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    log_system(&format!("LOADED CONFIG FROM: {}", path));
                    return config;
                }
                Err(e) => {
                    log_error(0, &format!("CONFIG STRUCTURE ERROR IN {}: {}", path, e));
                }
            }
        }
    }
    
    log_warning(0, "NO VALID CONFIG FILE FOUND, USING DEFAULTS.");
    AppConfig::default()
}

pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(load_config)
}

pub fn get_exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf()
}

pub fn get_accounts_path() -> String {
    get_exe_dir().join("Acc.csv").to_string_lossy().to_string()
}

pub fn get_template_dir() -> String {
    get_exe_dir().join("template").to_string_lossy().to_string()
}

pub fn get_roi_config_path() -> String {
    get_exe_dir().join("template").join("roi_config.json").to_string_lossy().to_string()
}
