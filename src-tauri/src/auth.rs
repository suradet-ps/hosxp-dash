//! HOSxP officer authentication and the in-memory login session.
//!
//! The app verifies operators against HOSxP `officer` rows only. The stored
//! MD5 digest never crosses the IPC boundary, the plaintext password is
//! moved into a [`SecretString`] for the verification window, and the session
//! lives in this process until logout, app restart, or a connection change
//! (`docs/auth.md`).

use crate::db::{is_connected, with_pool};
use md5::{Digest, Md5};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fmt;
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// Reads one local HOSxP officer for password verification.
///
/// Only `officer` is used: `opduser` is blacklisted by the HOSxP gateway and
/// the plaintext `officer_login_password` column is never selected. The
/// display name comes from the officer name fields, falling back to
/// `doctor.name` through `officer_doctor_code` when they are blank.
const FETCH_OFFICER_SQL: &str = "\
SELECT CAST(o.officer_id AS SIGNED) AS officer_id,
       o.officer_login_name AS login_name,
       COALESCE(o.officer_login_password_md5, '') AS password_md5,
       COALESCE(o.officer_active, 'N') AS active,
       CAST(COALESCE(o.officer_authentication_type_id, 1) AS SIGNED) AS auth_type_id,
       NULLIF(o.officer_doctor_code, '') AS doctor_code,
       COALESCE(
           NULLIF(TRIM(CONCAT(COALESCE(o.officer_pname, ''), COALESCE(o.officer_fname, ''), ' ', COALESCE(o.officer_lname, ''))), ''),
           NULLIF(TRIM(d.name), ''),
           ''
       ) AS display_name
FROM officer o
LEFT JOIN doctor d ON d.code = NULLIF(o.officer_doctor_code, '')
WHERE o.officer_login_name = ?
LIMIT 1";

/// Maximum accepted characters for the login name.
const USERNAME_MAX_CHARS: usize = 100;

/// Maximum accepted characters for the password.
const PASSWORD_MAX_CHARS: usize = 200;

/// One `officer` row as read from HOSxP. Internal to the backend; the hash
/// never crosses the IPC boundary.
#[derive(Clone, PartialEq, Eq)]
pub struct OfficerAccount {
  /// `officer.officer_id`.
  pub officer_id: i64,
  /// `officer.officer_login_name` - the username.
  pub login_name: String,
  /// `officer.officer_login_password_md5` - 32 hex characters, lower/upper.
  pub password_md5: String,
  /// `officer.officer_active`; only `"Y"` is accepted.
  pub active: String,
  /// `officer.officer_authentication_type_id`; `NULL` is treated as 1
  /// (local database) by the query.
  pub auth_type_id: i64,
  /// `officer.officer_doctor_code`, when present.
  pub doctor_code: Option<String>,
  /// Display name assembled from the officer name fields.
  pub display_name: String,
}

impl fmt::Debug for OfficerAccount {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("OfficerAccount")
      .field("officer_id", &self.officer_id)
      .field("login_name", &self.login_name)
      .field("password_md5", &"<redacted>")
      .field("active", &self.active)
      .field("auth_type_id", &self.auth_type_id)
      .field("doctor_code", &self.doctor_code)
      .field("display_name", &self.display_name)
      .finish()
  }
}

/// The sanitized user returned to the frontend after a successful login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
  /// `officer.officer_id`.
  pub officer_id: i64,
  /// HOSxP login name.
  pub login_name: String,
  /// Display name for the header; falls back to the login name.
  pub display_name: String,
  /// `officer.officer_doctor_code`, when present.
  pub doctor_code: Option<String>,
  /// App role. HOSxP group permissions are empty, so the first version
  /// assigns every signed-in officer the same role.
  pub role: String,
}

/// Username/password submitted from the login gate. Deliberately does not
/// derive `Debug` - the password must never be formatted into a log line.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
  /// HOSxP `officer.officer_login_name`.
  pub username: String,
  /// Plaintext password, verified in memory against the stored MD5.
  pub password: String,
}

static SESSION: OnceLock<RwLock<Option<SessionUser>>> = OnceLock::new();

fn session_lock() -> &'static RwLock<Option<SessionUser>> {
  SESSION.get_or_init(|| RwLock::new(None))
}

/// Replaces the signed-in operator.
pub async fn set_session(user: SessionUser) {
  *session_lock().write().await = Some(user);
}

/// Clears the signed-in operator.
pub async fn clear_session() {
  *session_lock().write().await = None;
}

/// Returns the signed-in operator, if any (memory only).
pub async fn session() -> Option<SessionUser> {
  session_lock().read().await.clone()
}

/// Rejects a command that requires a signed-in operator.
pub async fn require_session() -> Result<SessionUser, String> {
  session()
    .await
    .ok_or_else(|| "กรุณาเข้าสู่ระบบก่อนใช้งาน".to_string())
}

/// Returns the lowercase 32-character MD5 hex digest of `password`.
pub fn md5_hex(password: &str) -> String {
  format!("{:x}", Md5::digest(password.as_bytes()))
}

/// True when `password` hashes to the stored 32-hex MD5 digest.
///
/// The stored value is validated for shape and compared case-insensitively.
/// A missing, short, or non-hex digest never matches.
pub fn verify_password(password: &str, stored_md5: &str) -> bool {
  let stored = stored_md5.trim();
  if stored.len() != 32 || !stored.chars().all(|c| c.is_ascii_hexdigit()) {
    return false;
  }
  md5_hex(password).eq_ignore_ascii_case(stored)
}

/// Fetches one `officer` row by login name.
async fn fetch_officer(login_name: &str) -> Result<Option<OfficerAccount>, String> {
  let login_name = login_name.to_string();
  with_pool(move |pool| {
    Box::pin(async move {
      let row = sqlx::query(FETCH_OFFICER_SQL)
        .bind(&login_name)
        .fetch_optional(pool)
        .await?;

      let Some(row) = row else {
        return Ok::<_, sqlx::Error>(None);
      };

      Ok::<_, sqlx::Error>(Some(OfficerAccount {
        officer_id: row.try_get("officer_id")?,
        login_name: row.try_get("login_name")?,
        password_md5: row.try_get("password_md5")?,
        active: row.try_get("active")?,
        auth_type_id: row.try_get("auth_type_id")?,
        doctor_code: row.try_get("doctor_code")?,
        display_name: row.try_get("display_name")?,
      }))
    })
  })
  .await
}

/// Verifies the credentials and starts an in-memory session.
///
/// The plaintext password is moved into a [`SecretString`] for the
/// verification window; the stored hash never leaves this process.
pub async fn authenticate(input: LoginInput) -> Result<SessionUser, String> {
  let LoginInput { username, password } = input;
  let username = username.trim().to_string();
  if username.is_empty() {
    return Err("กรุณากรอกชื่อผู้ใช้".to_string());
  }
  if username.chars().count() > USERNAME_MAX_CHARS {
    return Err("ชื่อผู้ใช้ยาวเกินไป".to_string());
  }

  let password = SecretString::from(password);
  if password.expose_secret().is_empty() {
    return Err("กรุณากรอกรหัสผ่าน".to_string());
  }
  if password.expose_secret().chars().count() > PASSWORD_MAX_CHARS {
    return Err("รหัสผ่านยาวเกินไป".to_string());
  }

  if !is_connected().await {
    return Err("ยังไม่ได้เชื่อมต่อฐานข้อมูล กรุณาตั้งค่าการเชื่อมต่อก่อนเข้าสู่ระบบ".to_string());
  }

  clear_session().await;
  let account = fetch_officer(&username)
    .await
    .map_err(|err| format!("เชื่อมต่อฐานข้อมูลไม่สำเร็จ: {err}"))?;
  let Some(account) = account else {
    return Err("ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง".to_string());
  };
  if account.auth_type_id != 1 {
    return Err("บัญชีนี้ใช้การยืนยันตัวตนแบบ LDAP ซึ่งยังไม่เปิดใช้งาน".to_string());
  }
  if !account.active.eq_ignore_ascii_case("Y") {
    return Err("บัญชีนี้ถูกปิดใช้งาน กรุณาติดต่อผู้ดูแลระบบ".to_string());
  }
  if !verify_password(password.expose_secret(), &account.password_md5) {
    return Err("ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง".to_string());
  }

  let display_name = if account.display_name.trim().is_empty() {
    account.login_name.clone()
  } else {
    account.display_name
  };
  let user = SessionUser {
    officer_id: account.officer_id,
    login_name: account.login_name,
    display_name,
    doctor_code: account.doctor_code,
    role: "STAFF".to_string(),
  };
  set_session(user.clone()).await;
  Ok(user)
}

/// Verifies a HOSxP officer password and starts an in-memory session.
#[tauri::command]
pub async fn login(input: LoginInput) -> Result<SessionUser, String> {
  authenticate(input).await
}

/// Clears the in-memory session.
#[tauri::command]
pub async fn logout() -> Result<(), String> {
  clear_session().await;
  Ok(())
}

/// Returns the current session, if any (memory only).
#[tauri::command]
pub async fn current_session() -> Result<Option<SessionUser>, String> {
  Ok(session().await)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn md5_hex_matches_the_rfc_1321_vector() {
    assert_eq!(md5_hex("abc"), "900150983cd24fb0d6963f7d28e17f72");
  }

  #[test]
  fn verify_accepts_lower_and_upper_hex() {
    assert!(verify_password("abc", "900150983cd24fb0d6963f7d28e17f72"));
    assert!(verify_password("abc", "900150983CD24FB0D6963F7D28E17F72"));
  }

  #[test]
  fn verify_rejects_wrong_or_malformed_hashes() {
    assert!(!verify_password("abcd", "900150983cd24fb0d6963f7d28e17f72"));
    assert!(!verify_password("abc", ""));
    assert!(!verify_password("abc", "not-a-md5"));
    assert!(!verify_password("abc", "900150983cd24fb0d6963f7d28e17f7z"));
  }

  #[test]
  fn officer_query_uses_only_the_safe_columns() {
    assert!(FETCH_OFFICER_SQL.contains("FROM officer o"));
    assert!(FETCH_OFFICER_SQL.contains("officer_login_password_md5"));
    assert!(!FETCH_OFFICER_SQL.contains("officer_login_password "));
    assert!(!FETCH_OFFICER_SQL.contains("opduser"));
    assert!(FETCH_OFFICER_SQL.contains("LEFT JOIN doctor d"));
    assert!(FETCH_OFFICER_SQL.contains("LIMIT 1"));
    assert!(!FETCH_OFFICER_SQL.contains(';'));
  }

  #[test]
  fn officer_debug_redacts_the_password_hash() {
    let account = OfficerAccount {
      officer_id: 1,
      login_name: "pharm".into(),
      password_md5: "test-digest".into(),
      active: "Y".into(),
      auth_type_id: 1,
      doctor_code: None,
      display_name: "Pharmacist".into(),
    };
    let debug = format!("{account:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("test-digest"));
  }
}
