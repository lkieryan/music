use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use types::errors::Result;

use crate::provider::base::BaseProvider;

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    inner: Arc<RwLock<HashMap<String, Arc<dyn BaseProvider>>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self { Self::default() }

    pub async fn add(&self, key: String, provider: Arc<dyn BaseProvider>) {
        self.inner.write().await.insert(key, provider);
    }

    pub async fn get(&self, key: &str) -> Option<Arc<dyn BaseProvider>> {
        self.inner.read().await.get(key).cloned()
    }

   pub async fn keys(&self) -> Vec<String> {
       self.inner.read().await.keys().cloned().collect()
   }

   pub async fn remove(&self, key: &str) -> Option<Arc<dyn BaseProvider>> {
       self.inner.write().await.remove(key)
   }

   pub async fn initialize_all(&self) {
       let providers: Vec<Arc<dyn BaseProvider>> = self.inner.read().await.values().cloned().collect();
       for p in providers { let _ = p.initialize().await; }
   }
}
