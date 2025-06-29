use futures_util::future;
use presage::{libsignal_service::configuration::SignalServers, Manager};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};

pub enum SignalState {
    None,
    Linking,
    Registering,
    Connected,
}

pub enum SignalCommand {
    Link { device_name: String },
}

#[derive(Clone, Debug)]
pub enum SignalEvent {
    LinkingUrlAvailable(String),
    LinkingCancelled,
}

pub async fn run_signal(
    state: Arc<Mutex<SignalState>>,
    mut rx: mpsc::Receiver<SignalCommand>,
    tx: broadcast::Sender<SignalEvent>,
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
                let tx = tx.clone();
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
                                tx.send(SignalEvent::LinkingUrlAvailable(url.as_str().into()))?;
                            }
                            Err(_) => {
                                tx.send(SignalEvent::LinkingCancelled)?;
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
