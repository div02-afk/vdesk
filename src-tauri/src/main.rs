// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window_handler;
use window_handler::create_virtual_desktop_manager;

fn main() {
    println!("Architecture: {:?}", std::env::consts::ARCH);

    let desktop_manager_result = create_virtual_desktop_manager();
    match desktop_manager_result {
        Ok(desktop_manager) => {
            vdesk_lib::run();
        }
        Err(_) => {}
    }
}
