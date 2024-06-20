use std::collections::HashMap;

use super::{kv_error::KVError, stored_type::StoredType};

pub trait KVDatabase {
    fn read(&self, key: String) -> Result<StoredType, KVError>;
    fn write(&mut self, key: String, value: StoredType) -> Result<(), KVError>;
}

impl KVDatabase for HashMap<String, StoredType> {
    fn read(&self, key: String) -> Result<StoredType, KVError> {
        match self.get(&key) {
            Some(value) => Ok(value.clone()),
            None => Err(KVError::not_found()),
        }
    }

    fn write(&mut self, key: String, value: StoredType) -> Result<(), KVError> {
        self.insert(key, value);
        Ok(())
    }
}
