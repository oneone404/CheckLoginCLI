# CHECK LOGIN CLI - LDPLAYER AUTOMATION TOOL

A powerful, modular CLI tool designed for professional LDPlayer management and automation.

## 🚀 MAIN FEATURES

- **[1] CHECK LOGIN**: Automate account login verification across multiple LDPlayer instances.
- **[2] AUTO CONFIG NPH**: Rapidly configure NPH settings with optimized click delays.
- **[3] AUTO LOGIN NPH**: Seamlessly log into NPH accounts with pre-defined coordinates.
- **[4] RUN AUTO START LD**: Launch selected LDPlayer instances simultaneously.
- **[5] SETTINGS (AUTO START)**:
    - Toggle Windows Auto-Start (Registry integration).
    - Manage LDPlayer list for auto-launch.
    - Toggle **AUTO-SORT** after launching.
    - Configure custom **SORT COLUMNS**.
- **[6] CLOSE ALL LD & NPH**: Kill all running LDPlayer instances and the NPHTool process instantly.
- **[7] POWER OPTIONS**: Quick access to **SHUTDOWN** or **RESTART** your computer with safety delays.

## 🛠 ADVANCED CAPABILITIES

- **CUSTOM WINDOW SORTING**: 
    - Uses Win32 API for pixel-perfect window arrangement.
    - **AUTO-RESIZE**: Automatically scales LDPlayer windows to fit perfectly in a grid (Optimized for **4K/2K/FULL HD**).
    - **INDEX SORT**: Arranges windows according to their LDMultiPlayer index (0, 1, 2...).
- **MODULAR ARCHITECTURE**: Highly maintainable code structure (Config, ADB, LDPlayer, Tasks, UI Utils).
- **PREMIUM UI**: 
    - Fully translated to **ENGLISH**.
    - **BOLD & UPPERCASE** styling for high readability.
    - ANSI-based screen clearing for a clean console experience.

## 📦 INSTALLATION & USAGE

1. Clone the repository.
2. Ensure you have `ldconsole.exe` in your system PATH or place the tool in the LDPlayer directory.
3. Run the executable:
   ```bash
   ./CheckLogin.exe
   ```
4. For auto-start at boot, use the built-in Settings menu.

## ⚙️ CONFIGURATION

Settings are stored in `config.json`. Key parameters:
- `sort_columns`: Number of windows per row (Default: 5).
- `auto_sort_after_start`: Automatically arrange windows after launching LDs.
- `config_nph_delay_ms`: Click delay for automation tasks.

---
*DEVELOPED WITH PRIDE FOR HIGH-PERFORMANCE AUTOMATION.*
