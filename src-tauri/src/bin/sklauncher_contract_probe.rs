fn main() {
    if let Err(error) =
        minecraft_setup_manager_lib::validation_tools::run_sklauncher_contract_probe()
    {
        eprintln!("SKlauncher contract probe failed: {error}");
        std::process::exit(1);
    }
}
