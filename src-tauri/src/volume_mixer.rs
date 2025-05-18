use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

// Define the data structures for our volume mixer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSession {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub volume: f32,
    pub muted: bool,
    pub icon_path: Option<String>,
}

#[derive(Debug, Default)]
pub struct VolumeMixerState {
    pub sessions: Arc<Mutex<HashMap<u32, AudioSession>>>,
}

// Function to get a placeholder icon
fn get_app_icon_base64(_exe_path: &str) -> Option<String> {
    // Return a simple placeholder icon
    Some(String::from("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAIGNIUk0AAHolAACAgwAA+f8AAIDpAAB1MAAA6mAAADqYAAAXb5JfxUYAAABnSURBVHja7JNBCsAgDAQX8f+PtgcFEUUPIlKKJ0G8uBBCYA8zG7MiIjWzCqAiUgDM3QGYGQCqCgDufkHUzN4afKyBu8NdCz6A7Qs4EXlEHLMGMTPkRsTdH3kFe5YUEflHYQ0AAP//AwALWBHT5sqHBwAAAABJRU5ErkJggg=="))
}

// Function to get all audio sessions
#[tauri::command]
pub async fn get_audio_sessions(state: State<'_, VolumeMixerState>) -> Result<Vec<AudioSession>, String> {
    let winmix = winmix::WinMix::default();

    let sessions = unsafe {
        match winmix.enumerate() {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to enumerate audio sessions: {}", e)),
        }
    };

    let mut result = Vec::new();
    let mut sessions_map = HashMap::new();

    for session in sessions {
        let volume = unsafe {
            match session.vol.get_master_volume() {
                Ok(v) => v,
                Err(_) => 0.0,
            }
        };

        let muted = unsafe {
            match session.vol.get_mute() {
                Ok(m) => m,
                Err(_) => false,
            }
        };

        // Extract the executable name from the path and remove extension
        let mut name = session.path.split('\\').last().unwrap_or(&session.path).to_string();

        // Remove file extension (like .exe)
        if let Some(dot_pos) = name.rfind('.') {
            name = name[..dot_pos].to_string();
        }

        // Get the icon for the application
        let icon_path = get_app_icon_base64(&session.path);

        let audio_session = AudioSession {
            pid: session.pid,
            name,
            path: session.path.clone(),
            volume,
            muted,
            icon_path,
        };

        result.push(audio_session.clone());
        sessions_map.insert(session.pid, audio_session);
    }

    // Update the state
    if let Ok(mut sessions) = state.sessions.lock() {
        *sessions = sessions_map;
    }

    Ok(result)
}

// Function to set the volume for a specific application
#[tauri::command]
pub async fn set_volume(pid: u32, volume: f32, state: State<'_, VolumeMixerState>) -> Result<(), String> {
    let winmix = winmix::WinMix::default();

    let sessions = unsafe {
        match winmix.enumerate() {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to enumerate audio sessions: {}", e)),
        }
    };

    for session in sessions {
        if session.pid == pid {
            unsafe {
                if let Err(e) = session.vol.set_master_volume(volume) {
                    return Err(format!("Failed to set volume: {}", e));
                }
            }

            // Update the state
            if let Ok(mut sessions_map) = state.sessions.lock() {
                if let Some(audio_session) = sessions_map.get_mut(&pid) {
                    audio_session.volume = volume;
                }
            }

            return Ok(());
        }
    }

    Err(format!("No audio session found with PID: {}", pid))
}

// Function to mute/unmute a specific application
#[tauri::command]
pub async fn set_mute(pid: u32, mute: bool, state: State<'_, VolumeMixerState>) -> Result<(), String> {
    let winmix = winmix::WinMix::default();

    let sessions = unsafe {
        match winmix.enumerate() {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to enumerate audio sessions: {}", e)),
        }
    };

    for session in sessions {
        if session.pid == pid {
            unsafe {
                if let Err(e) = session.vol.set_mute(mute) {
                    return Err(format!("Failed to set mute: {}", e));
                }
            }

            // Update the state
            if let Ok(mut sessions_map) = state.sessions.lock() {
                if let Some(audio_session) = sessions_map.get_mut(&pid) {
                    audio_session.muted = mute;
                }
            }

            return Ok(());
        }
    }

    Err(format!("No audio session found with PID: {}", pid))
}
