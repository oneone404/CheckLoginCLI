use crate::utils::silent_command;
use std::time::Duration;

pub fn adb_tap(serial: &str, x: i32, y: i32) {
    let _ = silent_command("adb")
        .args(["-s", serial, "shell", "input", "tap", &x.to_string(), &y.to_string()])
        .output();
}

pub fn adb_text(serial: &str, text: &str) {
    let safe_text = text.replace(" ", "%s");
    let _ = silent_command("adb")
        .args(["-s", serial, "shell", "input", "text", &safe_text])
        .output();
}

pub fn adb_clear_field(serial: &str) {
    // Move to end
    let _ = silent_command("adb")
        .args(["-s", serial, "shell", "input", "keyevent", "123"])
        .output();

    // Ctrl+A (select all)
    let _ = silent_command("adb")
        .args(["-s", serial, "shell", "input", "keyevent", "KEYCODE_CTRL_LEFT", "KEYCODE_A"])
        .output();

    std::thread::sleep(Duration::from_millis(100));

    // Backspace
    let _ = silent_command("adb")
        .args(["-s", serial, "shell", "input", "keyevent", "67"])
        .output();

    // Extra backspaces
    let _ = silent_command("adb")
        .args(["-s", serial, "shell", "input", "keyevent",
            "67", "67", "67", "67", "67", "67", "67", "67", "67", "67",
            "67", "67", "67", "67", "67", "67", "67", "67", "67", "67"])
        .output();
}

pub fn get_adb_devices() -> Vec<String> {
    let output = silent_command("adb").arg("devices").output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.lines().skip(1)
                .filter(|line| line.contains("\tdevice"))
                .map(|line| line.split('\t').next().unwrap_or("").to_string())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}
