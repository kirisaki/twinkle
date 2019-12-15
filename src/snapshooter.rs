use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::*;

pub struct Snapshooter {
    pub store: Arc<Mutex<Store>>,
}
