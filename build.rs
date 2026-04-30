use std::env;

fn main() {
    // Only run on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();

        // Set icon
        res.set_icon("icon.ico");

        // Set app name and metadata
        res.set("ProductName", "Check Login");
        res.set("FileDescription", "Check Login CLI - LDPlayer");
        res.set("LegalCopyright", "CBS 2026");
        res.set("ProductVersion", "1.0.0");
        res.set("FileVersion", "1.0.0");

        // Compile resource
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
