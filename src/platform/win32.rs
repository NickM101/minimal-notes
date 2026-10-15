use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[link(name = "user32")]
extern "system" {
    fn ShowWindow(hwnd: isize, n_cmd_show: i32) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
    fn BringWindowToTop(hwnd: isize) -> i32;
    fn LoadImageW(
        hinst: isize,
        lpszname: *const u16,
        utype: u32,
        cx: i32,
        cy: i32,
        fuload: u32,
    ) -> isize;
    fn SendMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
}

const IMAGE_ICON: u32 = 1;
const LR_LOADFROMFILE: u32 = 0x00000010;
const LR_DEFAULTSIZE: u32 = 0x00000040;
const WM_SETICON: u32 = 0x0080;
const ICON_SMALL: usize = 0;
const ICON_BIG: usize = 1;

pub fn show_and_focus_window(window: &gpui_kit::Window) {
    if let Ok(handle) = HasWindowHandle::window_handle(window) {
        if let RawWindowHandle::Win32(handle) = handle.as_raw() {
            let hwnd = handle.hwnd.get();
            unsafe {
                ShowWindow(hwnd, 5 /* SW_SHOW */);
                BringWindowToTop(hwnd);
                SetForegroundWindow(hwnd);

                // Load and apply the app icon to the window and taskbar
                let icon_path = std::path::Path::new("assests/app_icon.ico");
                if icon_path.exists() {
                    let path_wide: Vec<u16> = OsStr::new(icon_path)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();

                    let hicon_big = LoadImageW(
                        0,
                        path_wide.as_ptr(),
                        IMAGE_ICON,
                        0,
                        0,
                        LR_LOADFROMFILE | LR_DEFAULTSIZE,
                    );
                    if hicon_big != 0 {
                        SendMessageW(hwnd, WM_SETICON, ICON_BIG, hicon_big);
                    }

                    let hicon_small = LoadImageW(
                        0,
                        path_wide.as_ptr(),
                        IMAGE_ICON,
                        16,
                        16,
                        LR_LOADFROMFILE,
                    );
                    if hicon_small != 0 {
                        SendMessageW(hwnd, WM_SETICON, ICON_SMALL, hicon_small);
                    }
                }
            }
        }
    }
}
