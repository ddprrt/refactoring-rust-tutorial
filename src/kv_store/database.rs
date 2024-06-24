use std::{collections::HashMap, sync::RwLock};

use super::{kv_error::KVError, stored_type::StoredType};

pub trait KVDatabase {
    fn read(&self, key: String) -> Result<StoredType, KVError>;
    fn insert(&mut self, key: String, value: StoredType) -> Result<(), KVError>;
}

impl KVDatabase for HashMap<String, StoredType> {
    fn read(&self, key: String) -> Result<StoredType, KVError> {
        match self.get(&key) {
            Some(value) => Ok(value.clone()),
            None => Err(KVError::not_found()),
        }
    }

    fn insert(&mut self, key: String, value: StoredType) -> Result<(), KVError> {
        self.insert(key, value);
        Ok(())
    }
}

impl KVDatabase for RwLock<HashMap<String, StoredType>> {
    fn read(&self, key: String) -> Result<StoredType, KVError> {
        match self.read()?.get(&key) {
            Some(value) => Ok(value.clone()),
            None => Err(KVError::not_found()),
        }
    }

    fn insert(&mut self, key: String, value: StoredType) -> Result<(), KVError> {
        self.write()?.insert(key, value);
        Ok(())
    }
}
