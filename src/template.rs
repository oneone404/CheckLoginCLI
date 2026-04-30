use std::collections::HashMap;
use std::fs;
use image::{DynamicImage, GenericImageView};
use crate::types::Roi;
use crate::config::{get_template_dir, get_roi_config_path};
use crate::state::ROI_CACHE;
use crate::utils::silent_command;
use crate::adb::adb_tap;

pub fn get_roi_config() -> &'static HashMap<String, Roi> {
    ROI_CACHE.get_or_init(|| {
        let mut roi_map = HashMap::new();
        let path = get_roi_config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object() {
                    for (key, value) in obj {
                        if key.starts_with("_") { continue; }
                        if let Some(arr) = value.as_array() {
                            if arr.len() == 4 {
                                let x1 = arr[0].as_u64().unwrap_or(0) as u32;
                                let y1 = arr[1].as_u64().unwrap_or(0) as u32;
                                let x2 = arr[2].as_u64().unwrap_or(960) as u32;
                                let y2 = arr[3].as_u64().unwrap_or(540) as u32;
                                roi_map.insert(key.clone(), Roi { x1, y1, x2, y2 });
                            }
                        }
                    }
                }
            }
        }
        roi_map
    })
}

pub fn load_template(template_name: &str) -> Option<DynamicImage> {
    let template_path = format!("{}\\{}", get_template_dir(), template_name);

    if !std::path::Path::new(&template_path).exists() {
        return None;
    }

    image::open(&template_path).ok()
}

pub fn is_pixel_match(screen: &DynamicImage, template: &DynamicImage, sx: u32, sy: u32, tx: u32, ty: u32) -> bool {
    let p1 = screen.get_pixel(sx + tx, sy + ty);
    let p2 = template.get_pixel(tx, ty);

    if p2.0[3] < 10 { return true; }

    let r_diff = (p1.0[0] as i32 - p2.0[0] as i32).abs();
    let g_diff = (p1.0[1] as i32 - p2.0[1] as i32).abs();
    let b_diff = (p1.0[2] as i32 - p2.0[2] as i32).abs();

    r_diff < 40 && g_diff < 40 && b_diff < 40
}

pub fn find_templates_batch(adb_serial: &str, template_names: &[&str]) -> HashMap<String, bool> {
    let mut results: HashMap<String, bool> = HashMap::new();

    let _ = silent_command("adb")
        .args(["-s", adb_serial, "shell", "screencap", "-p", "/sdcard/cl_check.png"])
        .output();

    let temp_path = std::env::temp_dir().join(format!("cl_check_{}.png", adb_serial.replace(":", "_")));
    let _ = silent_command("adb")
        .args(["-s", adb_serial, "pull", "/sdcard/cl_check.png", temp_path.to_str().unwrap_or("cl_check.png")])
        .output();

    let _ = silent_command("adb")
        .args(["-s", adb_serial, "shell", "rm", "-f", "/sdcard/cl_check.png"])
        .output();

    let screen_img = match image::open(&temp_path) {
        Ok(img) => img,
        Err(_) => {
            let _ = fs::remove_file(&temp_path);
            for name in template_names {
                results.insert(name.to_string(), false);
            }
            return results;
        }
    };

    let (sw, sh) = screen_img.dimensions();
    let roi_map = get_roi_config();

    for template_name in template_names {
        let template_img = match load_template(template_name) {
            Some(img) => img,
            None => {
                results.insert(template_name.to_string(), false);
                continue;
            }
        };

        let (tw, th) = template_img.dimensions();
        if tw > sw || th > sh {
            drop(template_img);
            results.insert(template_name.to_string(), false);
            continue;
        }

        let (scan_x1, scan_y1, scan_x2, scan_y2) = if let Some(roi) = roi_map.get(*template_name) {
            let scale_x = sw as f32 / 960.0;
            let scale_y = sh as f32 / 540.0;
            let x1 = ((roi.x1 as f32 * scale_x) as u32).min(sw.saturating_sub(tw));
            let y1 = ((roi.y1 as f32 * scale_y) as u32).min(sh.saturating_sub(th));
            let x2 = ((roi.x2 as f32 * scale_x) as u32).min(sw.saturating_sub(tw));
            let y2 = ((roi.y2 as f32 * scale_y) as u32).min(sh.saturating_sub(th));
            (x1, y1, x2, y2)
        } else {
            (0, 0, sw.saturating_sub(tw), sh.saturating_sub(th))
        };

        let cx = tw / 2;
        let cy = th / 2;
        let mut found = false;

        const SAMPLES: u32 = 6;
        const MATCH_THRESHOLD: i32 = 27;

        'outer: for y in scan_y1..=scan_y2 {
            for x in scan_x1..=scan_x2 {
                if is_pixel_match(&screen_img, &template_img, x, y, cx, cy) {
                    let mut match_count = 0;
                    for i in 0..SAMPLES {
                        for j in 0..SAMPLES {
                            let tx = i * (tw - 1) / (SAMPLES - 1);
                            let ty = j * (th - 1) / (SAMPLES - 1);
                            if is_pixel_match(&screen_img, &template_img, x, y, tx, ty) {
                                match_count += 1;
                            }
                        }
                    }
                    if match_count >= MATCH_THRESHOLD {
                        found = true;
                        break 'outer;
                    }
                }
            }
        }

        drop(template_img);
        results.insert(template_name.to_string(), found);
    }

    drop(screen_img);
    let _ = fs::remove_file(&temp_path);

    results
}

pub fn find_and_click_template(adb_serial: &str, template_name: &str) -> bool {
    let _ = silent_command("adb")
        .args(["-s", adb_serial, "shell", "screencap", "-p", "/sdcard/cl_click.png"])
        .output();

    let temp_path = std::env::temp_dir().join(format!("cl_click_{}.png", adb_serial.replace(":", "_")));
    let _ = silent_command("adb")
        .args(["-s", adb_serial, "pull", "/sdcard/cl_click.png", temp_path.to_str().unwrap_or("cl_click.png")])
        .output();

    let _ = silent_command("adb")
        .args(["-s", adb_serial, "shell", "rm", "-f", "/sdcard/cl_click.png"])
        .output();

    let screen_img = match image::open(&temp_path) {
        Ok(img) => img,
        Err(_) => {
            let _ = fs::remove_file(&temp_path);
            return false;
        }
    };

    let template_img = match load_template(template_name) {
        Some(img) => img,
        None => {
            drop(screen_img);
            let _ = fs::remove_file(&temp_path);
            return false;
        }
    };

    let (sw, sh) = screen_img.dimensions();
    let (tw, th) = template_img.dimensions();

    if tw > sw || th > sh {
        drop(template_img);
        drop(screen_img);
        let _ = fs::remove_file(&temp_path);
        return false;
    }

    let roi_map = get_roi_config();
    let (scan_x1, scan_y1, scan_x2, scan_y2) = if let Some(roi) = roi_map.get(template_name) {
        let scale_x = sw as f32 / 960.0;
        let scale_y = sh as f32 / 540.0;
        let x1 = ((roi.x1 as f32 * scale_x) as u32).min(sw.saturating_sub(tw));
        let y1 = ((roi.y1 as f32 * scale_y) as u32).min(sh.saturating_sub(th));
        let x2 = ((roi.x2 as f32 * scale_x) as u32).min(sw.saturating_sub(tw));
        let y2 = ((roi.y2 as f32 * scale_y) as u32).min(sh.saturating_sub(th));
        (x1, y1, x2, y2)
    } else {
        (0, 0, sw.saturating_sub(tw), sh.saturating_sub(th))
    };

    let cx = tw / 2;
    let cy = th / 2;

    const SAMPLES: u32 = 6;
    const MATCH_THRESHOLD: i32 = 25;

    for y in (scan_y1..=scan_y2).step_by(3) {
        for x in (scan_x1..=scan_x2).step_by(3) {
            if is_pixel_match(&screen_img, &template_img, x, y, cx, cy) {
                let mut match_count = 0;
                for i in 0..SAMPLES {
                    for j in 0..SAMPLES {
                        let tx = i * (tw - 1) / (SAMPLES - 1);
                        let ty = j * (th - 1) / (SAMPLES - 1);
                        if is_pixel_match(&screen_img, &template_img, x, y, tx, ty) {
                            match_count += 1;
                        }
                    }
                }
                if match_count >= MATCH_THRESHOLD {
                    let click_x = (x + tw / 2) as i32;
                    let click_y = (y + th / 2) as i32;

                    drop(template_img);
                    drop(screen_img);
                    let _ = fs::remove_file(&temp_path);

                    adb_tap(adb_serial, click_x, click_y);
                    return true;
                }
            }
        }
    }

    drop(template_img);
    drop(screen_img);
    let _ = fs::remove_file(&temp_path);
    false
}
