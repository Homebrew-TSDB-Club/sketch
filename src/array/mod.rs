pub(crate) mod bitmap;
pub(crate) mod dictionary;
pub mod scalar;

use std::fmt::Debug;
use std::hash::Hash;

use crate::primitive::Primitive;

use self::bitmap::Bitmap;
use self::dictionary::Dictionary;
use self::scalar::{
    NullableFixedSizeListRef, NullableFixedSizeListRefMut, NullableFixedSizedList, Scalar, ScalarRef, ScalarRefMut,
};

pub trait Array: 'static + Debug + Send + Sync {
    type Item: for<'a> Scalar<Ref<'a> = Self::ItemRef<'a>>;
    type ItemRef<'a>: ScalarRef<'a, Owned = Self::Item>
    where
        Self: 'a;
    type ItemRefMut<'a>: ScalarRefMut<'a, Owned = Self::Item>
    where
        Self: 'a;

    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>>;
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_>;
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>>;
    fn push(&mut self, value: Self::ItemRef<'_>);
    fn push_zero(&mut self);
    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait IdentifiedArray: Array {
    type ID: Eq + Hash;
}

#[derive(Debug)]
pub struct FixedSizedListArray<P: Primitive> {
    data: Vec<P>,
    list_size: usize,
}

impl<P: Primitive> FixedSizedListArray<P> {
    #[inline]
    pub fn new(list_size: usize) -> Self {
        Self {
            list_size,
            data: Vec::new(),
        }
    }

    #[inline]
    fn slice_raw_mut(&mut self, start: usize, end: usize) -> &mut [P] {
        &mut self.data[start..end]
    }

    #[inline]
    fn slice_raw(&self, start: usize, end: usize) -> &[P] {
        &self.data[start..end]
    }
}

impl<P: Primitive> Array for FixedSizedListArray<P> {
    type Item = Vec<P>;
    type ItemRef<'a> = &'a [P];
    type ItemRefMut<'a> = &'a mut [P];

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        if id * self.list_size > self.data.len() {
            None
        } else {
            Some(self.get_unchecked(id))
        }
    }

    #[inline]
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        self.slice_raw(id * self.list_size, (id + 1) * self.list_size)
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        if id * self.list_size > self.data.len() {
            None
        } else {
            Some(self.slice_raw_mut(id * self.list_size, (id + 1) * self.list_size))
        }
    }

    #[inline]
    fn push(&mut self, value: Self::ItemRef<'_>) {
        self.data.extend_from_slice(value);
    }

    #[inline]
    fn push_zero(&mut self) {
        self.push(&vec![Default::default(); self.list_size]);
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len() / self.list_size
    }
}

#[derive(Debug)]
pub struct ListArray<P: Primitive> {
    data: Vec<P>,
    offsets: Vec<usize>,
}

impl<P: Primitive> ListArray<P> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl<P: Primitive> Default for ListArray<P> {
    #[inline]
    fn default() -> Self {
        Self {
            data: Vec::<P>::new(),
            offsets: vec![0],
        }
    }
}

impl<P: Primitive> Array for ListArray<P> {
    type Item = Vec<P>;
    type ItemRef<'a> = &'a [P];
    type ItemRefMut<'a> = &'a mut [P];

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        let offset = self.offsets.get(id)?;
        let end = self.offsets.get(id + 1)?;
        Some(&self.data[*offset..*end])
    }

    #[inline]
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        let offset = self.offsets[id];
        let end = self.offsets[id + 1];
        &self.data[offset..end]
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        let offset = self.offsets.get(id)?;
        let end = self.offsets.get(id + 1)?;
        Some(&mut self.data[*offset..*end])
    }

    #[inline]
    fn push(&mut self, value: Self::ItemRef<'_>) {
        let id = self.offsets.len() - 1;
        let end = self.offsets[id] + value.len();
        self.offsets.push(end);
        self.data.extend_from_slice(value);
    }

    #[inline]
    fn push_zero(&mut self) {
        self.offsets.push(self.offsets[self.offsets.len() - 1]);
    }

    #[inline]
    fn len(&self) -> usize {
        self.offsets.len() - 1
    }
}

#[derive(Debug)]
pub struct NullableFixedSizedListArray<P: Primitive> {
    validity: Bitmap,
    data: FixedSizedListArray<P>,
}

impl<P: Primitive> NullableFixedSizedListArray<P> {
    #[inline]
    pub fn new(list_size: usize) -> Self {
        Self {
            data: FixedSizedListArray::<P>::new(list_size),
            validity: Bitmap::new(),
        }
    }
}

impl<P: Primitive> Array for NullableFixedSizedListArray<P> {
    type Item = NullableFixedSizedList<P>;
    type ItemRef<'a> = NullableFixedSizeListRef<'a, P>;
    type ItemRefMut<'a> = NullableFixedSizeListRefMut<'a, P>;

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        if id * self.data.list_size > self.data.data.len() {
            None
        } else {
            Some(self.get_unchecked(id))
        }
    }

    #[inline]
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        let validity_step = (self.data.list_size + 7) / 8 * 8;
        NullableFixedSizeListRef {
            validity: self.validity.slice(id * validity_step, (id + 1) * validity_step),
            data: self
                .data
                .slice_raw(id * self.data.list_size, (id + 1) * self.data.list_size),
        }
    }

    #[inline]
    fn get_mut(&mut self, offset: usize) -> Option<Self::ItemRefMut<'_>> {
        let (start, end) = (offset * self.data.list_size, (offset + 1) * self.data.list_size);
        Some(NullableFixedSizeListRefMut::new(
            self.validity.slice_mut(start, end),
            self.data.slice_raw_mut(start, end),
        ))
    }

    #[inline]
    fn push(&mut self, value: Self::ItemRef<'_>) {
        self.validity.add(value.validity);
        self.data.data.extend_from_slice(value.data);
    }

    #[inline]
    fn push_zero(&mut self) {
        for _ in 0..self.data.list_size {
            self.validity.push(false);
        }
        self.data.push_zero();
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug)]
pub struct IdArray<A: Array>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>> + Hash,
{
    values: Dictionary<A>,
    data: Vec<usize>,
}

impl<A: Array> IdArray<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>> + Hash,
{
    pub fn new(array: A) -> Self {
        Self {
            values: Dictionary::new(array),
            data: Vec::<usize>::new(),
        }
    }
}

impl<A: Array> Array for IdArray<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>> + Hash,
{
    type Item = Option<A::Item>;
    type ItemRef<'a> = Option<A::ItemRef<'a>>;
    type ItemRefMut<'a> = Option<A::ItemRefMut<'a>>;

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        let vid = self.data.get(id)?;
        self.values.get(*vid)
    }

    #[inline]
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        self.values.get_unchecked(self.data[id])
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        let vid = self.data.get(id)?;
        self.values.get_mut(*vid)
    }

    #[inline]
    fn push(&mut self, value: Self::ItemRef<'_>) {
        match value {
            Some(value) => {
                self.data.push(self.values.lookup_or_insert(value));
            }
            None => {
                self.push_zero();
            }
        }
    }

    #[inline]
    fn push_zero(&mut self) {
        self.data.push(0);
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<A: Array + Default> IdentifiedArray for IdArray<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>> + Hash,
{
    type ID = usize;
}

#[derive(Debug, Default)]
pub struct PrimitiveArray<P: Primitive> {
    data: Vec<P>,
}

impl<P: Primitive> PrimitiveArray<P> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<P: Primitive> Array for PrimitiveArray<P> {
    type Item = P;
    type ItemRef<'a> = &'a P;
    type ItemRefMut<'a> = &'a mut P;

    #[inline]
    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        self.data.get(id)
    }

    #[inline]
    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        unsafe { self.data.get_unchecked(id) }
    }

    #[inline]
    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        self.data.get_mut(id)
    }

    #[inline]
    fn push(&mut self, value: Self::ItemRef<'_>) {
        self.data.push(value.clone())
    }

    #[inline]
    fn push_zero(&mut self) {
        self.data.push(P::default())
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug)]
pub struct ConstFixedSizedListArray<P: Primitive, const SIZE: usize> {
    array: FixedSizedListArray<P>,
}

impl<P: Primitive, const SIZE: usize> Default for ConstFixedSizedListArray<P, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Primitive, const SIZE: usize> ConstFixedSizedListArray<P, SIZE> {
    #[inline]
    pub fn new() -> Self {
        Self {
            array: FixedSizedListArray::new(SIZE),
        }
    }
}

impl<P: Primitive, const SIZE: usize> Array for ConstFixedSizedListArray<P, SIZE> {
    type Item = <FixedSizedListArray<P> as Array>::Item;
    type ItemRef<'a> = <FixedSizedListArray<P> as Array>::ItemRef<'a>;
    type ItemRefMut<'a> = <FixedSizedListArray<P> as Array>::ItemRefMut<'a>;

    fn get(&self, id: usize) -> Option<Self::ItemRef<'_>> {
        self.array.get(id)
    }

    fn get_unchecked(&self, id: usize) -> Self::ItemRef<'_> {
        self.array.get_unchecked(id)
    }

    fn get_mut(&mut self, id: usize) -> Option<Self::ItemRefMut<'_>> {
        self.array.get_mut(id)
    }

    fn push(&mut self, value: Self::ItemRef<'_>) {
        self.array.push(value)
    }

    fn push_zero(&mut self) {
        self.array.push_zero()
    }

    fn len(&self) -> usize {
        self.array.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::array::scalar::NullableFixedSizedList;
    use crate::array::NullableFixedSizedListArray;

    use super::scalar::Scalar;
    use super::{Array, FixedSizedListArray, IdArray, ListArray, PrimitiveArray};

    #[test]
    fn test_fixed_sized_list_array() {
        let mut array = FixedSizedListArray::new(2);
        array.push(&[1, 2]);
        array.push(&[3, 4]);
        assert!(array.get(0) == Some(&[1, 2]));
        assert!(array.get(1) == Some(&[3, 4]));
        array.push_zero();
        assert!(array.get(2) == Some(&[Default::default(); 2]));
    }

    #[test]
    fn test_list_array() {
        let mut array = ListArray::new();
        array.push(&[1, 2]);
        array.push(&[2, 3, 4]);
        assert!(array.get(0) == Some(&[1, 2]));
        assert!(array.get(1) == Some(&[2, 3, 4]));
    }

    #[test]
    fn test_nullable_array() {
        let mut array = NullableFixedSizedListArray::new(2);
        array.push(NullableFixedSizedList::<_>::from(vec![None, Some(1)]).as_ref());
        array.push(NullableFixedSizedList::<_>::from(vec![Some(2), Some(3)]).as_ref());
        assert!(array.get(0).unwrap().get(0) == Some(None));
        assert!(array.get(0).unwrap().get(1) == Some(Some(&1)));
        assert!(array.get(1).unwrap().get(0) == Some(Some(&2)));
        assert!(array.get(1).unwrap().get(1) == Some(Some(&3)));
        let mut ref_mut = array.get_mut(0).unwrap();
        ref_mut.insert(0, Some(1));
        assert!(array.get(0).unwrap().get(0) == Some(Some(&1)));
    }

    #[test]
    fn test_id_array() {
        let mut array = IdArray::<ListArray<u8>>::new(ListArray::<u8>::new());
        array.push(Some("foo".as_ref()));
        array.push(Some("bar".as_ref()));
        array.push(Some("quaz".as_ref()));
        array.push(Some("bar".as_ref()));
        assert!(array.get(0) == Some(Some("foo".as_ref())));
        assert!(array.get(1) == Some(Some("bar".as_ref())));
        assert!(array.get(2) == Some(Some("quaz".as_ref())));
        assert!(array.get(3).unwrap().unwrap().as_ptr() == array.get(1).unwrap().unwrap().as_ptr());
    }

    #[test]
    fn test_primitive_array() {
        let mut array = PrimitiveArray::new();
        array.push(&1);
        array.push(&2);
        array.push(&3);
        assert!(array.get(0) == Some(&1));
        assert!(array.get(1) == Some(&2));
        assert!(array.get(2) == Some(&3));
    }
}
