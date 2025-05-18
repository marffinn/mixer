// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;

mod volume_mixer;
use volume_mixer::{get_audio_sessions, set_mute, set_volume, VolumeMixerState};

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn resize_window(window: tauri::Window, height: f64) {
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: 250,
        height: height as u32,
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(VolumeMixerState {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
        .invoke_handler(tauri::generate_handler![
            get_audio_sessions,
            set_volume,
            set_mute,
            close_window,
            resize_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
