use crate::window_handler::{ create_virtual_desktop_manager, get_open_windows, launch_and_get_pid };
use serde::{ Deserialize, Serialize };
use serde_json::to_writer_pretty;
use serde_json::{ json, value::Value };
use tauri::path::BaseDirectory;
use tauri::App;
use std::fs::{ DirBuilder, File, OpenOptions };
use std::io::{ Error, ErrorKind, Read, Seek, Write };
use std::path::PathBuf;
use std::thread;
use std::{ collections::HashMap, str::FromStr, sync::Mutex };
use uuid::Uuid;
use winvd::{ get_desktop, move_window_to_desktop };

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

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct AppState {
    pub configs: Mutex<HashMap<Uuid, Config>>,
    pub live_config: Mutex<Uuid>,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[tauri::command]
pub fn send_open_windows(state: tauri::State<AppState>) -> Value {
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
pub fn start_config(state: tauri::State<AppState>, config_id: String) -> Value {
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
pub fn send_all_configs(state: tauri::State<AppState>) -> Value {
    let configs = state.configs.lock().unwrap();
    let response = json!(*configs);
    return response;
}

fn create_and_open_config_save() -> Result<File, Error> {
    let app_data = std::env
        ::var("APPDATA")
        .map_err(|_| {
            Error::new(ErrorKind::NotFound, "APPDATA environment variable not found")
        })?;

    let mut path = PathBuf::from(app_data);
    path.push("vdesk.json");

    println!("{:?}", path);
    let file = OpenOptions::new().read(true).write(true).open(&path);
    file.or_else(|err| {
        if err.kind() == ErrorKind::NotFound {
            File::create(&path).or_else(|err| {
                println!("{}", err);
                Err(err)
            })
        } else {
            Err(err)
        }
    })
}

fn create_new_config_save() -> Result<File, Error> {
    let app_data = std::env
        ::var("APPDATA")
        .map_err(|_| {
            Error::new(ErrorKind::NotFound, "APPDATA environment variable not found")
        })?;

    let mut path = PathBuf::from(app_data);
    path.push("vdesk.json");

    println!("{:?}", path);
    File::create(&path).or_else(|err| {
        println!("{}", err);
        Err(err)
    })
}

#[tauri::command]
pub fn save_config(state: tauri::State<AppState>, config_id: String) -> Value {
    println!("saving {}", config_id);
    let config_uuid_result = Uuid::from_str(&config_id);

    let mut error_response: ErrorResponse = ErrorResponse {
        message: "".to_string(),
    };
    match config_uuid_result {
        Ok(config_uuid) => {
            let config_save_result = create_new_config_save();
            if config_save_result.is_err() {
                println!("Panic");
                return json!(error_response);
            }
            let config_save = config_save_result.unwrap();

            let serialized_config = to_writer_pretty(&config_save, &*state);
            match serialized_config {
                Ok(_) => {
                    println!("saved");
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        }
        Err(e) => {
            error_response.message = "Invalid uuid".to_string();
        }
    }

    return json!(error_response);
}

#[tauri::command]
pub fn read_configs_from_save(state: tauri::State<AppState>) -> Value {
    println!("reading config");
    let mut error_response: ErrorResponse = ErrorResponse {
        message: "".to_string(),
    };
    let config_save_result = create_and_open_config_save();
    if config_save_result.is_err() {
        println!("Panic");
        return json!(error_response);
    }
    let config_save = config_save_result.unwrap();

    let saved_configs_result: Result<AppState, serde_json::Error> = serde_json::from_reader(
        config_save
    );

    match saved_configs_result {
        Ok(saved_configs) => {
            println!("{:?}", saved_configs);
        }
        Err(e) => {
            println!("{:?}", e);
            error_response.message = "Can't read from config".to_string();
        }
    }
    return json!(error_response);
}
