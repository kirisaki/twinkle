use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use std::io;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::types::*;
use crate::store::*;


pub struct Snapshooter<P: AsRef<Path>> {
    pub store: Arc<Mutex<Store>>,
    pub path: P,
    pub duration: Duration,
}

impl <P: AsRef<Path>> Snapshooter<P> {

}
