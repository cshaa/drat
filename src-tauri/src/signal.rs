use futures_util::future;
use presage::{libsignal_service::configuration::SignalServers, Manager};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use ts_rs::TS;

#[derive(TS, Serialize, Debug, Clone)]
#[ts(export)]
pub enum SignalState {
    None,
    Linking { url: String },
    Registering,
    Connected,
}

pub enum SignalCommand {
    Link { device_name: String },
}

#[derive(TS, Clone, Debug, Serialize)]
#[ts(export)]
pub enum SignalEvent {
    LinkingCancelled,
}

fn set_state(
    state: &Arc<Mutex<SignalState>>,
    state_change_tx: broadcast::Sender<()>,
    value: SignalState,
) {
    {
        let mut state = state.lock().unwrap();
        *state = value;
    }
    state_change_tx.send(()).unwrap();
}

pub async fn run_signal(
    state: Arc<Mutex<SignalState>>,
    mut rx: mpsc::Receiver<SignalCommand>,
    event_tx: broadcast::Sender<SignalEvent>,
    state_change_tx: broadcast::Sender<()>,
) -> Result<(), anyhow::Error> {
    let sqlite_db_path = directories::ProjectDirs::from("org", "whisperfish", "presage")
        .unwrap()
        .config_dir()
        .join("cli.db3")
        .display()
        .to_string();

    let config_store = presage_store_sqlite::SqliteStore::open_with_passphrase(
        &sqlite_db_path,
        Some("secret123"),
        presage::model::identity::OnNewIdentity::Trust,
    )
    .await?;

    println!(
        "Opened DB on {} and started a config store there",
        sqlite_db_path
    );

    while let Some(cmd) = rx.recv().await {
        match cmd {
            SignalCommand::Link { device_name } => {
                let state = state.clone();
                let event_tx = event_tx.clone();
                let state_change_tx = state_change_tx.clone();
                let (provisioning_link_tx, provisioning_link_rx) =
                    futures_channel::oneshot::channel();
                let manager = future::join(
                    Manager::link_secondary_device(
                        config_store.clone(),
                        SignalServers::Staging,
                        device_name.clone(),
                        provisioning_link_tx,
                    ),
                    async move {
                        match provisioning_link_rx.await {
                            Ok(url) => {
                                set_state(
                                    &state,
                                    state_change_tx,
                                    SignalState::Linking {
                                        url: url.as_str().into(),
                                    },
                                );
                            }
                            Err(_) => {
                                event_tx.send(SignalEvent::LinkingCancelled)?;
                            }
                        }
                        Ok::<(), anyhow::Error>(())
                    },
                )
                .await;

                manager.1?;

                match manager.0 {
                    Ok(manager) => {
                        let whoami = manager.whoami().await.unwrap();
                        println!("{whoami:?}");
                    }
                    Err(err) => {
                        println!("{err:?}");
                    }
                }
            }
        }
    }

    Ok(())
}
