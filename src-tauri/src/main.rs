// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    fs::{self, read_to_string, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{absolute, Path, PathBuf},
    sync::Mutex,
    vec,
};

use serde::{Deserialize, Serialize};
use tauri::{App, Manager, State};

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    profile: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SnapShot {
    index: u32,
    name: String,
    date: String,
}

impl Context {
    fn init(app: &mut App) -> Result<(), io::Error> {
        let mut context = Context {
            profile: "Default".to_string(),
        };

        context.restore()?;
        app.manage(Mutex::new(context));

        Ok(())
    }

    fn save(&self) -> Result<(), io::Error> {
        let path = Path::new("status.json");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let content = serde_json::to_string(self)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn restore(&mut self) -> Result<(), io::Error> {
        let path = Path::new("status.json");
        if path.exists() {
            let content = read_to_string(path).unwrap();
            *self = serde_json::from_str(&content).unwrap();
        } else {
            self.save()?;
        }
        Ok(())
    }

    fn set_profile(&mut self, profile: &str) -> Result<(), io::Error> {
        self.profile = profile.to_string();
        self.save()?;
        Ok(())
    }
}

const WILDFROST_DATA_PATH: &str =
    "C:/Users/$USERNAME/AppData/LocalLow/Deadpan Games/Wildfrost/Profiles";

fn get_wildfrost_data_path() -> PathBuf {
    let username = env::var("USERNAME").unwrap();
    return PathBuf::from(absolute(&WILDFROST_DATA_PATH.replace("$USERNAME", &username)).unwrap());
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[tauri::command]
fn set_profile(state: State<'_, Mutex<Context>>, profile: &str) {
    let mut state = state.lock().unwrap();
    state.set_profile(profile).unwrap();
}

#[tauri::command]
fn get_profile(state: State<'_, Mutex<Context>>) -> String {
    let state = &*state.lock().unwrap();
    return state.profile.clone();
}

#[tauri::command]
fn get_current(state: State<'_, Mutex<Context>>) -> Option<u32> {
    let profile = &*state.lock().unwrap().profile;
    let mut save_path = get_wildfrost_data_path();
    save_path.push(profile);
    save_path.push("save.meta");
    if save_path.exists() {
        let snapshot: SnapShot = serde_json::from_str(&read_to_string(save_path).unwrap()).unwrap();
        return Some(snapshot.index);
    }
    return None;
}

#[tauri::command]
fn get_snapshots(state: State<'_, Mutex<Context>>) -> Vec<SnapShot> {
    let profile = &*state.lock().unwrap().profile;
    let mut snapshot_path = PathBuf::from("saves");
    snapshot_path.push(profile);
    let dirs = fs::read_dir(&snapshot_path)
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                None
            } else {
                Some(err)
            }
        })
        .ok();

    if dirs.is_none() {
        return vec![];
    }

    let mut results = vec![];

    for (_, f) in dirs.unwrap().enumerate() {
        let meta_path = f.unwrap().path().join("save.meta");

        if !meta_path.exists() {
            continue;
        };

        let snapshot: SnapShot = serde_json::from_str(&read_to_string(meta_path).unwrap()).unwrap();
        results.push(snapshot);
    }
    return results;
}

#[tauri::command]
fn get_profiles(_: State<'_, Mutex<Context>>) -> Vec<String> {
    let save_path = get_wildfrost_data_path();
    fs::read_dir(save_path)
        .unwrap()
        .filter_map(|f| {
            let file = f.unwrap();
            if file.path().is_dir() {
                Some(file.file_name().into_string().unwrap())
            } else {
                None
            }
        })
        .collect()
}

#[tauri::command]
fn snapshot(state: State<'_, Mutex<Context>>, snapshot: SnapShot) {
    let profile = &*state.lock().unwrap().profile;
    let mut save_path = get_wildfrost_data_path();
    save_path.push(profile);

    let mut snapshot_path = PathBuf::from("saves");
    snapshot_path.push(profile);
    snapshot_path.push(&snapshot.name);
    copy_dir_all(&save_path, &snapshot_path).unwrap();

    snapshot_path.push("save.meta");
    let mut meta_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&snapshot_path)
        .unwrap();

    let content = serde_json::to_string(&snapshot).unwrap();
    meta_file.write_all(&content.as_bytes()).unwrap();
}

#[tauri::command]
fn restore(state: State<'_, Mutex<Context>>, snapshot: SnapShot) {
    let profile = &*state.lock().unwrap().profile;
    let mut save_path = get_wildfrost_data_path();
    save_path.push(profile);

    let mut snapshot_path = PathBuf::from("saves");
    snapshot_path.push(profile);
    snapshot_path.push(&snapshot.name);
    copy_dir_all(&snapshot_path, &save_path).unwrap();
}

#[tauri::command]
fn clear(state: State<'_, Mutex<Context>>, name: String) {
    let profile = &*state.lock().unwrap().profile;
    let mut snapshot_path = PathBuf::from("saves");
    snapshot_path.push(profile);
    snapshot_path.push(&name);
    fs::remove_dir_all(snapshot_path).unwrap_or_default();
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            Context::init(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_profile,
            get_profile,
            get_current,
            get_snapshots,
            get_profiles,
            snapshot,
            restore,
            clear
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
