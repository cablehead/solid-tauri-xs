use tauri::Manager;

use tokio::time::{sleep, Duration};

#[derive(serde::Serialize, Clone)]
pub struct Packet {
    message: String,
}

#[tauri::command]
async fn stream_items(on_event: tauri::ipc::Channel<Packet>) {
    let mut count = 0u32;

    loop {
        count += 1;
        on_event
            .send(Packet {
                message: format!("Hello from Rust! {}", count),
            })
            .unwrap();
        sleep(Duration::from_secs(1)).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![stream_items])
        .setup(move |app| {
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
