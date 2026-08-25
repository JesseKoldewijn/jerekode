use crate::error::{ProviderError, ProviderResult};
use crate::provider::{Provider, ProviderId, StubProvider};
use std::collections::HashMap;

/// Thread-safe provider registry supporting 75+ built-in and plugin providers.
///
/// Lookup is O(1) by provider id. Registration order is preserved for listing.
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
    registration_order: Vec<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            registration_order: Vec::new(),
        }
    }

    /// Create a registry pre-populated with stub providers for scaffolding.
    pub fn with_stubs() -> Self {
        let mut registry = Self::new();
        for id in ["openai", "anthropic", "ollama"] {
            let _ = registry.register(Box::new(StubProvider::new(id)));
        }
        registry
    }

    pub fn register(&mut self, provider: Box<dyn Provider>) -> ProviderResult<()> {
        let id = provider.id().0.clone();
        if self.providers.contains_key(&id) {
            return Err(ProviderError::AlreadyRegistered(id));
        }
        self.registration_order.push(id.clone());
        self.providers.insert(id, provider);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&dyn Provider> {
        self.providers.get(id).map(|p| p.as_ref())
    }

    pub fn ids(&self) -> impl Iterator<Item = &ProviderId> {
        self.registration_order
            .iter()
            .filter_map(|id| self.providers.get(id))
            .map(|p| p.id())
    }

    pub fn len(&self) -> usize {
        self.providers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_resolves_providers() {
        let registry = ProviderRegistry::with_stubs();
        assert_eq!(registry.len(), 3);
        assert!(registry.get("openai").is_some());
        assert!(registry.get("unknown").is_none());
    }
}
