//! Encrypted configuration management.
//!
//! Stores database connection settings in an encrypted `config.json` file
//! inside the platform-specific app data directory.  The master key is
//! persisted separately in `.master_key` and loaded on startup.

use encryptman::{decrypt, encrypt, generate_master_key, MasterKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::db::DbConfig;

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Application name used for the app-data directory.
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

/// Returns the path to `.master_key`.
fn key_path() -> Result<PathBuf, String> {
  Ok(app_dir()?.join(".master_key"))
}

// ── Master key persistence ─────────────────────────────────────────────────────

/// Load the master key from disk, or generate and persist a new one.
///
/// The key file is a raw 32-byte binary file.
fn load_or_create_master_key() -> Result<MasterKey, String> {
  let path = key_path()?;

  if path.exists() {
    let bytes = fs::read(&path).map_err(|e| format!("failed to read master key: {e}"))?;
    return MasterKey::try_from(bytes.as_slice()).map_err(|e| format!("invalid master key: {e}"));
  }

  let key = generate_master_key();
  fs::write(&path, key.as_bytes()).map_err(|e| format!("failed to write master key: {e}"))?;
  Ok(key)
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

/// Encrypt a single field value.
fn enc(key: &MasterKey, value: &str) -> Result<String, String> {
  encrypt(key, value).map_err(|e| format!("encryption failed: {e}"))
}

/// Decrypt a single field value.
fn dec(key: &MasterKey, value: &str) -> Result<String, String> {
  // encryptman::encrypt uses the default context, which matches here
  decrypt(key, value).map_err(|e| format!("decryption failed: {e}"))
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

  let master_key = load_or_create_master_key()?;

  Ok(Some(DbConfig {
    host: dec(&master_key, &encrypted.host)?,
    port: dec(&master_key, &encrypted.port)?
      .parse::<u16>()
      .map_err(|e| format!("invalid port: {e}"))?,
    user: dec(&master_key, &encrypted.user)?,
    password: dec(&master_key, &encrypted.password)?,
    database: dec(&master_key, &encrypted.database)?,
  }))
}

/// Encrypt and save the config to disk.
///
/// Creates the app data directory and master key if they don't exist yet.
pub fn save_config(config: &DbConfig) -> Result<(), String> {
  let master_key = load_or_create_master_key()?;

  let encrypted = EncryptedConfigFile {
    host: enc(&master_key, &config.host)?,
    port: enc(&master_key, &config.port.to_string())?,
    user: enc(&master_key, &config.user)?,
    password: enc(&master_key, &config.password)?,
    database: enc(&master_key, &config.database)?,
  };

  let json =
    serde_json::to_string_pretty(&encrypted).map_err(|e| format!("failed to serialize: {e}"))?;

  fs::write(config_path()?, json).map_err(|e| format!("failed to write config: {e}"))?;
  Ok(())
}
