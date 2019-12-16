use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, MutexGuard};

use crate::types::*;

#[derive(Clone)]
pub struct Store {
    store: Arc<Mutex<HashMap<Bytes, Bytes>>>,
}

impl Store {
    pub fn new() -> Store {
        Store{store: Arc::new(Mutex::new(HashMap::new()))}
    }
    pub async fn lock(&'_ self) -> MutexGuard<'_, HashMap<Bytes, Bytes>> {
        self.store.lock().await
    }

}
