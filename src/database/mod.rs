use crate::types::StoredType;
use std::collections::HashMap;

pub trait KeyValueStore {
    fn get_item(&self, key: String) -> Option<StoredType>;
    fn set_item(&mut self, key: String, value: StoredType);
}

impl KeyValueStore for HashMap<String, StoredType> {
    fn get_item(&self, key: String) -> Option<StoredType> {
        self.get(&key).map(|val| val.to_owned())
    }

    fn set_item(&mut self, key: String, value: StoredType) {
        self.insert(key, value);
    }
}
