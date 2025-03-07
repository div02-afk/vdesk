// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::os::raw::c_void;

use windows::{
    core::{ IUnknown, Interface, BOOL, GUID, HRESULT },
    Win32::{
        Foundation::{ self, HWND, LPARAM },
        Graphics::Dwm::{ DwmGetWindowAttribute, DWMWA_CLOAKED },
        System::Com::{ CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED },
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

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
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
        // println!(
        //     "HWND: {:?}, Title: \"{}\", Class: \"{}\", PID: {}",
        //     hwnd,
        //     title,
        //     class_name,
        //     process_id
        // );.
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

        if is_gui && is_not_tool && has_size {
            println!(
                "GUI Window - HWND: {:?}, Title: \"{}\", Class: \"{}\", PID: {}",
                hwnd,
                title,
                class_name,
                process_id
            );
        }
    }

    BOOL(1) // Continue enumeration
}

fn virtual_desktop_manager() -> windows::core::Result<()> {
    println!("in");
    unsafe {
        println!("unsafe");
        let res = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if res.is_err() {
            println!("Failed");
            ();
        }
        println!("Coinit worked");
        // Create an instance of IVirtualDesktopManager
        let windows = EnumWindows(Some(enum_windows_proc), LPARAM(0));

        let clsid = GUID::from_values(
            0xaa509086,
            0x5ca9,
            0x4c25,
            [0x8f, 0x95, 0x58, 0x9d, 0x3c, 0x07, 0xb4, 0x8a]
        );

        let desktop_manager: windows::core::Result<IVirtualDesktopManager> = CoCreateInstance(
            &clsid, // CLSID of IVirtualDesktopManager
            None, // No aggregation
            CLSCTX_ALL // Create in all COM contexts
        );
        match desktop_manager {
            Ok(_) => println!("IVirtualDesktopManager created."),
            Err(e) => {
                eprintln!("CoCreateInstance failed: {:?}", e);
                return Err(e);
            }
        }
//4396A448-3963-4019-9FB8-88DD45B8F931
        let handle_value = 0xf0630;
        let toplevelwindow = HWND(handle_value as isize as *mut _);
        let t = desktop_manager.unwrap().GetWindowDesktopId(toplevelwindow);
        match t{
            Ok(r)=> println!("Guid {:?}",r),
            Err(e) => {
                eprintln!("Window Desktop failed: {:?}", e);
                return Err(e);
            }
        }
        
        println!("desktop manager created");
        match windows {
            Ok(_) => println!("Successfully enumerated windows."),
            Err(e) => eprintln!("EnumWindows failed: {:?}", e),
        }
        // windows::Win32::System::Com::CoUninitialize();
        Ok(())
    }
}

fn main() {
    println!("Architecture: {:?}", std::env::consts::ARCH);

    let _ = virtual_desktop_manager();
    vdesk_lib::run();
}
