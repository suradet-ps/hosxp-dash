//! Global MySQL connection-pool management.
//!
//! Exposes [`init_pool`] to (re-)initialise the pool from a [`DbConfig`] and
//! [`with_pool`] so command handlers can borrow the pool without holding a
//! long-lived lock.

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::RwLock;

// ── Connection pool ────────────────────────────────────────────────────────────

static POOL: OnceLock<RwLock<Option<MySqlPool>>> = OnceLock::new();

fn get_pool_lock() -> &'static RwLock<Option<MySqlPool>> {
  POOL.get_or_init(|| RwLock::new(None))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
/// MySQL connection parameters supplied by the frontend on first connect.
pub struct DbConfig {
  pub host: String,
  pub port: u16,
  pub user: String,
  pub password: String,
  pub database: String,
}

/// Initialise (or replace) the global MySQL connection pool.
///
/// Pool is tuned for a dashboard workload:
///  - 12 max connections → enough for parallel queries during a full page load
///  - 3 min connections kept warm → instant acquire on UI interaction
///  - 20 s acquire timeout → UI shows an error rather than hanging forever
///  - test_before_acquire = false → skip the PING round-trip on every acquire
pub async fn init_pool(config: DbConfig) -> Result<(), String> {
  let url = format!(
    "mysql://{}:{}@{}:{}/{}",
    urlencoding_simple(&config.user),
    urlencoding_simple(&config.password),
    config.host,
    config.port,
    config.database,
  );

  let pool = MySqlPoolOptions::new()
    .max_connections(12)
    .min_connections(3)
    .acquire_timeout(Duration::from_secs(20))
    .idle_timeout(Duration::from_secs(600)) // 10 min → close idle connections
    .max_lifetime(Duration::from_secs(3600)) // 1 hr  → recycle stale connections
    .test_before_acquire(false) // skip PING; saves ~1 ms per acquire
    .connect(&url)
    .await
    .map_err(|e| format!("DB connection failed: {}", e))?;

  let mut guard = get_pool_lock().write().await;
  *guard = Some(pool);

  Ok(())
}

/// Percent-encode the characters that are meaningful in a MySQL DSN
/// (`@`, `:`, `/`, space, `#`).  All other bytes are passed through as-is.
///
/// Using a hand-rolled encoder instead of a crate keeps the dependency tree
/// small and avoids pulling in `url` or `percent-encoding` just for five chars.
fn urlencoding_simple(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 8); // small head-room for escapes
  for c in s.chars() {
    match c {
      '@' => out.push_str("%40"),
      ':' => out.push_str("%3A"),
      '/' => out.push_str("%2F"),
      ' ' => out.push_str("%20"),
      '#' => out.push_str("%23"),
      _ => out.push(c),
    }
  }
  out
}

/// Whether a connection pool has been initialised.
///
/// Used by the login command to return a Thai "configure the connection
/// first" message instead of the generic pool error.
pub async fn is_connected() -> bool {
  get_pool_lock().read().await.is_some()
}

/// Acquire a read-lock on the pool, then call `f` with a reference to it.
///
/// Returns `Err("Database not connected …")` when no pool has been
/// initialised yet, so every caller gets a human-readable message instead of
/// an unwrap panic.
pub async fn with_pool<F, T, E>(f: F) -> Result<T, String>
where
  F: for<'a> FnOnce(
    &'a MySqlPool,
  ) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'a>,
  >,
  E: std::fmt::Display,
{
  let guard = get_pool_lock().read().await;
  match guard.as_ref() {
    Some(pool) => f(pool).await.map_err(|e| e.to_string()),
    None => Err("Database not connected. Please configure connection settings.".to_string()),
  }
}
