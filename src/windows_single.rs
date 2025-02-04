// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{SingleInstanceCallback, ID};
use std::{ffi::CStr, sync::OnceLock};

use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM},
    System::{
        DataExchange::COPYDATASTRUCT,
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateMutexW, ReleaseMutex},
    },
    UI::WindowsAndMessaging::{
        self as w32wm, CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowW,
        RegisterClassExW, SendMessageW, GWL_STYLE, GWL_USERDATA, WINDOW_LONG_PTR_INDEX,
        WM_COPYDATA, WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TRANSPARENT, WS_OVERLAPPED, WS_POPUP, WS_VISIBLE,
    },
};

static MUTEX_HANDLE: OnceLock<MutexHandle> = OnceLock::new();
static TARGET_WINDOW_HANDLE: OnceLock<TargetWindowHandle> = OnceLock::new();

#[derive(Debug)]
struct MutexHandle(isize);
#[derive(Debug)]
struct TargetWindowHandle(isize);

const WMCOPYDATA_SINGLE_INSTANCE_DATA: usize = 1542;

pub fn init(f: Box<SingleInstanceCallback>) {
    #[allow(unused_mut)]
    let id = ID.get().expect("register() called before prepare()");

    let class_name = encode_wide(format!("{id}-sic"));
    let window_name = encode_wide(format!("{id}-siw"));
    let mutex_name = encode_wide(format!("{id}-sim"));

    let hmutex = unsafe { CreateMutexW(std::ptr::null(), true.into(), mutex_name.as_ptr()) };

    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            let hwnd = FindWindowW(class_name.as_ptr(), window_name.as_ptr());

            if !hwnd.is_null() {
                let data = format!(
                    "{}|{}\0",
                    std::env::current_dir()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                    std::env::args().collect::<Vec<String>>().join("|")
                );
                let bytes = data.as_bytes();
                let cds = COPYDATASTRUCT {
                    dwData: WMCOPYDATA_SINGLE_INSTANCE_DATA,
                    cbData: bytes.len() as _,
                    lpData: bytes.as_ptr() as _,
                };
                SendMessageW(hwnd, WM_COPYDATA, 0, &cds as *const _ as _);
                // app.cleanup_before_exit();
                std::process::exit(0);
            }
        }
    } else {
        MUTEX_HANDLE.set(MutexHandle(hmutex as _)).unwrap();

        let hwnd = create_event_target_window(&class_name, &window_name);
        unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, Box::into_raw(Box::new(f)) as _) };

        TARGET_WINDOW_HANDLE
            .set(TargetWindowHandle(hwnd as _))
            .unwrap();
    }
}

// pub fn destroy() {
//     if let Some(hmutex) = MUTEX_HANDLE.get() {
//         unsafe {
//             ReleaseMutex(hmutex.0 as _);
//             CloseHandle(hmutex.0 as _);
//         }
//     }
//     if let Some(hwnd) = TARGET_WINDOW_HANDLE.get() {
//         unsafe { DestroyWindow(hwnd.0 as _) };
//     }
// }

unsafe extern "system" fn single_instance_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let data_ptr = GetWindowLongPtrW(hwnd, GWL_USERDATA) as *mut (Box<SingleInstanceCallback>);
    let callback = &mut *data_ptr;

    match msg {
        WM_COPYDATA => {
            let cds_ptr = lparam as *const COPYDATASTRUCT;
            if (*cds_ptr).dwData == WMCOPYDATA_SINGLE_INSTANCE_DATA {
                let data = CStr::from_ptr((*cds_ptr).lpData as _).to_string_lossy();
                let mut s = data.split('|');
                let cwd = s.next().unwrap();
                let args = s.map(|s| s.to_string()).collect();
                callback(args, cwd.to_string());
            }
            1
        }

        WM_DESTROY => {
            let _ = Box::from_raw(data_ptr);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create_event_target_window(class_name: &[u16], window_name: &[u16]) -> HWND {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(single_instance_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(std::ptr::null()),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        RegisterClassExW(&class);

        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE
            | WS_EX_TRANSPARENT
            | WS_EX_LAYERED
            // WS_EX_TOOLWINDOW prevents this window from ever showing up in the taskbar, which
            // we want to avoid. If you remove this style, this window won't show up in the
            // taskbar *initially*, but it can show up at some later point. This can sometimes
            // happen on its own after several hours have passed, although this has proven
            // difficult to reproduce. Alternatively, it can be manually triggered by killing
            // `explorer.exe` and then starting the process back up.
            // It is unclear why the bug is triggered by waiting for several hours.
            | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            GetModuleHandleW(std::ptr::null()),
            std::ptr::null(),
        );
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            // The window technically has to be visible to receive WM_PAINT messages (which are used
            // for delivering events during resizes), but it isn't displayed to the user because of
            // the LAYERED style.
            (WS_VISIBLE | WS_POPUP) as isize,
        );
        hwnd
    }
}

pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    std::os::windows::prelude::OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    w32wm::SetWindowLongW(hwnd, index, value as _) as _
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    w32wm::SetWindowLongPtrW(hwnd, index, value)
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    w32wm::GetWindowLongW(hwnd, index) as _
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    w32wm::GetWindowLongPtrW(hwnd, index)
}
