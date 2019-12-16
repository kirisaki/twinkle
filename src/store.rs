use std::io::{Write, BufWriter, Read, BufReader};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, MutexGuard};

use crate::types::*;
use crate::errors::*;

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
    pub async fn serialize<W: Write>(self, w: W) -> Result<(), TwinkleError> {
        let store = self.store.lock().await.clone();
        let mut writer = BufWriter::new(w);
        for (k, v) in store {
            match writer.write(&encode(k)) {
                Ok(_) => {},
                Err(_) => return Err(TwinkleError::FailedSerialization),
            };
            match writer.write(&encode(v)) {
                Ok(_) => {},
                Err(_) => return Err(TwinkleError::FailedSerialization),
            };
        };
        let _ = writer.flush();
        Ok(())
    }
}

fn encode(mut data: Vec<u8>) -> Bytes {
    let mut buf = vec![];
    let lb = data.len().to_be_bytes();
    buf.append(&mut vec![lb[6], lb[7]]);
    buf.append(&mut data);
    buf
}

