use std::{sync::Mutex, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use ts_rs::TS;

#[derive(Debug)]
struct AppState {
    counter: i32,
}

#[derive(TS, Deserialize)]
#[ts(export)]
enum Command {
    Greet(String),
    Increment,
    Decrement,
    Sleep(u32),
}

#[derive(TS, Serialize)]
#[ts(export)]
enum CommandResult {
    Greet(String),
    Increment(i32),
    Decrement(i32),
    Sleep(()),
}

#[tauri::command]
async fn command<'a>(
    state: tauri::State<'a, Mutex<AppState>>,
    which: Command,
) -> Result<CommandResult, ()> {
    match which {
        Command::Greet(s) => Ok(CommandResult::Greet(format!(
            "Hello, {}, you have been greeted from Rust!",
            s
        ))),
        Command::Increment => {
            let mut state = state.lock().unwrap();
            state.counter += 1;
            Ok(CommandResult::Increment(state.counter))
        }
        Command::Decrement => {
            let mut state = state.lock().unwrap();
            state.counter -= 1;
            Ok(CommandResult::Decrement(state.counter))
        }
        Command::Sleep(ms) => {
            tokio::time::sleep(Duration::from_millis(ms.into())).await;
            Ok(CommandResult::Sleep(()))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Mutex::new(AppState { counter: 0 });

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
