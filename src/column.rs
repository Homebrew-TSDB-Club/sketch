use crate::array::{
    Array, ConstFixedSizedListArray, IdArray, IdentifiedArray, ListArray, NullableFixedSizedListArray, PrimitiveArray,
};
use crate::common::{Duration, Instant};
use crate::index::IndexImpl;

pub type UInt8Field = NullableFixedSizedListArray<u8>;
pub type UInt16Field = NullableFixedSizedListArray<u16>;
pub type UInt32Field = NullableFixedSizedListArray<u32>;
pub type UInt64Field = NullableFixedSizedListArray<u64>;
pub type Int8Field = NullableFixedSizedListArray<i8>;
pub type Int16Field = NullableFixedSizedListArray<i16>;
pub type Int32Field = NullableFixedSizedListArray<i32>;
pub type Int64Field = NullableFixedSizedListArray<i64>;
pub type Float32Field = NullableFixedSizedListArray<f32>;
pub type Float64Field = NullableFixedSizedListArray<f64>;
pub type BoolField = NullableFixedSizedListArray<bool>;

pub type StringLabel = IdArray<ListArray<u8>>;
pub type IPv4Label = IdArray<ConstFixedSizedListArray<u8, 4>>;
pub type IPv6Label = IdArray<ConstFixedSizedListArray<u8, 16>>;
pub type IntLabel = IdArray<PrimitiveArray<i64>>;
pub type BoolLabel = IdArray<PrimitiveArray<bool>>;

#[derive(Debug, Clone)]
pub struct LabelColumn<A: IdentifiedArray> {
    array: A,
    index: Vec<IndexImpl<A::ID>>,
}

impl<A: IdentifiedArray> PartialEq for LabelColumn<A> {
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
    }
}

#[derive(Debug, Clone)]
pub struct FieldColumn<A: Array> {
    array: A,
}

impl<A: Array> PartialEq for FieldColumn<A> {
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelImpl {
    String(LabelColumn<StringLabel>),
    IPv4(LabelColumn<IPv4Label>),
    IPv6(LabelColumn<IPv6Label>),
    Int(LabelColumn<IntLabel>),
    Bool(LabelColumn<BoolLabel>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldImpl {
    UInt8(FieldColumn<UInt8Field>),
    UInt16(FieldColumn<UInt16Field>),
    UInt32(FieldColumn<UInt32Field>),
    UInt64(FieldColumn<UInt64Field>),
    Int8(FieldColumn<Int8Field>),
    Int16(FieldColumn<Int16Field>),
    Int32(FieldColumn<Int32Field>),
    Int64(FieldColumn<Int64Field>),
    Float32(FieldColumn<Float32Field>),
    Float64(FieldColumn<Float64Field>),
    Bool(FieldColumn<BoolField>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMeta {
    pub(crate) start_at: Instant,
    time_interval: Duration,
    series_len: u32,
}

impl ChunkMeta {
    #[inline]
    pub(crate) fn end_at(&self) -> Instant {
        self.start_at + self.time_interval * (self.series_len - 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutableChunk {
    pub label: Vec<LabelImpl>,
    pub field: Vec<FieldImpl>,
    pub meta: ChunkMeta,
}
