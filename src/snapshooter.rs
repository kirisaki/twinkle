use std::path::Path;
use std::fs::File;
use std::time::Duration;
use std::thread::sleep;

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
        sleep(duration);
        store.serialize(File::open(path)?).await?;
        Ok(())
    }
}
