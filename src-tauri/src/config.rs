//! Encrypted configuration management.
//!
//! Stores database connection settings in an encrypted `config.json` file
//! inside the platform-specific app data directory. The master key is
//! stored in the OS keychain via `encryptman-keyring`.

use encryptman_keyring::Vault;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::db::DbConfig;

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Application name used for the keyring service and app-data directory.
const APP_NAME: &str = "hosxp-dash";

/// Returns the platform-specific app data directory.
///
/// - macOS:   `~/Library/Application Support/hosxp-dash`
/// - Linux:   `~/.config/hosxp-dash`
/// - Windows: `%APPDATA%\hosxp-dash`
fn app_dir() -> Result<PathBuf, String> {
  let dir = dirs::data_dir()
    .ok_or("cannot determine app data directory")?
    .join(APP_NAME);
  fs::create_dir_all(&dir).map_err(|e| format!("failed to create app dir: {e}"))?;
  Ok(dir)
}

/// Returns the path to `config.json`.
fn config_path() -> Result<PathBuf, String> {
  Ok(app_dir()?.join("config.json"))
}

/// Returns the path to legacy `.master_key` file (for migration).
fn legacy_key_path() -> Result<PathBuf, String> {
  Ok(app_dir()?.join(".master_key"))
}

// ── Vault ─────────────────────────────────────────────────────────────────────

/// Create or load a vault backed by the OS keychain.
///
/// On first run, a new master key is generated and stored in the keychain.
/// On subsequent runs, the existing key is loaded automatically.
fn vault() -> Result<Vault, String> {
  Vault::new(APP_NAME).map_err(|e| format!("keychain error: {e}"))
}

// ── Migration from file-based keys ────────────────────────────────────────────

/// Migrate a legacy `.master_key` file into the OS keychain.
///
/// If a `.master_key` file exists, it is imported into the keychain and
/// deleted on success. Returns `Ok(true)` if a migration happened.
pub fn migrate_legacy_key() -> Result<bool, String> {
  let path = legacy_key_path()?;
  if !path.exists() {
    return Ok(false);
  }

  Vault::migrate_from_file(APP_NAME, &path)
    .map_err(|e| format!("failed to migrate master key: {e}"))?;

  Ok(true)
}

// ── Encrypted config ───────────────────────────────────────────────────────────

/// On-disk representation of the encrypted config.
///
/// Each field is the base64-encoded ciphertext produced by `encryptman`.
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedConfigFile {
  host: String,
  port: String,
  user: String,
  password: String,
  database: String,
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Load and decrypt the saved config from disk.
///
/// Returns `Ok(None)` when no config file exists yet (first launch).
pub fn load_config() -> Result<Option<DbConfig>, String> {
  let path = config_path();
  if !path.as_ref().map_or(false, |p| p.exists()) {
    return Ok(None);
  }

  let raw = fs::read(path?).map_err(|e| format!("failed to read config: {e}"))?;
  let encrypted: EncryptedConfigFile =
    serde_json::from_slice(&raw).map_err(|e| format!("invalid config JSON: {e}"))?;

  let vault = vault()?;

  Ok(Some(DbConfig {
    host: vault
      .decrypt(&encrypted.host)
      .map_err(|e| format!("decryption failed: {e}"))?,
    port: vault
      .decrypt(&encrypted.port)
      .map_err(|e| format!("decryption failed: {e}"))?
      .parse::<u16>()
      .map_err(|e| format!("invalid port: {e}"))?,
    user: vault
      .decrypt(&encrypted.user)
      .map_err(|e| format!("decryption failed: {e}"))?,
    password: vault
      .decrypt(&encrypted.password)
      .map_err(|e| format!("decryption failed: {e}"))?,
    database: vault
      .decrypt(&encrypted.database)
      .map_err(|e| format!("decryption failed: {e}"))?,
  }))
}

/// Encrypt and save the config to disk.
///
/// Creates the app data directory and master key if they don't exist yet.
pub fn save_config(config: &DbConfig) -> Result<(), String> {
  let vault = vault()?;

  let encrypted = EncryptedConfigFile {
    host: vault
      .encrypt(&config.host)
      .map_err(|e| format!("encryption failed: {e}"))?,
    port: vault
      .encrypt(&config.port.to_string())
      .map_err(|e| format!("encryption failed: {e}"))?,
    user: vault
      .encrypt(&config.user)
      .map_err(|e| format!("encryption failed: {e}"))?,
    password: vault
      .encrypt(&config.password)
      .map_err(|e| format!("encryption failed: {e}"))?,
    database: vault
      .encrypt(&config.database)
      .map_err(|e| format!("encryption failed: {e}"))?,
  };

  let json =
    serde_json::to_string_pretty(&encrypted).map_err(|e| format!("failed to serialize: {e}"))?;

  fs::write(config_path()?, json).map_err(|e| format!("failed to write config: {e}"))?;
  Ok(())
}
