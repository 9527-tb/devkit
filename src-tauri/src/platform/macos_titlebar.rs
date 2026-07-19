//! macOS Overlay 标题栏：将原生红绿灯垂直居中到自定义标题栏高度内。

#![cfg(target_os = "macos")]

use objc2::MainThreadMarker;
use objc2_app_kit::{NSButton, NSWindow, NSWindowButton};
use objc2_foundation::NSPoint;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Manager, Runtime, Window};

/// 与前端 `.app-titlebar.is-mac { --titlebar-h: 41px }` 保持一致。
const TITLEBAR_HEIGHT: f64 = 41.0;
/// 关闭按钮左侧边距。
const TRAFFIC_LIGHT_X: f64 = 14.0;
/// 红绿灯水平间距（固定值，避免拖拽缩放时读 frame 抖动）。
const TRAFFIC_LIGHT_GAP: f64 = 20.0;
/// 连续 resize 时合并定位，避免红绿灯跳动。
const RESIZE_DEBOUNCE: Duration = Duration::from_millis(80);

static LAST_APPLY: Mutex<Option<Instant>> = Mutex::new(None);
static PENDING: Mutex<bool> = Mutex::new(false);

pub fn apply_main_traffic_lights<R: Runtime>(app: &impl Manager<R>) {
    if let Some(win) = app.get_webview_window("main") {
        apply_traffic_lights_ns(win.ns_window().ok());
    }
}

pub fn apply_traffic_lights_for_window(window: &Window) {
    if window.label() != "main" {
        return;
    }
    apply_traffic_lights_ns(window.ns_window().ok());
}

/// 窗口缩放中调用：防抖后再定位，避免红绿灯随拖拽跳动。
pub fn schedule_traffic_lights_for_window(window: &Window) {
    if window.label() != "main" {
        return;
    }
    {
        let mut pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
        if *pending {
            return;
        }
        *pending = true;
    }

    let label = window.label().to_string();
    let handle = window.app_handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(RESIZE_DEBOUNCE);
        let app = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            if let Ok(mut pending) = PENDING.lock() {
                *pending = false;
            }
            if let Some(win) = app.get_webview_window(&label) {
                apply_traffic_lights_ns(win.ns_window().ok());
            }
        });
    });
}

fn apply_traffic_lights_ns(ns: Option<*mut std::ffi::c_void>) {
    let Some(ptr) = ns.filter(|p| !p.is_null()) else {
        return;
    };
    if let Ok(mut last) = LAST_APPLY.lock() {
        if let Some(t) = *last {
            if t.elapsed() < Duration::from_millis(16) {
                return;
            }
        }
        *last = Some(Instant::now());
    }
    // SAFETY: AppKit 需主线程；ns_window 指向存活的 NSWindow。
    unsafe {
        center_traffic_lights(ptr, TRAFFIC_LIGHT_X, TITLEBAR_HEIGHT);
    }
}

/// 将 titlebar 容器高度设为自定义标题栏高度，并把红绿灯在容器内垂直居中。
unsafe fn center_traffic_lights(
    ns_window_ptr: *mut std::ffi::c_void,
    x: f64,
    titlebar_height: f64,
) {
    let Some(_mtm) = MainThreadMarker::new() else {
        return;
    };
    let window = &*(ns_window_ptr as *const NSWindow);

    // 全屏由系统管理标题栏，勿强改 frame（避免进出全屏时跳动）
    if window.styleMask().contains(objc2_app_kit::NSWindowStyleMask::FullScreen) {
        return;
    }

    let Some(close) = window.standardWindowButton(NSWindowButton::CloseButton) else {
        return;
    };
    let Some(miniaturize) = window.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
        return;
    };
    let Some(zoom) = window.standardWindowButton(NSWindowButton::ZoomButton) else {
        return;
    };

    let Some(button_superview) = close.superview() else {
        return;
    };
    let Some(title_bar_container) = button_superview.superview() else {
        return;
    };

    let button_h = close.frame().size.height;
    let win_h = window.frame().size.height;

    // 容器贴窗口顶部，高度与 HTML 标题栏一致
    let mut title_bar_rect = title_bar_container.frame();
    let target_y = win_h - titlebar_height;
    if (title_bar_rect.size.height - titlebar_height).abs() > 0.5
        || (title_bar_rect.origin.y - target_y).abs() > 0.5
    {
        title_bar_rect.size.height = titlebar_height;
        title_bar_rect.origin.y = target_y;
        title_bar_container.setFrame(title_bar_rect);
    }

    // Cocoa 坐标系原点在左下：在容器内垂直居中
    let button_y = ((titlebar_height - button_h) / 2.0).max(0.0);
    let buttons: [&NSButton; 3] = [close.as_ref(), miniaturize.as_ref(), zoom.as_ref()];
    for (i, button) in buttons.into_iter().enumerate() {
        let next = NSPoint {
            x: x + (i as f64 * TRAFFIC_LIGHT_GAP),
            y: button_y,
        };
        let cur = button.frame().origin;
        if (cur.x - next.x).abs() > 0.5 || (cur.y - next.y).abs() > 0.5 {
            button.setFrameOrigin(next);
        }
    }
}
