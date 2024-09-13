use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::async_runtime;
use tauri::Manager;

use xs::store::{FollowOption, Frame, ReadOptions, Store};

struct SharedState {
    open_channels: Arc<Mutex<HashMap<u32, tokio::sync::oneshot::Sender<()>>>>,
    store: Store,
}

#[tauri::command]
async fn stream_items(
    state: tauri::State<'_, SharedState>,
    on_event: tauri::ipc::Channel<Frame>,
) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    let open_channels = state.open_channels.clone();
    open_channels.lock().unwrap().insert(on_event.id(), tx);

    let store = state.store.clone();

    async_runtime::spawn(async move {
        let options = ReadOptions::builder().follow(FollowOption::On).build();
        let mut recver = store.read(options).await;
        loop {
            tokio::select! {
                Some(frame) = recver.recv() => {
                    if let Err(e) = on_event.send(frame) {
                        eprintln!("Failed to send packet: {}", e);
                        break;
                    }
                }
                _ = &mut rx => {
                    println!("Received stop signal");
                    break;
                }
                else => break,
            }
        }
        open_channels.lock().unwrap().remove(&on_event.id());
    });

    Ok(())
}

#[tauri::command]
fn stream_items_stop(state: tauri::State<'_, SharedState>, id: u32) -> Result<(), String> {
    let mut map = state.open_channels.lock().unwrap();
    if let Some(tx) = map.remove(&id) {
        tx.send(())
            .map_err(|_| "Failed to send stop signal".to_string())?;
        Ok(())
    } else {
        Err(format!("No stream found with id: {}", id))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![stream_items, stream_items_stop])
        .setup(move |app| {
            {
                let app = app.handle().clone();
                async_runtime::spawn(async move {
                    let store = Store::spawn("./store".into()).await;
                    let pool = xs::thread_pool::ThreadPool::new(10);
                    let engine = xs::nu::Engine::new(store.clone()).unwrap();

                    {
                        let store = store.clone();
                        app.manage(SharedState {
                            open_channels: Arc::new(Mutex::new(HashMap::new())),
                            store,
                        });
                    }

                    async_runtime::spawn(async move {
                        xs::api::serve(store, engine.clone(), pool.clone(), "./store/sock")
                            .await
                            .unwrap();
                    });
                });
            }

            std::thread::sleep(std::time::Duration::from_secs(2));

            let window = app.get_webview_window("main").unwrap();
            #[cfg(debug_assertions)]
            if std::env::var("STACK_DEVTOOLS").is_ok() {
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
