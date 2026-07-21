//! Tauri IPC command handlers and in-process query-result caching.
//!
//! Each public `async fn` decorated with `#[tauri::command]` maps to a
//! frontend `invoke()` call.  Heavy query results are cached in process-wide
//! `LazyLock<RwLock<HashMap>>` statics with a 5-minute TTL.

use crate::config;
use crate::db::{DbConfig, init_pool, with_pool};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ── Cache infrastructure ───────────────────────────────────────────────────────
//
// Each query family gets its own typed cache keyed by a string that encodes
// the query parameters (year, limit, icode, …).  Caches live as process-wide
// statics so they survive across multiple Tauri command invocations.
//
// TTL = 5 minutes.  Data is re-fetched transparently when an entry expires.
// All caches are wiped on every successful DB reconnect.

const CACHE_TTL: Duration = Duration::from_secs(300);

/// `Arc<RwLock<…>>` so the static can be shared + mutated across async tasks.
type Cache<T> = Arc<RwLock<HashMap<String, (Instant, T)>>>;

static YEARS_CACHE: LazyLock<Cache<Vec<i32>>> =
  LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

static TOP_DRUGS_CACHE: LazyLock<Cache<Vec<DrugSummary>>> =
  LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

static MONTHLY_CACHE: LazyLock<Cache<Vec<DrugMonthlyData>>> =
  LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Try to read a fresh value from `cache` for `key`.
/// Returns `None` if the key is absent or its TTL has expired.
async fn cache_get<T: Clone>(cache: &Cache<T>, key: &str) -> Option<T> {
  let guard = cache.read().await;
  if let Some((ts, val)) = guard.get(key)
    && ts.elapsed() < CACHE_TTL
  {
    return Some(val.clone());
  }
  None
}

/// Insert (or overwrite) a value in `cache` under `key`, stamped with now.
async fn cache_set<T: Clone>(cache: &Cache<T>, key: String, val: T) {
  cache.write().await.insert(key, (Instant::now(), val));
}

/// Wipe every cache.  Called by `connect_db` after a successful reconnect so
/// that stale data from the previous connection is never served.
pub(crate) async fn invalidate_all_caches() {
  YEARS_CACHE.write().await.clear();
  TOP_DRUGS_CACHE.write().await.clear();
  MONTHLY_CACHE.write().await.clear();
}

// ── Data structs ───────────────────────────────────────────────────────────────

/// Monthly dispensing data for a single drug across 12 months.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrugMonthlyData {
  pub icode: String,
  pub drug_name: String,
  /// 12-element vec; index 0 = January, index 11 = December.
  pub monthly_qty: Vec<f64>,
  pub total_qty: f64,
}

/// Aggregate summary used in the top-drugs panel.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrugSummary {
  pub icode: String,
  pub drug_name: String,
  pub total_qty: f64,
  /// 1-based month number (1 = January … 12 = December).
  pub peak_month: u32,
}

/// Lightweight item for autocomplete search results.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrugItem {
  pub icode: String,
  pub name: String,
}

/// Combined payload returned by `get_dashboard_data`.
/// Fetching years + top-drugs in one IPC round-trip saves an extra async hop.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardBundle {
  pub years: Vec<i32>,
  pub top_drugs: Vec<DrugSummary>,
}

// ── Row-extraction helpers ─────────────────────────────────────────────────────
//
// MySQL function results (MONTH, YEAR, CAST … AS DOUBLE) can come back with
// different column types depending on the server version and sqlx decoder.
// These helpers try the most-likely types and fall back gracefully so that
// commands never panic on a type-mismatch at runtime.

fn col_string(row: &sqlx::mysql::MySqlRow, col: &str) -> String {
  if let Ok(v) = row.try_get::<String, _>(col) {
    return v;
  }
  if let Ok(Some(v)) = row.try_get::<Option<String>, _>(col) {
    return v;
  }
  String::new()
}

fn col_u32(row: &sqlx::mysql::MySqlRow, col: &str) -> u32 {
  if let Ok(v) = row.try_get::<u32, _>(col) {
    return v;
  }
  if let Ok(v) = row.try_get::<i64, _>(col) {
    return v.clamp(0, u32::MAX as i64) as u32;
  }
  if let Ok(Some(v)) = row.try_get::<Option<u32>, _>(col) {
    return v;
  }
  if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(col) {
    return v.clamp(0, u32::MAX as i64) as u32;
  }
  0
}

fn col_f64(row: &sqlx::mysql::MySqlRow, col: &str) -> f64 {
  if let Ok(v) = row.try_get::<f64, _>(col) {
    return v;
  }
  if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(col) {
    return v;
  }
  if let Ok(v) = row.try_get::<i64, _>(col) {
    return v as f64;
  }
  if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(col) {
    return v as f64;
  }
  if let Ok(v) = row.try_get::<u64, _>(col) {
    return v as f64;
  }
  0.0
}

// ── Internal query helpers ─────────────────────────────────────────────────────
//
// These functions take a bare `&MySqlPool` so they can be called from both
// standalone Tauri commands AND from `tokio::join!` blocks (where two futures
// share the same pool reference concurrently).

/// Fetch distinct years present in `opitemrece.vstdate`, newest first.
/// Result is cached under the fixed key `"years"`.
async fn fetch_years_internal(pool: &sqlx::MySqlPool) -> Result<Vec<i32>, sqlx::Error> {
  if let Some(cached) = cache_get(&*YEARS_CACHE, "years").await {
    return Ok(cached);
  }

  let rows = sqlx::query("SELECT DISTINCT YEAR(vstdate) AS yr FROM opitemrece ORDER BY yr DESC")
    .fetch_all(pool)
    .await?;

  let years: Vec<i32> = rows
    .iter()
    .map(|r| col_u32(r, "yr") as i32)
    .filter(|&y| y > 0)
    .collect();

  cache_set(&*YEARS_CACHE, "years".into(), years.clone()).await;
  Ok(years)
}

/// Fetch the top-`limit` drugs by total dispensed quantity in `year`.
///
/// ### Why two steps instead of a correlated subquery?
///
/// The original SQL had a correlated subquery for `peak_month`:
///
/// ```sql
/// (SELECT MONTH(…) FROM opitemrece WHERE icode = o.icode … LIMIT 1) AS peak_month
/// ```
///
/// MySQL executes that subquery **once per result row** — an N+1 problem that
/// becomes O(drugs²) on large tables.
///
/// The replacement strategy:
/// 1. One fast `GROUP BY` scan → totals only, no subquery.
/// 2. One `IN(…)` scan for just the top-N icodes → monthly breakdown.
/// 3. Peak month computed in Rust → O(N × 12), zero extra SQL round-trips.
async fn fetch_top_drugs_internal(
  pool: &sqlx::MySqlPool,
  year: i32,
  limit: u8,
) -> Result<Vec<DrugSummary>, sqlx::Error> {
  let cache_key = format!("top_{}_{}", year, limit);
  if let Some(cached) = cache_get(&*TOP_DRUGS_CACHE, &cache_key).await {
    return Ok(cached);
  }

  // ── Step 1: totals only (single table scan, no correlated subquery) ──────
  let total_rows = sqlx::query(
    r#"
        SELECT STRAIGHT_JOIN
            o.icode                           AS icode,
            COALESCE(d.name, o.icode)         AS drug_name,
            CAST(SUM(o.qty) AS DOUBLE)        AS total_qty
        FROM opitemrece o
        LEFT JOIN drugitems d ON d.icode = o.icode
        WHERE YEAR(o.vstdate) = ?
        GROUP BY o.icode, d.name
        ORDER BY total_qty DESC
        LIMIT ?
        "#,
  )
  .bind(year)
  .bind(limit as i64)
  .fetch_all(pool)
  .await?;

  if total_rows.is_empty() {
    cache_set(&*TOP_DRUGS_CACHE, cache_key, vec![]).await;
    return Ok(vec![]);
  }

  let icodes: Vec<String> = total_rows.iter().map(|r| col_string(r, "icode")).collect();

  // ── Step 2: monthly breakdown for just those N icodes ────────────────────
  // Dynamic IN clause — sqlx doesn't support array binding, so we build the
  // placeholder string manually and bind each icode individually.
  let placeholders = icodes.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
  let monthly_sql = format!(
    r#"
        SELECT
            o.icode                      AS icode,
            MONTH(o.vstdate)             AS month,
            CAST(SUM(o.qty) AS DOUBLE)   AS qty
        FROM opitemrece o
        WHERE YEAR(o.vstdate) = ?
          AND o.icode IN ({})
        GROUP BY o.icode, MONTH(o.vstdate)
        "#,
    placeholders
  );

  let mut q = sqlx::query(&monthly_sql).bind(year);
  for ic in &icodes {
    q = q.bind(ic.as_str());
  }
  let monthly_rows = q.fetch_all(pool).await?;

  // ── Step 3: pivot monthly rows into [f64; 12] maps in Rust ───────────────
  let mut monthly_map: HashMap<String, [f64; 12]> = HashMap::with_capacity(icodes.len());
  for row in &monthly_rows {
    let ic = col_string(row, "icode");
    let mo = col_u32(row, "month");
    let qty = col_f64(row, "qty");
    if (1..=12).contains(&mo) {
      monthly_map.entry(ic).or_insert([0.0; 12])[(mo - 1) as usize] += qty;
    }
  }

  // ── Step 4: assemble DrugSummary, computing peak_month in O(N*12) ────────
  let result: Vec<DrugSummary> = total_rows
    .iter()
    .map(|row| {
      let ic = col_string(row, "icode");
      let dn = {
        let v = col_string(row, "drug_name");
        if v.is_empty() { ic.clone() } else { v }
      };
      let total = col_f64(row, "total_qty");
      let peak = if let Some(months) = monthly_map.get(&ic) {
        months
          .iter()
          .enumerate()
          .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
          .map(|(i, _)| (i + 1) as u32)
          .unwrap_or(1)
      } else {
        1
      };
      DrugSummary {
        icode: ic,
        drug_name: dn,
        total_qty: total,
        peak_month: peak,
      }
    })
    .collect();

  cache_set(&*TOP_DRUGS_CACHE, cache_key, result.clone()).await;
  Ok(result)
}

// ── Tauri commands ─────────────────────────────────────────────────────────────

/// Initialise (or replace) the global MySQL connection pool, then wipe all
/// query caches so stale data from the previous session is never served.
#[tauri::command]
pub async fn connect_db(config: DbConfig) -> Result<(), String> {
  init_pool(config).await?;
  invalidate_all_caches().await;
  Ok(())
}

/// ★ Combined bootstrap command.
///
/// Fetches available years AND top-N drugs **in parallel** using
/// `tokio::join!`, which drives both futures on the same Tokio task but
/// allows each to make progress while the other waits for the DB.
///
/// The frontend calls this single IPC command on startup instead of issuing
/// two separate `invoke()` calls, halving the IPC round-trip overhead.
#[tauri::command]
pub async fn get_dashboard_data(year: i32, top_limit: u8) -> Result<DashboardBundle, String> {
  with_pool(move |pool| {
    Box::pin(async move {
      // Both queries share the same pool reference concurrently.
      // `&MySqlPool` is Copy, so this is safe — the pool itself is
      // Arc-backed and handles concurrent borrows internally.
      let (years_result, top_result) = tokio::join!(
        fetch_years_internal(pool),
        fetch_top_drugs_internal(pool, year, top_limit),
      );
      Ok::<DashboardBundle, sqlx::Error>(DashboardBundle {
        years: years_result?,
        top_drugs: top_result?,
      })
    })
  })
  .await
}

/// Return monthly dispensing quantities for every drug active in `year`.
/// When `icode` is `Some(…)`, only that drug is returned (used for the
/// detail chart view).  Results are cached per `(year, icode)` key.
#[tauri::command]
pub async fn get_drug_monthly_qty(
  year: i32,
  icode: Option<String>,
) -> Result<Vec<DrugMonthlyData>, String> {
  let cache_key = match &icode {
    Some(ic) => format!("monthly_{}_drug_{}", year, ic),
    None => format!("monthly_{}_<all>", year),
  };

  // Check cache before acquiring a pool connection.
  if let Some(cached) = cache_get(&*MONTHLY_CACHE, &cache_key).await {
    return Ok(cached);
  }

  with_pool(move |pool| {
    Box::pin(async move {
      let rows = match &icode {
        Some(ic) => {
          sqlx::query(
            r#"
                        SELECT STRAIGHT_JOIN
                            o.icode                      AS icode,
                            COALESCE(d.name, o.icode)    AS drug_name,
                            MONTH(o.vstdate)             AS month,
                            CAST(SUM(o.qty) AS DOUBLE)   AS total_qty
                        FROM opitemrece o
                        LEFT JOIN drugitems d ON d.icode = o.icode
                        WHERE YEAR(o.vstdate) = ?
                          AND o.icode = ?
                        GROUP BY o.icode, d.name, MONTH(o.vstdate)
                        ORDER BY month
                        "#,
          )
          .bind(year)
          .bind(ic.as_str())
          .fetch_all(pool)
          .await?
        }
        None => {
          sqlx::query(
            r#"
                        SELECT STRAIGHT_JOIN
                            o.icode                      AS icode,
                            COALESCE(d.name, o.icode)    AS drug_name,
                            MONTH(o.vstdate)             AS month,
                            CAST(SUM(o.qty) AS DOUBLE)   AS total_qty
                        FROM opitemrece o
                        LEFT JOIN drugitems d ON d.icode = o.icode
                        WHERE YEAR(o.vstdate) = ?
                        GROUP BY o.icode, d.name, MONTH(o.vstdate)
                        ORDER BY o.icode, month
                        "#,
          )
          .bind(year)
          .fetch_all(pool)
          .await?
        }
      };

      // Pivot flat (icode, month, qty) rows into per-drug 12-element vecs.
      let mut map: HashMap<String, DrugMonthlyData> = HashMap::with_capacity(64);
      for row in &rows {
        let ic = col_string(row, "icode");
        let dn = {
          let v = col_string(row, "drug_name");
          if v.is_empty() { ic.clone() } else { v }
        };
        let mo = col_u32(row, "month");
        let qty = col_f64(row, "total_qty");

        let entry = map.entry(ic.clone()).or_insert_with(|| DrugMonthlyData {
          icode: ic.clone(),
          drug_name: dn,
          monthly_qty: vec![0.0; 12],
          total_qty: 0.0,
        });
        if (1..=12).contains(&mo) {
          entry.monthly_qty[(mo - 1) as usize] = qty;
          entry.total_qty += qty;
        }
      }

      let mut result: Vec<DrugMonthlyData> = map.into_values().collect();
      result.sort_by(|a, b| {
        b.total_qty
          .partial_cmp(&a.total_qty)
          .unwrap_or(std::cmp::Ordering::Equal)
      });

      cache_set(&*MONTHLY_CACHE, cache_key, result.clone()).await;
      Ok::<Vec<DrugMonthlyData>, sqlx::Error>(result)
    })
  })
  .await
}

/// Convenience wrapper around `fetch_top_drugs_internal` for callers that
/// want top drugs without the full `DashboardBundle`.
#[tauri::command]
pub async fn get_top_drugs(year: i32, limit: u8) -> Result<Vec<DrugSummary>, String> {
  with_pool(move |pool| Box::pin(async move { fetch_top_drugs_internal(pool, year, limit).await }))
    .await
}

/// Return all distinct years present in `opitemrece.vstdate`, newest first.
#[tauri::command]
pub async fn get_available_years() -> Result<Vec<i32>, String> {
  with_pool(|pool| Box::pin(async move { fetch_years_internal(pool).await })).await
}

/// Search `drugitems` by icode prefix or name substring.
/// Returns at most 50 results ordered by name, for use in autocomplete.
/// Results are NOT cached because the dataset is small and the query is fast.
#[tauri::command]
pub async fn get_drug_list(search: String) -> Result<Vec<DrugItem>, String> {
  let escaped = search
    .replace('\\', "\\\\")
    .replace('%', "\\%")
    .replace('_', "\\_");
  let pattern = format!("%{}%", escaped);
  with_pool(move |pool| {
    Box::pin(async move {
      let rows = sqlx::query(
        r#"
                SELECT
                    icode,
                    COALESCE(name, icode) AS drug_name
                FROM drugitems
                WHERE icode LIKE ?
                   OR name  LIKE ?
                ORDER BY name
                LIMIT 50
                "#,
      )
      .bind(pattern.as_str())
      .bind(pattern.as_str())
      .fetch_all(pool)
      .await?;

      let result: Vec<DrugItem> = rows
        .iter()
        .map(|r| {
          let icode = col_string(r, "icode");
          let name = {
            let v = col_string(r, "drug_name");
            if v.is_empty() { icode.clone() } else { v }
          };
          DrugItem { icode, name }
        })
        .collect();

      Ok::<Vec<DrugItem>, sqlx::Error>(result)
    })
  })
  .await
}

// ── Config commands ────────────────────────────────────────────────────────────

/// Load the encrypted database config from disk.
/// Returns `Ok(None)` on first launch (no config file yet).
#[tauri::command]
pub async fn load_config() -> Result<Option<DbConfig>, String> {
  config::load_config()
}

/// Encrypt and persist the database config to disk.
#[tauri::command]
pub async fn save_db_config(config: DbConfig) -> Result<(), String> {
  config::save_config(&config)
}
