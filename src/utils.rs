use colored::*;
use std::process::Command;
use std::io::Write;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::stdout().flush();
}

pub fn pause_and_return() {
    println!();
    log_system("PRESS ENTER TO CONTINUE...");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut String::new());
}

pub fn log_info(ld_index: i32, msg: &str) {
    let time = chrono_time();
    let prefix = if ld_index >= 0 { format!("LD-{}: ", ld_index).cyan().bold() } else { "".normal() };
    println!("{} {}{}",
        format!("[{}]", time).dimmed().bold(),
        prefix,
        msg.to_uppercase().bold()
    );
}

pub fn log_success(ld_index: i32, msg: &str) {
    let time = chrono_time();
    let prefix = if ld_index >= 0 { format!("LD-{}: ", ld_index).green().bold() } else { "".normal() };
    println!("{} {}{}",
        format!("[{}]", time).dimmed().bold(),
        prefix,
        msg.to_uppercase().green().bold()
    );
}

pub fn log_warning(ld_index: i32, msg: &str) {
    let time = chrono_time();
    let prefix = if ld_index >= 0 { format!("LD-{}: ", ld_index).yellow().bold() } else { "".normal() };
    println!("{} {}{}",
        format!("[{}]", time).dimmed().bold(),
        prefix,
        msg.to_uppercase().yellow().bold()
    );
}

pub fn log_error(ld_index: i32, msg: &str) {
    let time = chrono_time();
    let prefix = if ld_index >= 0 { format!("LD-{}: ", ld_index).red().bold() } else { "".normal() };
    println!("{} {}{}",
        format!("[{}]", time).dimmed().bold(),
        prefix,
        msg.to_uppercase().red().bold()
    );
}

pub fn log_system(msg: &str) {
    let time = chrono_time();
    println!("{} {}",
        format!("[{}]", time).dimmed().bold(),
        msg.to_uppercase().bright_white().bold()
    );
}

pub fn chrono_time() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() % 86400; // Seconds since midnight
    let hours = (secs / 3600 + 7) % 24; // UTC+7
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}

pub fn random_delay(min_sec: u64, max_sec: u64) -> u64 {
    use std::time::SystemTime;
    let min_ms = min_sec * 1000;
    let max_ms = max_sec * 1000;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    min_ms + (seed % (max_ms - min_ms + 1))
}
