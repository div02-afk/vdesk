mod window_handler;
use std::{ collections::HashMap, str::FromStr, sync::Mutex };
use serde::{ Deserialize, Serialize };
use serde_json::{ json, value::Value };
use uuid::Uuid;
use winvd::{ move_window_to_desktop, get_desktop };
use window_handler::{ create_virtual_desktop_manager, get_open_windows, launch_and_get_pid };
use std::thread;
#[derive(Deserialize, Debug, Serialize)]
pub struct WindowInfoSharable {
    pub title: String,
    pub path: String,
    pub process_id: u32,
    pub class_name: String,
    pub desktop_index: u32,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
    pub id: Uuid,
    pub data: Vec<WindowInfoSharable>,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct AppState {
    pub configs: Mutex<HashMap<Uuid, Config>>,
    pub live_config: Mutex<Uuid>,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[tauri::command]
fn send_open_windows(state: tauri::State<AppState>) -> Value {
    let desktop_manager_result = create_virtual_desktop_manager();
    match desktop_manager_result {
        Ok(desktop_manager) => {
            let open_windows = get_open_windows(&desktop_manager);

            let mut sharable_open_windows: Vec<WindowInfoSharable> = Vec::new();
            for open_window in open_windows {
                sharable_open_windows.push(WindowInfoSharable {
                    title: open_window.title,
                    path: open_window.path,
                    process_id: open_window.process_id,
                    class_name: open_window.class_name,
                    desktop_index: open_window.desktop_index,
                });
            }
            let config_id = Uuid::new_v4();
            let current_config: Config = Config {
                id: config_id,
                data: sharable_open_windows,
            };
            let response = serde_json::json!(current_config);
            let mut config_map = state.configs.lock().unwrap();
            let mut live_config = state.live_config.lock().unwrap();
            config_map.insert(config_id, current_config);
            *live_config = config_id;

            return response;
        }
        Err(_) => {
            return serde_json::json!("{}");
        }
    }
}

#[tauri::command]
fn start_config(state: tauri::State<AppState>, config_id: String) -> Value {
    println!("Starting {}", config_id);
    let config_uuid_result = Uuid::from_str(&config_id);

    let mut error_response: ErrorResponse = ErrorResponse {
        message: "".to_string(),
    };
    match config_uuid_result {
        Ok(config_uuid) => {
            let configs = state.configs.lock().unwrap();
            let config = configs.get(&config_uuid);

            if let Some(config) = config {
                let desktop_manager_result = create_virtual_desktop_manager();
                match desktop_manager_result {
                    Ok(desktop_manager) => {
                        for window in config.data.iter().clone() {
                            println!("Starting {} at {}", window.title, window.desktop_index);
                            let pid = launch_and_get_pid(&window.path);
                            let desktop = get_desktop(window.desktop_index);
                            if let Some(pid) = pid {
                                thread::sleep(std::time::Duration::from_millis(5000));
                                let open_windows = get_open_windows(&desktop_manager);
                                println!("{:?}", open_windows);
                                let hwnd = open_windows
                                    .iter()
                                    .filter_map(|window| {
                                        if window.process_id == pid {
                                            Some(window.hwnd)
                                        } else {
                                            None
                                        }
                                    })
                                    .next();

                                if let Some(hwnd) = hwnd {
                                    let window_move_result = move_window_to_desktop(desktop, &hwnd);
                                    match window_move_result {
                                        Ok(_) => {
                                            println!("Move successful");
                                        }
                                        Err(e) => {
                                            println!("{:?}", e);
                                        }
                                    }
                                } else {
                                    println!("HWND not found");
                                }
                            }
                        }
                    }

                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
            } else {
                error_response = ErrorResponse {
                    message: "No config found".to_string(),
                };
            }
        }
        Err(e) => {
            error_response = ErrorResponse {
                message: "Invalid UUid".to_string(),
            };
        }
    }

    return json!(error_response);
}

#[tauri::command]
fn send_all_configs(state: tauri::State<AppState>) -> Value {
    let configs = state.configs.lock().unwrap();
    let response = json!(*configs);
    return response;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder
        ::default()
        .manage(AppState {
            configs: Mutex::new(HashMap::new()),
            live_config: Mutex::new(Uuid::nil()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_open_windows, send_all_configs, start_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
