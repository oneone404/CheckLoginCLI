use std::time::Duration;
use std::sync::atomic::Ordering;
use tokio::time::sleep;
use crate::types::LdInstance;
use crate::state::{RUNNING, ACTIVE_COUNT};
use crate::config::get_config;
use crate::utils::{log_info, log_error, log_warning, log_success, random_delay};
use crate::adb::{adb_tap, adb_clear_field, adb_text};
use crate::template::{find_templates_batch, find_and_click_template};
use crate::account::{claim_account_for_ld, assign_account_to_file};

pub fn do_login(adb_serial: &str, username: &str, password: &str) -> bool {
    let config = get_config();
    let username_x = config.login_username_x;
    let username_y = config.login_username_y;
    let password_x = config.login_password_x;
    let password_y = config.login_password_y;
    let login_btn_x = config.login_btn_x;
    let login_btn_y = config.login_btn_y;

    adb_tap(adb_serial, username_x, username_y);
    std::thread::sleep(Duration::from_millis(300));

    adb_clear_field(adb_serial);
    std::thread::sleep(Duration::from_millis(200));
    adb_text(adb_serial, username);
    std::thread::sleep(Duration::from_millis(300));

    adb_tap(adb_serial, password_x, password_y);
    std::thread::sleep(Duration::from_millis(300));

    adb_clear_field(adb_serial);
    std::thread::sleep(Duration::from_millis(200));
    adb_text(adb_serial, password);
    std::thread::sleep(Duration::from_millis(300));

    adb_tap(adb_serial, login_btn_x, login_btn_y);

    true
}

pub async fn check_login_task(inst: LdInstance) {
    let ld_index = inst.index;
    let adb_serial = inst.adb_serial.clone();

    loop {
        if !RUNNING.load(Ordering::Relaxed) {
            break;
        }

        let config = get_config();
        while ACTIVE_COUNT.load(Ordering::Relaxed) >= config.max_concurrent {
            sleep(Duration::from_millis(500)).await;
            if !RUNNING.load(Ordering::Relaxed) { return; }
        }

        ACTIVE_COUNT.fetch_add(1, Ordering::Relaxed);

        let mut found_something = true;
        while found_something && RUNNING.load(Ordering::Relaxed) {
            found_something = false;

            let check_result = find_templates_batch(&adb_serial, &["btn_ok.png", "btn_login.png"]);

            if check_result.get("btn_ok.png").copied().unwrap_or(false) {
                if find_and_click_template(&adb_serial, "btn_ok.png") {
                    log_warning(ld_index, "CLICKED OK (DISMISSED ERROR)");
                    found_something = true;
                    sleep(Duration::from_millis(2000)).await;
                }
            }

            if check_result.get("btn_login.png").copied().unwrap_or(false) {
                if let Some(account) = claim_account_for_ld(ld_index) {
                    log_info(ld_index, &format!("LOGIN MODAL FOUND - ENTERING: {}", account.username));

                    if find_and_click_template(&adb_serial, "btn_login.png") {
                        sleep(Duration::from_millis(1500)).await;

                        if do_login(&adb_serial, &account.username, &account.password) {
                            log_success(ld_index, &format!("ENTERED USER/PASS FOR {}", account.username));

                            if account.ld_index < 0 {
                                assign_account_to_file(&account.username, ld_index);
                                log_info(ld_index, &format!("ASSIGNED {} TO CSV FILE", account.username));
                            }
                        }
                    }
                    found_something = true;
                } else {
                    log_error(ld_index, "NO ACCOUNTS AVAILABLE!");
                }
            }

            drop(check_result);
        }

        ACTIVE_COUNT.fetch_sub(1, Ordering::Relaxed);

        let delay = random_delay(config.check_interval_min_sec, config.check_interval_max_sec);
        sleep(Duration::from_millis(delay)).await;
    }
}
