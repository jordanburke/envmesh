// Storage module for encrypted environment variables
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Type alias for change records: (key, value, timestamp, machine_id, deleted)
pub type ChangeRecord = (String, String, i64, String, bool);

pub struct EnvStorage {
    conn: Connection,
}

impl EnvStorage {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS env_vars (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                machine_id TEXT NOT NULL,
                deleted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON env_vars(timestamp)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn get(&self, key: &str) -> Result<Option<(String, i64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT value, timestamp, machine_id FROM env_vars
             WHERE key = ? AND deleted = 0",
        )?;

        let result = stmt.query_row(params![key], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        });

        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set(&self, key: &str, value: &str, machine_id: &str) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO env_vars (key, value, timestamp, machine_id, deleted)
             VALUES (?, ?, ?, ?, 0)",
            params![key, value, timestamp, machine_id],
        )?;

        Ok(())
    }

    pub fn delete(&self, key: &str, _machine_id: &str) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        self.conn.execute(
            "UPDATE env_vars SET deleted = 1, timestamp = ?
             WHERE key = ?",
            params![timestamp, key],
        )?;

        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<(String, String, i64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, timestamp, machine_id FROM env_vars
             WHERE deleted = 0 ORDER BY key",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    pub fn get_changes_since(&self, timestamp: i64) -> Result<Vec<ChangeRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, timestamp, machine_id, deleted FROM env_vars
             WHERE timestamp > ? ORDER BY timestamp",
        )?;

        let rows = stmt.query_map(params![timestamp], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get::<_, i32>(4)? != 0,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}
