//! SQLite-backed session persistence.

use crate::session_store::SessionStorePort;
use jereko_core::{Session, SessionId};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    status TEXT NOT NULL,
    provider_id TEXT,
    messages_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

/// Durable session store backed by SQLite.
pub struct SqliteSessionStore {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl SqliteSessionStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create sqlite parent dir: {e}"))?;
        }
        let conn = Connection::open(&path).map_err(|e| format!("open sqlite: {e}"))?;
        conn.execute_batch(SCHEMA)
            .map_err(|e| format!("migrate sqlite schema: {e}"))?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn upsert(conn: &Connection, session: &Session) -> Result<(), String> {
        let status = serde_json::to_value(session.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| format!("{:?}", session.status).to_lowercase());
        let messages_json = serde_json::to_string(&session.messages)
            .map_err(|e| format!("serialize messages: {e}"))?;
        conn.execute(
            r#"
            INSERT INTO sessions (id, status, provider_id, messages_json, updated_at)
            VALUES (?1, ?2, ?3, ?4, datetime('now'))
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                provider_id = excluded.provider_id,
                messages_json = excluded.messages_json,
                updated_at = datetime('now')
            "#,
            params![session.id.0, status, session.provider_id, messages_json],
        )
        .map_err(|e| format!("upsert session: {e}"))?;
        Ok(())
    }

    fn load(conn: &Connection, id: &SessionId) -> Result<Option<Session>, String> {
        let mut stmt = conn
            .prepare("SELECT id, status, provider_id, messages_json FROM sessions WHERE id = ?1")
            .map_err(|e| format!("prepare get: {e}"))?;
        let mut rows = stmt
            .query(params![id.0])
            .map_err(|e| format!("query get: {e}"))?;
        let Some(row) = rows.next().map_err(|e| format!("row: {e}"))? else {
            return Ok(None);
        };
        let id_str: String = row.get(0).map_err(|e| e.to_string())?;
        let status_str: String = row.get(1).map_err(|e| e.to_string())?;
        let provider_id: Option<String> = row.get(2).map_err(|e| e.to_string())?;
        let messages_json: String = row.get(3).map_err(|e| e.to_string())?;
        let status = serde_json::from_value(serde_json::Value::String(status_str.clone()))
            .or_else(|_| serde_json::from_str(&format!("\"{status_str}\"")))
            .map_err(|e| format!("parse status: {e}"))?;
        let messages =
            serde_json::from_str(&messages_json).map_err(|e| format!("parse messages: {e}"))?;
        Ok(Some(Session {
            id: SessionId(id_str),
            status,
            messages,
            provider_id,
        }))
    }
}

impl SessionStorePort for SqliteSessionStore {
    fn insert(&self, session: Session) -> SessionId {
        let id = session.id.clone();
        let conn = self.conn.lock().expect("sqlite lock poisoned");
        Self::upsert(&conn, &session).expect("sqlite insert");
        id
    }

    fn get(&self, id: &SessionId) -> Option<Session> {
        let conn = self.conn.lock().expect("sqlite lock poisoned");
        Self::load(&conn, id).expect("sqlite get")
    }

    fn update(&self, session: Session) -> bool {
        let conn = self.conn.lock().expect("sqlite lock poisoned");
        let existed = Self::load(&conn, &session.id)
            .expect("sqlite get before update")
            .is_some();
        Self::upsert(&conn, &session).expect("sqlite update");
        existed
    }

    fn delete(&self, id: &SessionId) -> bool {
        self.conn
            .lock()
            .expect("sqlite lock")
            .execute("DELETE FROM sessions WHERE id = ?1", [&id.0])
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    fn list_ids(&self) -> Vec<SessionId> {
        let conn = self.conn.lock().expect("sqlite lock");
        let mut stmt = match conn.prepare("SELECT id FROM sessions") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).map(SessionId).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn len(&self) -> usize {
        let conn = self.conn.lock().expect("sqlite lock poisoned");
        conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| {
            row.get::<_, i64>(0)
        })
        .map(|n| n as usize)
        .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jereko_core::{Message, MessageRole, SessionStatus};
    use tempfile::TempDir;

    #[test]
    fn persists_and_reloads_across_open() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sessions.db");

        let id = {
            let store = SqliteSessionStore::open(&path).unwrap();
            let mut session = Session::new();
            session.status = SessionStatus::Active;
            session.provider_id = Some("openai".into());
            session.messages.push(Message {
                role: MessageRole::User,
                content: "hello".into(),
                provider: None,
            });
            let id = store.insert(session.clone());
            assert_eq!(store.get(&id).unwrap().messages.len(), 1);
            id
        };

        let reopened = SqliteSessionStore::open(&path).unwrap();
        let loaded = reopened.get(&id).expect("session survives reopen");
        assert_eq!(loaded.provider_id.as_deref(), Some("openai"));
        assert_eq!(loaded.messages[0].content, "hello");
        assert_eq!(reopened.len(), 1);
    }

    #[test]
    fn update_replaces_messages() {
        let dir = TempDir::new().unwrap();
        let store = SqliteSessionStore::open(dir.path().join("s.db")).unwrap();
        let mut session = Session::new();
        let id = store.insert(session.clone());
        session.messages.push(Message {
            role: MessageRole::Assistant,
            content: "hi".into(),
            provider: Some("openai".into()),
        });
        assert!(store.update(session));
        assert_eq!(store.get(&id).unwrap().messages.len(), 1);
    }

    #[test]
    fn deletes_and_lists_ids() {
        let dir = TempDir::new().unwrap();
        let store = SqliteSessionStore::open(dir.path().join("s.db")).unwrap();
        let a = store.insert(Session::new());
        let b = store.insert(Session::new());
        let ids = store.list_ids();
        assert_eq!(ids.len(), 2);
        assert!(store.delete(&a));
        assert!(!store.delete(&a));
        assert_eq!(store.list_ids(), vec![b]);
    }
}
