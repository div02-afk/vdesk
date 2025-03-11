mod window_handler;

use window_handler::{ create_virtual_desktop_manager, get_open_windows };
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn send_open_windows() -> String {
    let desktop_manager_result = create_virtual_desktop_manager();
    match desktop_manager_result {
        Ok(desktop_manager) => {
            let open_windows = get_open_windows(&desktop_manager);
            format!("{:?}",open_windows)
        }
        Err(_) => {
            return format!("Error");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, send_open_windows])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
