use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct Db {
    db: sled::Db,
}

impl Db {
    pub fn new(path: &str) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }

    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.db.insert(key.as_bytes(), serde_json::to_vec(value)?)?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let value = self.db.get(key.as_bytes())?;
        let value = value.map(|v| serde_json::from_slice(&v)).transpose()?;
        Ok(value)
    }

    pub fn list<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let imgs: Result<Vec<_>> = self
            .db
            .iter()
            .map(|e| {
                let (_, value) = e?;
                let img = serde_json::from_slice(&value)?;
                Ok(img)
            })
            .collect();
        Ok(imgs?)
    }
}
