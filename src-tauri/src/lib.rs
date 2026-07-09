mod app_state;
mod commands;
mod http_client;
mod launcher;
mod manifest;
mod minecraft;
mod performance_profiles;
mod planner;
mod server;
mod setup;
mod system;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_launchers,
            commands::export_diagnostics,
            commands::get_install_plan,
            commands::list_saved_servers,
            commands::resolve_server_manifest,
            commands::start_install,
            commands::validate_installation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Minecraft Setup Manager");
}
