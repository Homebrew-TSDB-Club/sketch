use std::sync::{Arc, RwLock};

use hashbrown::HashMap;

use crate::source::Table;

#[derive(Debug)]
pub struct Schema {
    tables: RwLock<HashMap<String, Arc<Table>>>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            tables: RwLock::new(HashMap::new()),
        }
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<Arc<Table>> {
        self.tables.read().unwrap().get(name).cloned()
    }
}
