mod commands;
mod launcher;
mod manifest;
mod minecraft;
mod performance_profiles;
mod setup;
mod system;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_launchers,
            commands::export_diagnostics,
            commands::get_install_plan,
            commands::start_install,
            commands::validate_installation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Maresme MC setup app");
}
