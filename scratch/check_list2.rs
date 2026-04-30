use std::process::Command;

fn main() {
    let output = Command::new("ldconsole.exe")
        .arg("list2")
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf16_lossy(&output.stdout.iter().map(|&x| x as u16).collect::<Vec<u16>>())); // Wait, ldconsole usually outputs in CP936 or UTF-8?
    // Let's just use from_utf8_lossy
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
