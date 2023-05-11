use crate::errors::Error;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub trait DB {
    fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: Into<String>;

    fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: Into<String>,
        V: AsRef<[u8]>;
}

#[derive(Debug)]
pub struct FileDB {
    store_dir: PathBuf,
}

impl FileDB {
    pub fn open(store_dir: PathBuf) -> Result<Self, Error> {
        if !store_dir.is_dir() {
            fs::create_dir(store_dir.clone())?;
        }
        Ok(Self { store_dir })
    }
}

impl DB for FileDB {
    fn get<K>(&self, key: K) -> Result<Option<Vec<u8>>, Error>
    where
        K: Into<String>,
    {
        Ok(Some(fs::read(self.store_dir.join(key.into()))?))
    }

    fn put<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: Into<String>,
        V: AsRef<[u8]>,
    {
        let mut f = File::create(self.store_dir.join(key.into()))?;
        Ok(f.write_all(value.as_ref())?)
    }
}
