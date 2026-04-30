use crate::types::LdInstance;
use crate::adb::get_adb_devices;
use crate::utils::silent_command;

pub fn get_ld_instances() -> Vec<LdInstance> {
    let adb_devices = get_adb_devices();

    let output = match silent_command("ldconsole.exe").arg("list2").output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut instances = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            if let Ok(idx) = parts[0].parse::<i32>() {
                let name = parts[1].to_string();

                // Check if running
                if !check_ld_running(idx) { continue; }

                let expected_port = 5555 + (idx * 2);
                let emulator_serial = format!("emulator-{}", expected_port - 1);
                let ip_serial = format!("127.0.0.1:{}", expected_port);

                let adb_serial = if adb_devices.contains(&emulator_serial) {
                    emulator_serial
                } else if adb_devices.contains(&ip_serial) {
                    ip_serial
                } else {
                    format!("LD-{}", idx)
                };

                instances.push(LdInstance { index: idx, name, adb_serial });
            }
        }
    }

    instances
}

pub fn check_ld_running(index: i32) -> bool {
    let output = silent_command("ldconsole.exe")
        .args(["isrunning", "--index", &index.to_string()])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_lowercase().contains("running"),
        Err(_) => false,
    }
}
