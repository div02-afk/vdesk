use std::{ ffi::OsString, os::windows::ffi::OsStringExt, process::Command };

use windows::{
    core::{ Error, GUID, PWSTR },
    Win32::{
        Foundation::{ self, BOOL, HANDLE, HWND, LPARAM, MAX_PATH },
        Graphics::Dwm::{ DwmGetWindowAttribute, DWMWA_CLOAKED },
        System::{
            Com::{ CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED },
            Threading::{
                OpenProcess,
                QueryFullProcessImageNameW,
                PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Shell::IVirtualDesktopManager,
            WindowsAndMessaging::{
                EnumWindows,
                GetClassNameW,
                GetWindowLongW,
                GetWindowRect,
                GetWindowTextW,
                GetWindowThreadProcessId,
                IsWindowVisible,
                GWL_EXSTYLE,
                GWL_STYLE,
                WS_EX_TOOLWINDOW,
                WS_OVERLAPPEDWINDOW,
                WS_POPUP,
            },
        },
    },
};
use winvd::get_desktop_by_window;

#[derive(Debug)]
pub struct WindowInfo {
    pub hwnd: HWND,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub desktop_id: GUID,
    pub path: String,
    pub desktop_index: u32,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let open_windows = (lparam.0 as *mut Vec<WindowInfo>).as_mut();
    if let Some(open_windows) = open_windows {
        let mut buffer = [0u16; 512]; // Buffer to store window title
        // Get window title
        let length = GetWindowTextW(hwnd, &mut buffer);
        let title = String::from_utf16_lossy(&buffer[..length as usize]);

        // Get window class name
        let class_length = GetClassNameW(hwnd, &mut buffer);
        let class_name = String::from_utf16_lossy(&buffer[..class_length as usize]);

        // Get Process ID of the window
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Check if window is visible
        if !title.is_empty() && IsWindowVisible(hwnd).as_bool() {
            let style = GetWindowLongW(hwnd, GWL_STYLE);

            // Check if it has typical GUI window styles
            let is_gui =
                (style & (WS_OVERLAPPEDWINDOW.0 as i32)) != 0 || (style & (WS_POPUP.0 as i32)) != 0;

            // Check if it's not a tool window or other special windows
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let is_not_tool = (ex_style & (WS_EX_TOOLWINDOW.0 as i32)) == 0;
            let mut rect = Foundation::RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect);
            let has_size = rect.right - rect.left > 0 && rect.bottom - rect.top > 0;

            // Check if window is cloaked (hidden but maintained by the system)
            let mut cloaked: BOOL = BOOL(0);
            let cloaked_check = DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut BOOL as *mut _,
                std::mem::size_of::<BOOL>() as u32
            );
            //use this if only current desktop's apps needed
            let not_cloaked = cloaked_check.is_err() || !cloaked.as_bool();
            let path = get_executable_path_from_pid(process_id).unwrap_or_default();
            if is_gui && is_not_tool && has_size {
                let desktop_result = get_desktop_by_window(hwnd.clone());
                let desktop_id: GUID;
                let desktop_index: u32;
                match desktop_result {
                    Ok(desktop_clean) => {
                        desktop_id = desktop_clean.get_id().unwrap_or_default();
                        desktop_index = desktop_clean.get_index().unwrap_or_default();
                    }
                    Err(e) => {
                        println!("{} {:?}", title, e);
                        desktop_id = GUID::zeroed();
                        desktop_index = 0;
                    }
                }
                open_windows.push(WindowInfo {
                    hwnd,
                    title,
                    class_name,
                    process_id,
                    desktop_id,
                    path,
                    desktop_index,
                });
                // println!(
                //     "GUI Window - HWND: {:?}, Title: \"{}\", Class: \"{}\", PID: {}",
                //     hwnd,
                //     title,
                //     class_name,
                //     process_id
                // );
            }
        }
    }
    BOOL(1) // Continue enumeration
}

pub fn get_window_desktop_id(
    handle_value: &HWND,
    desktop_manager: &IVirtualDesktopManager
) -> Result<GUID, Error> {
    // let toplevelwindow = HWND(handle_value as isize as *mut _);
    unsafe {
        let window_desktop_id_result = desktop_manager.GetWindowDesktopId(*handle_value);
        match window_desktop_id_result {
            Ok(window_desktop_id) => {
                return Ok(window_desktop_id);
            }
            Err(_) => {
                return Ok(GUID::zeroed());
            }
        }
    }
}

pub fn get_open_windows(desktop_manager: &IVirtualDesktopManager) -> Vec<WindowInfo> {
    unsafe {
        let mut open_windows: Vec<WindowInfo> = Vec::new();
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut open_windows as *mut _ as isize));

        // println!("{:?}", &open_windows);
        return open_windows;
    }
}

pub fn create_virtual_desktop_manager() -> Result<IVirtualDesktopManager, Error> {
    unsafe {
        let res = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if res.is_err() {
            println!("Failed");
            ();
        }

        let clsid = GUID::from_values(
            0xaa509086,
            0x5ca9,
            0x4c25,
            [0x8f, 0x95, 0x58, 0x9d, 0x3c, 0x07, 0xb4, 0x8a]
        );

        // Create an instance of IVirtualDesktopManager
        let desktop_manager_result: windows::core::Result<IVirtualDesktopManager> = CoCreateInstance(
            &clsid, // CLSID of IVirtualDesktopManager
            None, // No aggregation
            CLSCTX_ALL // Create in all COM contexts
        );
        match desktop_manager_result {
            Ok(_) => {
                return Ok(desktop_manager_result.unwrap());
            }
            Err(e) => {
                eprintln!("CoCreateInstance failed: {:?}", e);
                return Err(e);
            }
        }
    }
}

pub fn get_executable_path_from_hwnd(hwnd: HWND) -> Result<String, Error> {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }

    if pid == 0 {
        println!("pid 0");
        return Err(Error::empty());
    }

    get_executable_path_from_pid(pid)
}

pub fn get_executable_path_from_pid(pid: u32) -> Result<String, Error> {
    unsafe {
        let h_process_result = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        let h_process: HANDLE;
        match h_process_result {
            Ok(h) => {
                h_process = h;
            }
            Err(e) => {
                return Err(e);
            }
        }

        let mut buffer = [0u16; MAX_PATH as usize];
        let mut size = buffer.len() as u32;

        let success = QueryFullProcessImageNameW(
            h_process,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size
        );

        match success {
            Ok(_) => {
                let os_string = OsString::from_wide(&buffer[..size as usize]);
                let path = os_string.to_string_lossy().into_owned();
                // println!("Executable Path: {}", path);
                Ok(path)
            }
            Err(e) => { Err(e) }
        }
    }
}

pub fn launch_and_get_pid(path: &str) -> Option<u32> {
    match Command::new(path).spawn() {
        Ok(child) => {
            println!("Spawned process with PID: {}", child.id());
            Some(child.id())
        }
        Err(e) => {
            eprintln!("Failed to spawn: {:?}", e);
            None
        }
    }
}

pub fn move_window_to_desktop(
    desktop_manager: &IVirtualDesktopManager,
    handle: &HWND,
    desktop_id: &GUID
) -> Result<(), Error> {
    unsafe {
        return desktop_manager.MoveWindowToDesktop(*handle, desktop_id as *const _);
    }
}
