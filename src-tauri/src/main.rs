// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// extern crate r2d2;
// extern crate r2d2_sqlite;
// extern crate rusqlite;

mod blocks;
mod configuration;
mod storage;

fn main() {
    let configuration = configuration::Configuration::init().unwrap();
    log::info!("Starting My Blocks!");

    let db_pool = storage::setup_database(&configuration).expect("Could not set up database.");

    let connection = db_pool.get().expect("Could not get db connection");
    let workspace = blocks::Block::find_or_create_workspace_block(configuration.workspace_id, &connection); 

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(db_pool)
        .manage(configuration)
        .manage(workspace)
        .invoke_handler(tauri::generate_handler![
            configuration::load_configuration_command,
            blocks::create_block_command,
            blocks::get_block_command,
            blocks::load_blocks_for_page_command,
            blocks::change_block_order_command,
            blocks::load_home_page_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
