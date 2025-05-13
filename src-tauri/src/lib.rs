mod commands;
mod window_handler;

use crate::commands::{ send_all_configs, send_open_windows, start_config, save_config,read_configs_from_save };
use commands::AppState;
use std::{ collections::HashMap, sync::Mutex };
use uuid::Uuid;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder
        ::default()
        .manage(AppState {
            configs: Mutex::new(HashMap::new()),
            live_config: Mutex::new(Uuid::nil()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(
            tauri::generate_handler![send_open_windows, send_all_configs, start_config, save_config,read_configs_from_save]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
