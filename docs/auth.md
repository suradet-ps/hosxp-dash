# Authentication - HOSxP officer login

HOSxP Dash signs operators in with their HOSxP **`officer`** account. The
HOSxP database remains read-only: the login session lives only in the Tauri
process, and the app never writes an audit row or a session token back to
HOSxP.

The design is adapted from the sibling ER-Refill project
(`er-refill/docs/auth.md`), with the same source table, verification rule,
and memory-only session.

## A. Source of truth

| Table | Observed on the Sra Bos instance | Use |
|---|---|---|
| `officer` | 104 accounts, 100% active; `officer_login_name` unique and non-empty; `officer_login_password_md5` 32 hex on all 104 | **The only login source** |
| `opduser` | 221 old (V3) accounts | **Never read** - the gateway blacklists it |
| `onlineuser` | 56 rows / 19 users | Optional online-status hint only, never identity |
| `officer_activity_log` | 131,415 rows | Existing HOSxP audit trail; HOSxP Dash does not write it |
| `user_jwt` | 1,868 rows | Never read |
| `emp` | 0 rows | Not used |

Additional verified facts:

- `officer_authentication_type_id` is `NULL` on all 104 rows; the query
  coalesces it to **1 = Local Database**. LDAP (2) is rejected for now.
- `officer_login_password` (plaintext) has only one populated row. It is
  **never selected**; only `officer_login_password_md5` is read.
- `officer_active` is `'Y'` on all current rows, but the login command still
  checks it before creating a session.
- `officer_group*`, `officer_module`, `officer_task`, and
  `officer_operation` are not a usable permission source (group tables are
  empty). The app assigns every signed-in officer the same `STAFF` role for
  now; app-specific roles are a later milestone.
- `officer_doctor_code` is present on 103/104 accounts and has one orphan
  (no matching `doctor` row). The query uses `NULLIF` and falls back to the
  login name when neither the officer name fields nor `doctor.name` yield a
  display name.
- `emp_id` is 0 on all accounts, so no employee-table join is possible.

## B. Flow

```
Launch
  │
  └─ login gate (the first screen)
        ├─ "ตั้งค่าการเชื่อมต่อ" opens the connection dialog
        │     └─ connection form (host · port · database · read-only user · password)
        │           └─ connect → SELECT 1 → encrypt & save → pool
        │
        ├─ saved config is re-probed at startup
        │     ├─ reconnect: load vault → connect → SELECT 1
        │     └─ on failure: the login gate reports the connection error
        │
        └─ officer login (username + password)
              ├─ fetch one officer row by login name
              ├─ check active = 'Y'
              ├─ check auth_type = 1 (Local Database)
              ├─ verify MD5(password) against officer_login_password_md5
              └─ store an in-memory SessionUser; the dashboard opens
```

The login gate handles only officer credentials; connection setup lives in
the settings dialog. The connection must be usable before login succeeds.
The settings dialog can replace the connection later; changing the saved
connection clears the backend session and returns the operator to the login
gate.

## C. Password verification

The app computes `MD5(password)` in Rust and compares it with the stored
32-hex digest, case-insensitively. The password is moved into a
`secrecy::SecretString` for the verification window; it is never written to
disk, logged, or returned to the frontend.

> **Unverified assumption.** The 2026-10-21 survey confirmed the *shape*
> (MD5, 32 hex) but could not confirm from HOSxP source code what string is
> hashed (salt, encoding, or normalization). Before the pilot, test one
> known account with read-only access. If the real algorithm differs, login
> fails for every account and this assumption must be corrected first.
> Thai passwords would also require confirming the source encoding
> (UTF-8 vs TIS-620).

## D. Session and roles

- `SessionUser` is kept in a process-wide `tokio::sync::RwLock` inside the
  Tauri process only. Restarting the app requires signing in again.
- The frontend never receives the password hash.
- Commands that read dashboard data (`get_dashboard_data`,
  `get_drug_monthly_qty`, `get_antibiotic_usage`, `get_top_drugs`,
  `get_available_years`, `get_drug_list`) and the local PDF write
  (`save_pdf_file`) require a session at the command boundary.
- `login`, `logout`, `current_session`, `load_config`, `save_db_config`, and
  `connect_db` do not require a session: connection setup must work before
  the operator can sign in.
- The session role is `STAFF` for everyone until app-specific roles are
  introduced. HOSxP group permissions are not consulted.
- `connect_db` compares the incoming settings with the encrypted config on
  disk. A changed connection clears the session, so an operator cannot keep
  access after pointing the app at a different database. Re-testing the same
  connection keeps the session.

## E. Query

The only login query is `FETCH_OFFICER_SQL` in
`src-tauri/src/auth.rs`:

```sql
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
LIMIT 1
```

It is a single `SELECT` with one bound parameter; `opduser` and the
plaintext password column never appear. The header shows this display name;
if both the officer name fields and the doctor name are blank, the backend
falls back to the login name.

## F. Why the proposed app_* tables are not created

The sibling survey proposed `v_hosxp_user`, `v_hosxp_user_permission`,
`v_hosxp_online`, `app_user`, `app_session`, `app_login_log`, `app_role`,
and `app_role_permission`. None are created:

- `CREATE VIEW`/`CREATE TABLE` are writes, and HOSxP Dash must never write to
  HOSxP (README "direct read").
- HOSxP Dash has no separate application database. Adding one is a product
  decision, not part of this milestone.
- A persistent login log would duplicate HOSxP `officer_activity_log` and
  would store staff identifiers locally; the current privacy rule is to
  avoid that until the hospital approves a local audit store.
- Online status remains an optional read (`onlineuser`) if it is ever needed
  in the UI; it is not identity.

If a local application database is approved later, the proposed app tables
can be introduced there without touching HOSxP.

## G. Acceptance checklist

- [ ] Verify `MD5(password)` on one real test account (read-only).
- [ ] Confirm the test account's password encoding (ASCII/UTF-8/TIS-620).
- [ ] Confirm `officer_authentication_type_id` really means 1 = Local
      Database on this instance.
- [ ] Confirm the display-name sources: `officer_pname`, `officer_fname`,
      `officer_lname`, and the `doctor.name` fallback via `officer_doctor_code`.
- [ ] Confirm whether the 221 `opduser` users were migrated to `officer`;
      users who only exist in `opduser` cannot sign in.
- [ ] Test active/inactive and LDAP rejection paths with test data.
- [ ] Confirm the read-only MySQL user can `SELECT officer` (no write grant).
