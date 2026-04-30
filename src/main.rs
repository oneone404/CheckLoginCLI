mod types;
mod config;
mod state;
mod utils;
mod adb;
mod ldplayer;
mod account;
mod template;
mod auto_nph;
mod tasks;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use colored::*;
use std::io::{self, Write};
use std::process::Command;

use config::{get_config, load_config, AppConfig, get_exe_dir};
use state::{RUNNING, START_TIME, ACCOUNTS};
use utils::{log_system, log_success, log_info, log_error, log_warning, silent_command, clear_screen, pause_and_return};
use ldplayer::get_ld_instances;
use account::load_accounts;
use tasks::check_login_task;
use auto_nph::{run_auto_config_nph, run_login_nph};

fn kill_previous_instance() {
    let current_pid = std::process::id();

    let output = silent_command("tasklist")
        .args(["/FI", "IMAGENAME eq check_login.exe", "/FO", "CSV", "/NH"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let pid_str = parts[1].trim_matches('"');
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if pid != current_pid {
                        let _ = silent_command("taskkill")
                            .args(["/F", "/PID", &pid.to_string()])
                            .output();
                    }
                }
            }
        }
    }
}



fn run_auto_start_feature(exit_after: bool) {
    let config = get_config();
    if config.auto_start_lds.is_empty() {
        log_system("NO LDS SELECTED FOR AUTO-START.");
        return;
    }

    log_system(&format!("STARTING LDS: {:?}", config.auto_start_lds));
    for &idx in &config.auto_start_lds {
        log_system(&format!("LAUNCHING LD-{}...", idx));
        let _ = silent_command("ldconsole.exe")
            .args(["launch", "--index", &idx.to_string()])
            .output();
        std::thread::sleep(Duration::from_millis(1500)); // Delay between launches
    }
    log_system("COMMANDS SENT TO ALL LDS.");

    if config.auto_open_nph_enabled {
        run_nph_activation();
    }

    if config.auto_sort_after_start {
        log_system(&format!("WAITING {} SECONDS BEFORE SORTING...", config.auto_sort_delay_sec));
        std::thread::sleep(Duration::from_secs(config.auto_sort_delay_sec));
        run_sort_windows(false);
    }
    
    if exit_after {
        log_system("AUTO EXITING...");
        std::process::exit(0);
    } else {
        pause_and_return();
    }
}

fn config_auto_start() {
    clear_screen();
    println!();
    println!("{}", "--- AUTO START LD CONFIG ---".bright_cyan().bold());
    let mut config = load_config(); // Read fresh config
    let status = if config.auto_start_enabled { "ENABLED".green().bold() } else { "DISABLED".red().bold() };
    let sort_after = if config.auto_sort_after_start { "YES".green().bold() } else { "NO".red().bold() };
    let open_nph = if config.auto_open_nph_enabled { "YES".green().bold() } else { "NO".red().bold() };
    
    let nph_exists = std::path::Path::new("C:\\Program Files\\NPHTool\\tool.exe").exists();
    let nph_status = if nph_exists { "FOUND".green().bold() } else { "NOT FOUND".red().bold() };

    println!("{}", format!("AUTO-START: {}", status).bold());
    println!("{}", format!("AUTO-SORT AFTER START: {}", sort_after).bold());
    println!("{}", format!("AUTO-OPEN NPH TOOL: {}", open_nph).bold());
    println!("{}", format!("NPH TOOL STATUS (PATH): {}", nph_status).bold());
    println!("{}", format!("NPH ACTIVE COORDS: ({}, {})", config.nph_active_x, config.nph_active_y).bold());
    println!("{}", format!("AUTO-SORT DELAY: {} SEC", config.auto_sort_delay_sec).bold());
    println!("{}", format!("SELECTED LDS: {:?}", config.auto_start_lds).bold());
    println!("{}", format!("SORT COLUMNS: {}", config.sort_columns).bold());
    println!();
    println!("{}", "WHAT DO YOU WANT TO DO?".bold());
    println!("  {}  {}", "[1]".cyan().bold(), "TOGGLE AUTO-START WITH WINDOWS".bold());
    println!("  {}  {}", "[2]".cyan().bold(), "TOGGLE AUTO-SORT AFTER START".bold());
    println!("  {}  {}", "[3]".cyan().bold(), "TOGGLE AUTO-OPEN NPH TOOL".bold());
    println!("  {}  {}", "[4]".cyan().bold(), "CHANGE AUTO-SORT DELAY".bold());
    println!("  {}  {}", "[5]".cyan().bold(), "CHANGE LD LIST".bold());
    println!("  {}  {}", "[6]".cyan().bold(), "CHANGE SORT COLUMNS".bold());
    println!("  {}  {}", "[7]".cyan().bold(), "CHANGE NPH ACTIVE COORDS".bold());
    println!("  {}  {}", "[0]".cyan().bold(), "GO BACK".bold());
    print!("\n{}", ">> CHOICE: ".yellow().bold());
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let choice = input.trim().parse::<u8>().unwrap_or(0);

    match choice {
        1 => {
            config.auto_start_enabled = !config.auto_start_enabled;
            if config.auto_start_enabled {
                let exe_path = std::env::current_exe().unwrap_or_default();
                let cmd = format!("\"{}\" --auto-start", exe_path.to_string_lossy());
                let _ = silent_command("reg")
                    .args(["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "CheckLoginAutoStart", "/t", "REG_SZ", "/d", &cmd, "/f"])
                    .output();
                log_success(-1, "AUTO-START ENABLED!");
            } else {
                let _ = silent_command("reg")
                    .args(["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "CheckLoginAutoStart", "/f"])
                    .output();
                log_success(-1, "AUTO-START DISABLED!");
            }
            save_config(&config);
            pause_and_return();
        }
        2 => {
            config.auto_sort_after_start = !config.auto_sort_after_start;
            log_success(-1, &format!("AUTO-SORT AFTER START: {}", if config.auto_sort_after_start { "ON" } else { "OFF" }));
            save_config(&config);
            pause_and_return();
        }
        3 => {
            config.auto_open_nph_enabled = !config.auto_open_nph_enabled;
            log_success(-1, &format!("AUTO-OPEN NPH TOOL: {}", if config.auto_open_nph_enabled { "ON" } else { "OFF" }));
            save_config(&config);
            pause_and_return();
        }
        4 => {
            print!("{}", "ENTER DELAY IN SECONDS (DEFAULT 5): ".bold());
            let _ = io::stdout().flush();
            let mut delay_input = String::new();
            let _ = io::stdin().read_line(&mut delay_input);
            if let Ok(delay) = delay_input.trim().parse::<u64>() {
                config.auto_sort_delay_sec = delay;
                log_success(-1, &format!("AUTO-SORT DELAY SET TO: {} SEC", config.auto_sort_delay_sec));
                save_config(&config);
            }
            pause_and_return();
        }
        5 => {
            print!("{}", "ENTER LD INDICES (E.G. 1,2,5 OR 1 2 5): ".bold());
            let _ = io::stdout().flush();
            let mut lds_input = String::new();
            let _ = io::stdin().read_line(&mut lds_input);
            
            let mut lds: Vec<i32> = Vec::new();
            for part in lds_input.replace(",", " ").split_whitespace() {
                if let Ok(idx) = part.parse::<i32>() {
                    lds.push(idx);
                }
            }
            config.auto_start_lds = lds;
            log_success(-1, &format!("LD LIST SAVED: {:?}", config.auto_start_lds));
            save_config(&config);
            pause_and_return();
        }
        6 => {
            print!("{}", "ENTER NUMBER OF COLUMNS (DEFAULT 5): ".bold());
            let _ = io::stdout().flush();
            let mut col_input = String::new();
            let _ = io::stdin().read_line(&mut col_input);
            if let Ok(cols) = col_input.trim().parse::<i32>() {
                config.sort_columns = cols;
                log_success(-1, &format!("SORT COLUMNS SET TO: {}", config.sort_columns));
                save_config(&config);
            }
            pause_and_return();
        }
        7 => {
            print!("{}", "ENTER NPH ACTIVE X: ".bold());
            let _ = io::stdout().flush();
            let mut x_input = String::new();
            let _ = io::stdin().read_line(&mut x_input);
            
            print!("{}", "ENTER NPH ACTIVE Y: ".bold());
            let _ = io::stdout().flush();
            let mut y_input = String::new();
            let _ = io::stdin().read_line(&mut y_input);
            
            if let (Ok(x), Ok(y)) = (x_input.trim().parse::<i32>(), y_input.trim().parse::<i32>()) {
                config.nph_active_x = x;
                config.nph_active_y = y;
                log_success(-1, &format!("NPH ACTIVE COORDS SET TO: ({}, {})", x, y));
                save_config(&config);
            }
            pause_and_return();
        }
        _ => {}
    }
}

fn save_config(config: &AppConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let exe_dir = get_exe_dir();
        let config_path = exe_dir.join("config.json");
        let _ = std::fs::write(config_path, json);
    }
}

fn run_sort_windows(should_pause: bool) {
    clear_screen();
    log_system("CUSTOM ARRANGING LDPLAYER WINDOWS...");
    
    #[cfg(windows)]
    {
        use crate::auto_nph::{find_all_ld_windows, get_screen_size, get_window_size, move_window, enable_dpi_aware};
        
        enable_dpi_aware();
        let mut windows = find_all_ld_windows();
        if windows.is_empty() {
            log_error(0, "NO LDPLAYER WINDOWS FOUND!");
            pause_and_return();
            return;
        }

        // Sort by title: LDPlayer, LDPlayer(1), LDPlayer(2)...
        // Helper to extract number from title for better sorting
        windows.sort_by(|a, b| {
            let get_num = |s: &str| -> i32 {
                if let Some(start) = s.find('(') {
                    if let Some(end) = s.find(')') {
                        return s[start+1..end].parse().unwrap_or(0);
                    }
                }
                if s == "LDPlayer" { -1 } else { 999 }
            };
            get_num(&a.1).cmp(&get_num(&b.1))
        });

        let (sw, sh) = get_screen_size();
        log_info(-1, &format!("SCREEN RESOLUTION: {}X{}", sw, sh));

        enable_dpi_aware();

        // Get instances with HWNDs from ldconsole list2
        let output = match silent_command("ldconsole.exe").arg("list2").output() {
            Ok(o) => o,
            Err(_) => {
                log_error(-1, "FAILED TO RUN LDCONSOLE.EXE!");
                pause_and_return();
                return;
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut windows = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let index = parts[0].parse::<i32>().unwrap_or(-1);
                let title = parts[1].to_string();
                let hwnd = parts[2].parse::<isize>().unwrap_or(0);
                let is_running = parts[4] == "1";

                if is_running && hwnd != 0 {
                    windows.push((index, hwnd, title));
                }
            }
        }

        if windows.is_empty() {
            log_error(-1, "NO RUNNING LDPLAYER WINDOWS FOUND!");
            pause_and_return();
            return;
        }

        // Sort by index (the first element in our tuple)
        windows.sort_by_key(|w| w.0);

        let (ww, wh) = get_window_size(windows[0].1);
        
        let config = get_config();
        let cols = config.sort_columns;
        
        let target_width = sw / cols;
        let aspect_ratio = wh as f32 / ww as f32;
        let target_height = (target_width as f32 * aspect_ratio) as i32;
        
        log_info(-1, &format!("SORTING {} WINDOWS BY INDEX ({} COLUMNS)", windows.len(), cols));

        for (i, (idx, hwnd, title)) in windows.iter().enumerate() {
            let row = (i as i32) / cols;
            let col = (i as i32) % cols;
            
            let x = col * target_width;
            let y = row * target_height;
            
            move_window(*hwnd, x, y, target_width, target_height);
            log_success(*idx, &format!("{} -> POS({}, {})", title.to_uppercase(), x, y));
        }
    }
    
    #[cfg(not(windows))]
    {
        log_error(-1, "ONLY SUPPORTED ON WINDOWS!");
    }

    log_success(-1, "ALL WINDOWS ARRANGED!");
    if should_pause {
        pause_and_return();
    }
}

fn run_close_all() {
    clear_screen();
    log_system("CLOSING ALL LDPLAYERS AND NPH TOOL...");
    
    // Close LDPlayers
    let _ = silent_command("ldconsole.exe")
        .arg("quitall")
        .output();
    
    // Close NPH Tool
    let _ = silent_command("taskkill")
        .args(["/F", "/IM", "tool.exe", "/T"])
        .output();
        
    log_success(-1, "ALL CLOSED!");
    pause_and_return();
}

fn run_power_options() {
    clear_screen();
    println!();
    println!("{}", "--- POWER OPTIONS ---".bright_red().bold());
    println!("  {}  {}", "[1]".cyan().bold(), "SHUTDOWN COMPUTER".bold());
    println!("  {}  {}", "[2]".cyan().bold(), "RESTART COMPUTER".bold());
    println!("  {}  {}", "[0]".cyan().bold(), "CANCEL".bold());
    print!("\n{}", ">> CHOICE: ".yellow().bold());
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let choice = input.trim().parse::<u8>().unwrap_or(0);

    match choice {
        1 => {
            log_warning(-1, "COMPUTER WILL SHUTDOWN IN 5 SECONDS! PRESS CTRL+C TO CANCEL.");
            std::thread::sleep(Duration::from_secs(5));
            log_system("SHUTTING DOWN...");
            let _ = silent_command("shutdown")
                .args(["/s", "/t", "0", "/f"])
                .output();
        }
        2 => {
            log_warning(-1, "COMPUTER WILL RESTART IN 5 SECONDS! PRESS CTRL+C TO CANCEL.");
            std::thread::sleep(Duration::from_secs(5));
            log_system("RESTARTING...");
            let _ = silent_command("shutdown")
                .args(["/r", "/t", "0", "/f"])
                .output();
        }
        _ => {}
    }
}

fn run_nph_activation() {
    log_system("OPENING NPH TOOL...");
    let tool_path = "C:\\Program Files\\NPHTool\\tool.exe";
    let _ = silent_command("cmd")
        .args(["/c", "start", "", tool_path])
        .output();
    
    log_system("WAITING 5 SECONDS FOR NPH TO LOAD...");
    std::thread::sleep(Duration::from_secs(5));
    
    use crate::auto_nph::{find_and_focus, click_relative, get_hwnd_by_title};
    if find_and_focus("NPH") {
        let hwnd = get_hwnd_by_title("NPH");
        if hwnd != 0 {
            let config = get_config();
            log_system(&format!("CLICKING NPH ACTIVE BUTTON AT ({}, {})...", config.nph_active_x, config.nph_active_y));
            click_relative(hwnd, config.nph_active_x, config.nph_active_y);
            log_success(-1, "NPH ACTIVATED!");
        }
    } else {
        log_error(-1, "NPH WINDOW NOT FOUND!");
    }
}

fn run_mouse_pos_tool() {
    clear_screen();
    println!("{}", "--- MOUSE POSITION TOOL ---".bright_magenta().bold());
    println!("{}", "FOCUS THE TARGET WINDOW (NPHTOOL) TO GET RELATIVE COORDS.".yellow().bold());
    println!("{}", "PRESS CTRL+C TO STOP AND RETURN TO MENU.".bright_red().bold());
    println!();

    use crate::auto_nph::{get_hwnd_by_title, get_mouse_pos_relative};
    let target_hwnd = get_hwnd_by_title("NPH");

    loop {
        if let Some((x, y)) = get_mouse_pos_relative(target_hwnd) {
            print!("\r{}", format!("CURRENT MOUSE POS (REL TO NPH): X={}, Y={}      ", x, y).bold().cyan());
            let _ = io::stdout().flush();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn show_menu() -> u8 {
    println!();
    println!("{}", "========================================================".bright_cyan().bold());
    println!("{}  {}", "  ".on_bright_cyan(), " TOOL CLI V1.2".bright_white().bold());
    println!("{}", "========================================================".bright_cyan().bold());
    println!();
    println!("  {}  {}", "[1]".cyan().bold(), "CHECK LOGIN".bold());
    println!("  {}  {}", "[2]".cyan().bold(), "AUTO CONFIG NPH".bold());
    println!("  {}  {}", "[3]".cyan().bold(), "AUTO LOGIN NPH".bold());
    println!("  {}  {}", "[4]".cyan().bold(), "RUN AUTO START LD".bold());
    println!("  {}  {}", "[5]".cyan().bold(), "SETTINGS (AUTO START)".bold());
    println!("  {}  {}", "[6]".cyan().bold(), "CLOSE ALL LD & NPH".bold());
    println!("  {}  {}", "[7]".cyan().bold(), "POWER OPTIONS (SHUT/RESTART)".bold());
    println!("  {}  {}", "[8]".cyan().bold(), "MOUSE POSITION TOOL (RELATIVE)".bold());
    println!("  {}  {}", "[9]".cyan().bold(), "AUTO RUN NPH TOOL".bold());
    println!("  {}  {}", "[0]".cyan().bold(), "EXIT...".bold());
    println!();
    println!("{}", "========================================================".bright_cyan().bold());

    print!("\n{}", ">> CHOICE: ".yellow().bold());
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return 0;
    }

    input.trim().parse::<u8>().unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let _ = enable_ansi_support::enable_ansi_support();

    // Check hidden flag
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--auto-start".to_string()) {
        run_auto_start_feature(true);
        return;
    }

    kill_previous_instance();

    loop {
        clear_screen();
        let choice = show_menu();

        match choice {
            1 => run_check_login().await,
            2 => run_auto_config_nph(),
            3 => run_login_nph(),
            4 => run_auto_start_feature(false),
            5 => config_auto_start(),
            6 => run_close_all(),
            7 => run_power_options(),
            8 => run_mouse_pos_tool(),
            9 => {
                run_nph_activation();
                pause_and_return();
            }
            _ => {
                log_system("EXITING.");
                break;
            }
        }
    }
}

async fn run_check_login() {
    println!();
    println!("{}", "--- CHECK LOGIN MODE ---".bright_cyan().bold());
    println!("{}", "    PRESS CTRL+C TO STOP".dimmed().bold());
    println!();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!();
        log_system("STOPPING...");
        RUNNING.store(false, Ordering::Relaxed);
        r.store(false, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler");

    let accounts = load_accounts();
    log_system(&format!("LOADED {} ACCOUNTS", accounts.len()));
    {
        let mut acc_lock = ACCOUNTS.lock().unwrap();
        *acc_lock = accounts;
    }

    let instances = get_ld_instances();
    if instances.is_empty() {
        log_system("NO RUNNING LDPLAYER FOUND!");
        log_system("PLEASE START LDPLAYER BEFORE RUNNING THE TOOL.");
        pause_and_return();
        return;
    }

    let config = get_config();
    let _ = START_TIME.set(std::time::Instant::now());

    log_system(&format!("TIM THAY {} LDPLAYER DANG CHAY:", instances.len()));
    for inst in &instances {
        println!("  {} {} {}: {} ({})",
            "->".green().bold(),
            "LD".bold(),
            inst.index.to_string().cyan().bold(),
            inst.name.to_uppercase().white().bold(),
            inst.adb_serial.dimmed().bold()
        );
    }

    println!();
    log_system(&format!("CONFIG: INTERVAL {}-{}S, MAX {} THREADS, RESTART AFTER {} MINS",
        config.check_interval_min_sec,
        config.check_interval_max_sec,
        config.max_concurrent,
        config.restart_minutes
    ));
    println!("{}", "-".repeat(60).bright_white().bold());

    let mut handles = Vec::new();
    for inst in instances {
        let handle = tokio::spawn(check_login_task(inst));
        handles.push(handle);
    }

    if config.restart_minutes > 0 {
        let restart_ms = config.restart_minutes * 60 * 1000;
        tokio::spawn(async move {
            sleep(Duration::from_millis(restart_ms)).await;
            if RUNNING.load(Ordering::Relaxed) {
                log_system(&format!("RUNNING FOR {} MINS - AUTO RESTARTING...", config.restart_minutes));
                RUNNING.store(false, Ordering::Relaxed);

                let exe_path = std::env::current_exe().unwrap_or_default();
                let _ = Command::new(&exe_path).spawn();
            }
        });
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!();
    log_system("STOPPED ALL TASKS.");
}
