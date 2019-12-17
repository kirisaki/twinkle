use std::path::Path;
use std::fs::File;

use tokio::time::{Duration, delay_for};
use crate::store::*;


pub struct Snapshooter<P: AsRef<Path>> {
    pub store: Store,
    pub path: P,
    pub duration: Duration,
}

impl <P: AsRef<Path>> Snapshooter<P> {
    pub async fn run(self) -> Result<(), std::io::Error>
    {
        let Snapshooter{store, path, duration} = self;
        loop {
            delay_for(duration).await;
            store.clone().serialize(File::create(&path)?).await?;
        }
    }
}
