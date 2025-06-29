use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use ts_rs::TS;

use crate::signal::{run_signal, SignalCommand, SignalEvent, SignalState};

mod signal;

#[derive(Debug)]
struct AppState {
    counter: i32,
    signal_command_tx: mpsc::Sender<SignalCommand>,
    signal_event_tx: broadcast::Sender<SignalEvent>,
}

#[derive(TS, Deserialize)]
#[ts(export)]
#[serde(rename_all_fields = "camelCase")]
enum Command {
    Greet(String),
    Increment,
    Decrement,
    Sleep(u32),
    LinkSignal { device_name: String },
}

#[derive(TS, Serialize)]
#[ts(export)]
enum CommandResult {
    Greet(String),
    Increment(i32),
    Decrement(i32),
    Sleep(()),
    LinkSignal { url: String },
}

#[derive(Serialize)]
enum CommandError {
    ConnectionBroke,
    Cancelled,
}

#[tauri::command]
async fn command<'a>(
    state: tauri::State<'a, Mutex<AppState>>,
    which: Command,
) -> Result<CommandResult, CommandError> {
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
        Command::LinkSignal { device_name } => {
            let (tx, mut rx) = {
                let state = state.lock().unwrap();
                (
                    state.signal_command_tx.clone(),
                    state.signal_event_tx.subscribe(),
                )
            };
            tx.send(SignalCommand::Link { device_name }).await.unwrap();
            while let Ok(evt) = rx.recv().await {
                match evt {
                    SignalEvent::LinkingUrlAvailable(url) => {
                        return Ok(CommandResult::LinkSignal { url });
                    }
                    SignalEvent::LinkingCancelled => return Err(CommandError::Cancelled),
                }
            }
            Err(CommandError::ConnectionBroke)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let signal_state = Arc::new(Mutex::new(SignalState::None));

    let (signal_command_tx, signal_command_rx) = tokio::sync::mpsc::channel(64);
    let (signal_event_tx, mut signal_event_rx) = tokio::sync::broadcast::channel(64);
    let signal_event_tx_thread = signal_event_tx.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            run_signal(signal_state, signal_command_rx, signal_event_tx_thread)
                .await
                .unwrap();
        });
    });

    tokio::spawn(async move {
        while let Ok(evt) = signal_event_rx.recv().await {
            println!("Signal Event: {:?}", evt);
        }
    });

    let app_state = Mutex::new(AppState {
        counter: 0,
        signal_command_tx,
        signal_event_tx,
    });

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
