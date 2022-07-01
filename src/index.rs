use std::hash::Hash;

use croaring::Bitmap;
use hashbrown::HashMap;
use pdatastructs::filters::bloomfilter::BloomFilter;
use pdatastructs::filters::Filter;

pub trait Index {
    type Value;

    fn lookup(&self, value: &Self::Value, superset: &mut Option<Bitmap>);
    fn insert(&mut self, row: u32, value: Self::Value);
    fn exactly(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct InvertedIndex<V>
where
    V: Eq + Hash,
{
    data: HashMap<V, Bitmap>,
}

impl<V> InvertedIndex<V>
where
    V: Eq + Hash,
{
    #[inline]
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }
}

impl<V> Index for InvertedIndex<V>
where
    V: Eq + Hash,
{
    type Value = V;

    #[inline]
    fn lookup(&self, value: &Self::Value, superset: &mut Option<Bitmap>) {
        self.data.get(value).map(|set| match superset {
            Some(s) => s.and_inplace(set),
            None => *superset = Some(set.clone()),
        });
    }

    #[inline]
    fn insert(&mut self, row: u32, value: Self::Value) {
        let bitmap = self.data.entry(value).or_insert_with(|| Bitmap::create());
        bitmap.add(row);
    }

    #[inline]
    fn exactly(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct SparseIndex<V: Hash> {
    seens: Vec<BloomFilter<V>>,
    block_size: u32,
}

impl<T: Hash> SparseIndex<T> {
    #[inline]
    pub(crate) fn new(block_size: u32) -> Self {
        Self {
            seens: Vec::new(),
            block_size,
        }
    }
}

impl<V: Hash> Index for SparseIndex<V> {
    type Value = V;

    #[inline]
    fn lookup(&self, value: &Self::Value, superset: &mut Option<Bitmap>) {
        let mut bitmap: Box<Bitmap> = Box::new(Bitmap::create());
        for (offset, block) in self.seens.iter().enumerate() {
            if block.query(value) {
                let offset = offset as u32;
                bitmap.add_range((offset * self.block_size)..((offset + 1) * self.block_size));
            }
        }
        match superset {
            Some(s) => s.and_inplace(bitmap.as_ref()),
            None => *superset = Some(bitmap.as_ref().clone()),
        }
    }

    #[inline]
    fn insert(&mut self, row: u32, value: Self::Value) {
        let block = (row / self.block_size) as usize;
        if self.seens.len() <= block {
            self.seens.resize_with(block + 1, || {
                BloomFilter::with_properties(self.block_size as usize, 1.0 / 100.0)
            });
        }
        self.seens[block].insert(&value).unwrap();
    }

    #[inline]
    fn exactly(&self) -> bool {
        false
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum IndexType<Inverted, Sparse> {
    Inverted(Inverted),
    Sparse(Sparse),
}

pub type IndexImpl<V> = IndexType<InvertedIndex<V>, SparseIndex<V>>;

impl<V> IndexImpl<V>
where
    V: Eq + Hash,
{
    pub fn new(data_type: IndexType<(), u32>) -> Self {
        match data_type {
            IndexType::Inverted(_) => IndexImpl::Inverted(InvertedIndex::new()),
            IndexType::Sparse(block_size) => IndexImpl::Sparse(SparseIndex::new(block_size)),
        }
    }
}

#[cfg(test)]
mod tests {
    use croaring::Bitmap;
    use pdatastructs::filters::bloomfilter::BloomFilter;

    use crate::index::InvertedIndex;

    use super::{Index, SparseIndex};

    #[test]
    fn test_bloom_filter() {
        // - input size: we expect 10M elements
        // - reliability: probability of false positives should be <= 1%
        // - CPU efficiency: number of hash functions should be <= 10
        // - RAM efficiency: size should be <= 15MB
        let seen = BloomFilter::<u64>::with_properties(10_000_000, 1.0 / 100.0);
        const BOUND_HASH_FUNCTIONS: usize = 10;
        assert!(
            seen.k() <= BOUND_HASH_FUNCTIONS,
            "number of hash functions for bloom filter should be <= {} but is {}",
            BOUND_HASH_FUNCTIONS,
            seen.k(),
        );
        const BOUND_SIZE_BYTES: usize = 15_000_000;
        let size_bytes = (seen.m() + 7) / 8;
        assert!(
            size_bytes <= BOUND_SIZE_BYTES,
            "size of bloom filter should be <= {} bytes but is {} bytes",
            BOUND_SIZE_BYTES,
            size_bytes,
        );
    }

    #[test]
    fn test_sparse_index() {
        let mut index = SparseIndex::<usize>::new(1000);
        index.insert(0, 1);
        index.insert(1001, 1);
        let mut result = None;
        index.lookup(&1, &mut result);
        let mut expect = Bitmap::create();
        expect.add_range(0..2000);
        assert!(result == Some(expect));
    }

    #[test]
    fn test_inverted_index() {
        let mut index = InvertedIndex::<usize>::new();
        index.insert(0, 1);
        index.insert(1001, 1);
        let mut result = None;
        index.lookup(&1, &mut result);
        let mut expect = Bitmap::create();
        expect.add(0);
        expect.add(1001);
        assert!(result == Some(expect));
    }

    #[test]
    fn test_fusion_index() {
        let mut index_1 = SparseIndex::<usize>::new(1);
        let mut index_2 = InvertedIndex::<usize>::new();
        index_1.insert(0, 0);
        index_1.insert(1, 1);
        index_2.insert(0, 1);
        index_2.insert(1, 1);
        let mut result = None;
        index_1.lookup(&1, &mut result);
        index_2.lookup(&1, &mut result);
        let mut b = Bitmap::create();
        b.add(1);
        assert!(result == Some(b));
    }
}
