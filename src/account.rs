use std::fs;
use crate::types::Account;
use crate::config::get_accounts_path;
use crate::state::{ACCOUNTS, USED_ACCOUNTS, SESSION_CLAIMS};

pub fn load_accounts() -> Vec<Account> {
    let path = get_accounts_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut accounts = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let username = parts[0].to_string();
            let password = parts[1].to_string();
            let ld_index: i32 = if parts.len() >= 3 {
                let ld_str = parts[2].trim();
                if ld_str.starts_with("LD-") {
                    ld_str[3..].parse().unwrap_or(-1)
                } else {
                    ld_str.parse().unwrap_or(-1)
                }
            } else {
                -1
            };
            accounts.push(Account { username, password, ld_index });
        }
    }

    accounts
}

pub fn claim_account_for_ld(ld_index: i32) -> Option<Account> {
    let accounts = ACCOUNTS.lock().ok()?;
    let mut used = USED_ACCOUNTS.lock().ok()?;
    let mut claims = SESSION_CLAIMS.lock().ok()?;

    for acc in accounts.iter() {
        if acc.ld_index == ld_index {
            return Some(acc.clone());
        }
    }

    if let Some(claimed_username) = claims.get(&ld_index) {
        for acc in accounts.iter() {
            if acc.username == *claimed_username {
                return Some(acc.clone());
            }
        }
    }

    let claimed_usernames: std::collections::HashSet<_> = claims.values().cloned().collect();
    for acc in accounts.iter() {
        if acc.ld_index < 0
            && !used.contains(&acc.username)
            && !claimed_usernames.contains(&acc.username)
        {
            used.insert(acc.username.clone());
            claims.insert(ld_index, acc.username.clone());
            return Some(acc.clone());
        }
    }

    None
}

pub fn assign_account_to_file(username: &str, ld_index: i32) {
    let path = get_accounts_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut updated_lines: Vec<String> = Vec::new();
    for line in content.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            updated_lines.push(line.to_string());
            continue;
        }

        let parts: Vec<&str> = line_trimmed.split('|').collect();
        if parts.len() >= 2 && parts[0] == username {
            let new_line = format!("{}|{}|LD-{}", parts[0], parts[1], ld_index);
            updated_lines.push(new_line);
        } else {
            updated_lines.push(line.to_string());
        }
    }

    let _ = fs::write(&path, updated_lines.join("\n"));
}
