use std::io::{self, Write};
use std::time::Duration;
use colored::*;
use crate::config::get_config;
use crate::utils::{log_system, log_error, log_success, clear_screen, pause_and_return};

#[cfg(windows)]
mod win32 {
    const SW_RESTORE: i32 = 9;
    const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
    const MOUSEEVENTF_LEFTUP: u32 = 0x0004;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(lpEnumFunc: extern "system" fn(isize, isize) -> i32, lParam: isize) -> i32;
        fn GetWindowTextW(hWnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
        fn SetForegroundWindow(hWnd: isize) -> i32;
        fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
        fn IsIconic(hWnd: isize) -> i32;
        fn SetCursorPos(x: i32, y: i32) -> i32;
        fn mouse_event(dwFlags: u32, dx: u32, dy: u32, dwData: u32, dwExtraInfo: usize);
        fn SetProcessDPIAware() -> i32;
        fn MoveWindow(hWnd: isize, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;
        fn GetSystemMetrics(nIndex: i32) -> i32;
        fn GetWindowRect(hWnd: isize, lpRect: *mut Rect) -> i32;
        fn GetClassNameW(hWnd: isize, lpClassName: *mut u16, nMaxCount: i32) -> i32;
        fn GetCursorPos(lpPoint: *mut Point) -> i32;
        fn ScreenToClient(hWnd: isize, lpPoint: *mut Point) -> i32;
        fn ClientToScreen(hWnd: isize, lpPoint: *mut Point) -> i32;
        fn PostMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> i32;
    }
 
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;

    #[repr(C)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    pub struct Rect {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    pub fn enable_dpi_aware() {
        unsafe {
            SetProcessDPIAware();
        }
    }

    struct SearchData {
        search_text: String,
        found_hwnd: isize,
    }

    extern "system" fn enum_windows_callback(hwnd: isize, lparam: isize) -> i32 {
        unsafe {
            let data = &mut *(lparam as *mut SearchData);
            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                if title.contains(&data.search_text) {
                    data.found_hwnd = hwnd;
                    return 0; // Stop enumeration
                }
            }
        }
        1 // Continue
    }

    pub fn find_and_focus(title_part: &str) -> bool {
        unsafe {
            let mut data = SearchData {
                search_text: title_part.to_string(),
                found_hwnd: 0,
            };

            EnumWindows(enum_windows_callback, &mut data as *mut SearchData as isize);

            if data.found_hwnd == 0 {
                return false;
            }
            if IsIconic(data.found_hwnd) != 0 {
                ShowWindow(data.found_hwnd, SW_RESTORE);
            }
            SetForegroundWindow(data.found_hwnd);
            true
        }
    }

    pub fn click(x: i32, y: i32) {
        unsafe {
            SetCursorPos(x, y);
            mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
            mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        }
    }

    pub fn get_screen_size() -> (i32, i32) {
        unsafe {
            (GetSystemMetrics(0), GetSystemMetrics(1))
        }
    }

    pub fn move_window(hwnd: isize, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            MoveWindow(hwnd, x, y, w, h, 1);
        }
    }

    pub fn get_window_size(hwnd: isize) -> (i32, i32) {
        unsafe {
            let mut rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
            GetWindowRect(hwnd, &mut rect);
            (rect.right - rect.left, rect.bottom - rect.top)
        }
    }

    pub fn find_all_ld_windows() -> Vec<(isize, String)> {
        struct ListData {
            list: Vec<(isize, String)>,
        }
        extern "system" fn enum_callback(hwnd: isize, lparam: isize) -> i32 {
            unsafe {
                let data = &mut *(lparam as *mut ListData);
                
                // Check class name first
                let mut class_buf = [0u16; 256];
                let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), class_buf.len() as i32);
                if class_len > 0 {
                    let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);
                    if class_name == "LDPlayerMainFrame" {
                        // It is an LDPlayer window, get title
                        let mut title_buf = [0u16; 512];
                        let title_len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
                        if title_len > 0 {
                            let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
                            data.list.push((hwnd, title));
                        }
                    }
                }
            }
            1
        }
        let mut data = ListData { list: Vec::new() };
        unsafe {
            EnumWindows(enum_callback, &mut data as *mut ListData as isize);
        }
        data.list
    }

    pub fn get_hwnd_by_title(title_part: &str) -> isize {
        unsafe {
            let mut data = SearchData {
                search_text: title_part.to_string(),
                found_hwnd: 0,
            };
            EnumWindows(enum_windows_callback, &mut data as *mut SearchData as isize);
            data.found_hwnd
        }
    }

    pub fn get_mouse_pos_relative(hwnd: isize) -> Option<(i32, i32)> {
        unsafe {
            let mut pt = Point { x: 0, y: 0 };
            if GetCursorPos(&mut pt) != 0 {
                if hwnd != 0 {
                    ScreenToClient(hwnd, &mut pt);
                }
                return Some((pt.x, pt.y));
            }
            None
        }
    }

    pub fn click_relative(hwnd: isize, x: i32, y: i32) {
        if hwnd == 0 { return; }
        unsafe {
            let lparam = ((y as isize) << 16) | (x as isize);
            PostMessageW(hwnd, WM_LBUTTONDOWN, 1, lparam);
            std::thread::sleep(std::time::Duration::from_millis(10));
            PostMessageW(hwnd, WM_LBUTTONUP, 0, lparam);
        }
    }

    pub fn click_bg(hwnd: isize, x: i32, y: i32) {
        click_relative(hwnd, x, y);
    }
}

#[cfg(windows)]
pub use win32::*;

const NPH_CLICK_DATA: [[(i32, i32); 5]; 15] = [
    [(1768,692),(1735,1352),(2434,698),(2470,932),(1297,691)],
    [(1782,779),(1747,1452),(2418,780),(2395,1025),(1292,779)],
    [(1768,878),(1727,1545),(2417,874),(2391,1123),(1299,881)],
    [(1751,963),(1727,1626),(2398,965),(2409,1202),(1297,968)],
    [(1785,1059),(1729,1721),(2406,1065),(2400,1306),(1292,1062)],
    [(1769,1155),(1779,1826),(2390,1157),(2485,1400),(1298,1161)],
    [(1752,1252),(1791,1941),(2399,1245),(2410,1502),(1289,1250)],
    [(1756,1342),(1741,1136),(2415,1345),(2453,1605),(1288,1334)],
    [(1755,1427),(1723,1237),(2392,1433),(2445,1676),(1297,1442)],
    [(1759,1522),(1770,1336),(2391,1532),(2402,1767),(1299,1528)],
    [(1765,1621),(1756,1409),(2404,1622),(2436,1879),(1289,1619)],
    [(1784,1725),(1765,1514),(2399,1702),(2424,1961),(1292,1714)],
    [(1754,1803),(1810,1606),(2397,1801),(2407,1665),(1291,1802)],
    [(1823,1903),(1767,1696),(2405,1897),(2383,1761),(1288,1904)],
    [(1762,1995),(1757,1792),(2416,1995),(2405,1852),(1291,1993)],
];

const NPH_LOGIN_DATA: [(i32, i32); 15] = [
    (965, 689), (1081, 786), (1082, 883), (1082, 967), (1075, 1065),
    (1086, 1159), (1082, 1248), (1084, 1341), (1081, 1432), (1081, 1523),
    (1079, 1624), (1086, 1709), (1087, 1802), (1078, 1900), (1078, 1989)
];



pub fn run_auto_config_nph() {
    clear_screen();
    println!();
    println!("{}", "========================================================".bright_cyan().bold());
    println!("  {}  {}", "[1]".cyan().bold(), "EVENT".bold());
    println!("  {}  {}", "[2]".cyan().bold(), "GREEN".bold());
    println!("{}", "========================================================".bright_cyan().bold());
    print!("\n{}", ">> SELECT FILTER TYPE: ".yellow().bold());
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let choice = input.trim().parse::<u8>().unwrap_or(1);
    let is_xanh_la = choice == 2;

    println!();
    log_system("AUTO CONFIG NPH - SEARCHING FOR NPH WINDOW...");

    #[cfg(windows)]
    {
        win32::enable_dpi_aware();

        if !win32::find_and_focus("NPH") {
            log_error(0, "NPH WINDOW NOT FOUND! PLEASE OPEN NPH FIRST.");
            pause_and_return();
            return;
        }

        log_success(0, "NPH WINDOW FOUND");
        let hwnd = win32::get_hwnd_by_title("NPH");
        if hwnd == 0 {
             log_error(0, "COULD NOT GET WINDOW HANDLE!");
             pause_and_return();
             return;
        }

        std::thread::sleep(Duration::from_millis(300));

        let config = get_config();
        let delay_ms = config.config_nph_delay_ms;

        for (i, clicks) in NPH_CLICK_DATA.iter().enumerate() {
            let ld_num = (i + 1) as i32;

            win32::click_bg(hwnd, clicks[0].0, clicks[0].1);
            std::thread::sleep(Duration::from_millis(delay_ms));

            win32::click_bg(hwnd, clicks[1].0, clicks[1].1);
            std::thread::sleep(Duration::from_millis(delay_ms));

            win32::click_bg(hwnd, clicks[2].0, clicks[2].1);
            std::thread::sleep(Duration::from_millis(delay_ms));

            let loc_y = if is_xanh_la { clicks[3].1 - 150 } else { clicks[3].1 };
            win32::click_bg(hwnd, clicks[3].0, loc_y);
            std::thread::sleep(Duration::from_millis(delay_ms));

            win32::click_bg(hwnd, clicks[4].0 + 100, clicks[4].1);
            std::thread::sleep(Duration::from_millis(delay_ms));

            log_success(ld_num, "DONE");
        }

        println!();
        log_system("=== DONE ALL - FINISHED CONFIGURING 15 LDS ===");
    }

    #[cfg(not(windows))]
    {
        log_error(0, "ONLY SUPPORTED ON WINDOWS!");
    }

    pause_and_return();
}

pub fn run_login_nph() {
    clear_screen();
    println!();
    print!("{}", ">> HOW MANY INSTANCES TO LOGIN (MAX 15)? ".yellow().bold());
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let count = input.trim().parse::<usize>().unwrap_or(15).clamp(1, 15);

    println!();
    log_system("AUTO LOGIN NPH - SEARCHING FOR NPH WINDOW...");

    #[cfg(windows)]
    {
        win32::enable_dpi_aware();

        if !win32::find_and_focus("NPH") {
            log_error(0, "NPH WINDOW NOT FOUND! PLEASE OPEN NPH FIRST.");
            pause_and_return();
            return;
        }

        log_success(0, "NPH WINDOW FOUND");
        let hwnd = win32::get_hwnd_by_title("NPH");
        if hwnd == 0 {
             log_error(0, "COULD NOT GET WINDOW HANDLE!");
             pause_and_return();
             return;
        }

        std::thread::sleep(Duration::from_millis(300));

        let config = get_config();
        let delay_ms = config.config_nph_delay_ms;

        for (i, click) in NPH_LOGIN_DATA.iter().take(count).enumerate() {
            let ld_num = (i + 1) as i32;

            win32::click_bg(hwnd, click.0, click.1);
            std::thread::sleep(Duration::from_millis(delay_ms));

            log_success(ld_num, "LOGIN BUTTON CLICKED (BACKGROUND)");
        }

        println!();
        log_system(&format!("=== DONE ALL - CLICKED LOGIN FOR {} LDS ===", count));
    }

    #[cfg(not(windows))]
    {
        log_error(0, "ONLY SUPPORTED ON WINDOWS!");
    }

    pause_and_return();
}
