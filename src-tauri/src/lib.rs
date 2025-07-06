use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rspc::Router;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

#[derive(Debug)]
struct AppState {
    counter: i32,
    counter_tx: broadcast::Sender<i32>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (counter_tx, _) = broadcast::channel(64);
    let state = Arc::new(Mutex::new(AppState {
        counter: 0,
        counter_tx: counter_tx.clone(),
    }));

    let router = Router::<Arc<Mutex<AppState>>>::new()
        .config(
            rspc::Config::new()
                .set_ts_bindings_header("/* eslint-disable */")
                .export_ts_bindings(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/lib/bindings.gen.ts"),
                ),
        )
        .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .query("greet", |t| t(|_, name: String| greet(&name)))
        .mutation("increment", |t| {
            t(|state, _: ()| {
                let mut state = state.lock().unwrap();
                state.counter += 1;
                state.counter_tx.send(state.counter).unwrap();
            })
        })
        .mutation("decrement", |t| {
            t(|state, _: ()| {
                let mut state = state.lock().unwrap();
                state.counter -= 1;
                state.counter_tx.send(state.counter).unwrap();
            })
        })
        .subscription("count", |t| {
            t(|state, _: ()| {
                let state = state.lock().unwrap();
                let rx = state.counter_tx.subscribe();
                BroadcastStream::new(rx).map(|x| x.unwrap())
            })
        })
        .build();

    tokio::spawn(async {});

    tauri::Builder::default()
        .plugin(rspc_tauri::plugin(router.arced(), move |_| state.clone()))
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
