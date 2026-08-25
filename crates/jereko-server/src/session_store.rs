use jereko_core::{Session, SessionId};
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory session store (Phase 1). SQLite persistence stub in Phase 4.
#[derive(Debug, Default)]
pub struct SessionStore {
    inner: RwLock<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, session: Session) -> SessionId {
        let id = session.id.clone();
        self.inner
            .write()
            .expect("session store lock poisoned")
            .insert(id.0.clone(), session);
        id
    }

    pub fn get(&self, id: &SessionId) -> Option<Session> {
        self.inner
            .read()
            .expect("session store lock poisoned")
            .get(&id.0)
            .cloned()
    }

    pub fn update(&self, session: Session) -> bool {
        self.inner
            .write()
            .expect("session store lock poisoned")
            .insert(session.id.0.clone(), session)
            .is_some()
    }

    pub fn len(&self) -> usize {
        self.inner
            .read()
            .expect("session store lock poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_retrieves_sessions() {
        let store = SessionStore::new();
        let session = Session::new();
        let id = session.id.clone();
        store.insert(session.clone());
        assert_eq!(store.get(&id).unwrap().id, id);
    }
}
