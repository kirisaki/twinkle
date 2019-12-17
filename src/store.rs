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
    pub async fn serialize<W: Write>(self, inner: W) -> Result<(), TwinkleError> {
        let store = self.store.lock().await.clone();
        let mut writer = BufWriter::new(inner);
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
    pub fn deserialize<R: Read>(inner: R) -> Result<Store, TwinkleError> {
        let mut lh: u8 = 0;
        let mut kv = KeyOrVakue::Key;
        let mut st = ReadingState::AwaitHighByte;
        let mut store = HashMap::new();
        let mut keybuf  = vec![];
        let mut valbuf = vec![];
        for r in BufReader::new(inner).bytes() {
            let byte = match r {
                Ok(v) => v,
                Err(_) => return Err (TwinkleError::FailedDeserialization),
            };
            match st {
                ReadingState::AwaitHighByte => {
                    lh = byte;
                    st = ReadingState::AwaitLowByte;
                },
                ReadingState::AwaitLowByte => {
                    let ll = byte;
                    let size = u16::from_be_bytes([lh, ll]);
                    lh = 0;
                    st = ReadingState::ReadingData(0, size);
                },
                ReadingState::ReadingData(pos, size) if pos < size - 1 => {
                    match kv {
                        KeyOrVakue::Key => {
                            keybuf.push(byte);
                            st = ReadingState::ReadingData(pos + 1, size)
                        },
                        KeyOrVakue::Value => {
                            valbuf.push(byte);
                            st = ReadingState::ReadingData(pos + 1, size)
                        },
                    };
                },
                ReadingState::ReadingData(..) => {
                    match kv {
                        KeyOrVakue::Key => {
                            keybuf.push(byte);
                            kv = KeyOrVakue::Value;
                            st = ReadingState::AwaitHighByte;
                        },
                        KeyOrVakue::Value => {
                            valbuf.push(byte);
                            kv = KeyOrVakue::Key;
                            st = ReadingState::AwaitHighByte;
                            store.insert(keybuf, valbuf);
                            keybuf = vec![];
                            valbuf = vec![];
                        },
                    }
                },
            };
        };
        Ok(Store{store: Arc::new(Mutex::new(store))})
    }
}

fn encode(mut data: Vec<u8>) -> Bytes {
    let mut buf = vec![];
    let lb = data.len().to_be_bytes();
    buf.append(&mut vec![lb[6], lb[7]]);
    buf.append(&mut data);
    buf
}

#[derive(Debug)]
enum KeyOrVakue {
    Key,
    Value,
}

#[derive(Debug)]
enum ReadingState {
    AwaitHighByte,
    AwaitLowByte,
    ReadingData(u16, u16),
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::store::{Store};
    #[tokio::test]
    async fn test_serialization() {
        let store = Store::new();
        let mut file = vec![];
        {
            let mut s = store.lock().await;
            s.insert(b"foo".to_vec(), b"bar".to_vec());
            s.insert(b"hoge".to_vec(), b"fuga".to_vec());
        };
        {
            store.serialize(&mut file).await.unwrap();
        };
        let handle = Cursor::new(file);
        let s = Store::deserialize(handle).unwrap();
        {
            let hm = s.lock().await;
            assert_eq!(hm.get(&b"foo".to_vec()), Some(&b"bar".to_vec()));
            assert_eq!(hm.get(&b"hoge".to_vec()), Some(&b"fuga".to_vec()));
        };
    }
}
