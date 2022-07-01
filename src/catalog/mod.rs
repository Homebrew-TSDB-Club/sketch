pub mod schema;

use std::sync::{Arc, RwLock};

use hashbrown::HashMap;

use self::schema::Schema;

const DEFAULT_RESOURCE_NAME: &str = "_default";

#[derive(Debug)]
pub struct Catalog {
    pub schemas: RwLock<HashMap<String, Arc<Schema>>>,
}

impl Catalog {
    pub fn new() -> Self {
        let mut schemas = HashMap::new();
        schemas.insert(String::from(DEFAULT_RESOURCE_NAME), Arc::new(Schema::new()));
        Self {
            schemas: RwLock::new(schemas),
        }
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<Arc<Schema>> {
        self.schemas.read().unwrap().get(name).cloned()
    }

    #[inline]
    pub fn get_default(&self) -> Arc<Schema> {
        self.schemas.read().unwrap().get(DEFAULT_RESOURCE_NAME).unwrap().clone()
    }
}

#[derive(Debug)]
pub struct CatalogList {
    pub catalogs: RwLock<HashMap<String, Arc<Catalog>>>,
}

impl CatalogList {
    pub fn new() -> Self {
        let mut list = HashMap::new();
        list.insert(String::from(DEFAULT_RESOURCE_NAME), Arc::new(Catalog::new()));
        Self {
            catalogs: RwLock::new(list),
        }
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<Arc<Catalog>> {
        self.catalogs.read().unwrap().get(name).cloned()
    }

    #[inline]
    pub fn get_default(&self) -> Arc<Catalog> {
        self.catalogs
            .read()
            .unwrap()
            .get(DEFAULT_RESOURCE_NAME)
            .unwrap()
            .clone()
    }
}
