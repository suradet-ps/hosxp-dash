//! Tauri application entry-point.
//!
//! Wires up the plugin stack, registers IPC command handlers, and delegates
//! all business logic to the `commands` and `db` modules.

mod commands;
mod config;
mod db;

use commands::{
  connect_db, get_available_years, get_dashboard_data, get_drug_list, get_drug_monthly_qty,
  get_top_drugs, load_config, save_db_config,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
      connect_db,
      get_dashboard_data,
      get_drug_monthly_qty,
      get_top_drugs,
      get_available_years,
      get_drug_list,
      load_config,
      save_db_config,
    ])
    .run(tauri::generate_context!())
    .expect("invariant: tauri context is generated at compile time and is always valid");
}
