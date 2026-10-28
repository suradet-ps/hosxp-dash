//! Tauri application entry-point.
//!
//! Wires up the plugin stack, registers IPC command handlers, and delegates
//! all business logic to the `auth`, `commands`, and `db` modules.

mod auth;
mod commands;
mod config;
mod db;

use auth::{current_session, login, logout};
use commands::{
  connect_db, get_antibiotic_usage, get_available_years, get_dashboard_data, get_drug_list,
  get_drug_monthly_qty, get_top_drugs, load_config, save_db_config, save_pdf_file,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Migrate legacy `.master_key` file to OS keychain (if exists).
  if let Err(e) = config::migrate_legacy_key() {
    eprintln!("Warning: failed to migrate legacy master key: {e}");
  }

  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      // Open sized to the primary monitor — the 1440×900 config default
      // overflows smaller laptop screens — and centered before showing, so
      // the window never flashes at the old size or an off-center position.
      if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.primary_monitor() {
          let scale = monitor.scale_factor();
          let area = monitor.size().to_logical::<f64>(scale);
          let width = (area.width - 96.0).clamp(1024.0, 1440.0);
          let height = (area.height - 96.0).clamp(660.0, 900.0);
          let _ = window.set_size(tauri::LogicalSize::new(width, height));
        }
        let _ = window.center();
        let _ = window.show();
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      connect_db,
      current_session,
      get_antibiotic_usage,
      get_dashboard_data,
      get_drug_monthly_qty,
      get_top_drugs,
      get_available_years,
      get_drug_list,
      load_config,
      login,
      logout,
      save_db_config,
      save_pdf_file,
    ])
    .run(tauri::generate_context!())
    .expect("invariant: tauri context is generated at compile time and is always valid");
}
