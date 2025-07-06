use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::ipc;
use tokio::sync::{broadcast, mpsc};
use ts_rs::TS;

mod signal;
use crate::signal::{run_signal, SignalCommand, SignalEvent, SignalState};

#[derive(Debug)]
struct AppState {
    counter: i32,
    signal_command_tx: mpsc::Sender<SignalCommand>,
    signal_event: broadcast::Sender<SignalEvent>,
    signal_state_change: broadcast::Sender<()>,
    signal_state: Arc<Mutex<SignalState>>,
}

#[derive(TS, Serialize)]
#[ts(export)]
enum Event {
    SignalStateChanged(SignalState),
    SignalEvent(SignalEvent),
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
    LinkSignal(()),
}

#[derive(Serialize)]
enum CommandError {}

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
            let tx = {
                let state = state.lock().unwrap();
                state.signal_command_tx.clone()
            };
            tx.send(SignalCommand::Link { device_name }).await.unwrap();
            Ok(CommandResult::LinkSignal(()))
        }
    }
}

#[tauri::command]
async fn subscribe<'a>(
    state: tauri::State<'a, Mutex<AppState>>,
    channel: ipc::Channel<Event>,
) -> Result<(), ()> {
    let (mut signal_event, mut signal_state_change, signal_state) = {
        let state = state.lock().unwrap();
        (
            state.signal_event.subscribe(),
            state.signal_state_change.subscribe(),
            state.signal_state.clone(),
        )
    };
    let channel1 = channel.clone();
    tokio::spawn(async move {
        while let Ok(evt) = signal_event.recv().await {
            channel1.send(Event::SignalEvent(evt)).unwrap();
        }
    });
    tokio::spawn(async move {
        while let Ok(()) = signal_state_change.recv().await {
            let state = signal_state.lock().unwrap().clone();
            channel.send(Event::SignalStateChanged(state)).unwrap();
        }
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let signal_state = Arc::new(Mutex::new(SignalState::None));

    let (signal_command_tx, signal_command_rx) = tokio::sync::mpsc::channel(64);
    let (signal_event, _) = tokio::sync::broadcast::channel(64);
    let (signal_state_change, _) = tokio::sync::broadcast::channel(1);
    let signal_state_thread = signal_state.clone();
    let signal_event_thread = signal_event.clone();
    let signal_state_change_thread = signal_state_change.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            run_signal(
                signal_state_thread,
                signal_command_rx,
                signal_event_thread,
                signal_state_change_thread,
            )
            .await
            .unwrap();
        });
    });

    let app_state = Mutex::new(AppState {
        counter: 0,
        signal_command_tx,
        signal_event,
        signal_state_change,
        signal_state,
    });

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![command, subscribe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
