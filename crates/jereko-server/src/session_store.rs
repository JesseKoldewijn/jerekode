use jereko_core::{Session, SessionId};
use std::collections::HashMap;
use std::sync::RwLock;

/// Session persistence seam — in-memory and SQLite adapters.
pub trait SessionStorePort: Send + Sync {
    fn insert(&self, session: Session) -> SessionId;
    fn get(&self, id: &SessionId) -> Option<Session>;
    fn update(&self, session: Session) -> bool;
    fn delete(&self, id: &SessionId) -> bool;
    fn list_ids(&self) -> Vec<SessionId>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory session store (default for tests and ephemeral serve).
#[derive(Debug, Default)]
pub struct SessionStore {
    inner: RwLock<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SessionStorePort for SessionStore {
    fn insert(&self, session: Session) -> SessionId {
        let id = session.id.clone();
        self.inner
            .write()
            .expect("session store lock poisoned")
            .insert(id.0.clone(), session);
        id
    }

    fn get(&self, id: &SessionId) -> Option<Session> {
        self.inner
            .read()
            .expect("session store lock poisoned")
            .get(&id.0)
            .cloned()
    }

    fn update(&self, session: Session) -> bool {
        self.inner
            .write()
            .expect("session store lock poisoned")
            .insert(session.id.0.clone(), session)
            .is_some()
    }

    fn delete(&self, id: &SessionId) -> bool {
        self.inner
            .write()
            .expect("session store lock poisoned")
            .remove(&id.0)
            .is_some()
    }

    fn list_ids(&self) -> Vec<SessionId> {
        self.inner
            .read()
            .expect("session store lock poisoned")
            .keys()
            .cloned()
            .map(SessionId)
            .collect()
    }

    fn len(&self) -> usize {
        self.inner
            .read()
            .expect("session store lock poisoned")
            .len()
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
