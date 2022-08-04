use std::fmt::Debug;

use crate::column::MutableChunk;

// sharding over table or table cover sharding?

#[derive(Debug)]
struct TableShard {
    mutable_chunks: Vec<MutableChunk>,
}

#[derive(Debug)]
pub struct Table {
    name: String,
    shards: Vec<TableShard>,
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        self as *const Self == other as *const Self
    }
}

impl Table {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}
