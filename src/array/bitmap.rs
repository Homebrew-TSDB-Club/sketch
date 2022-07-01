const BIT_MASK: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
const UNSET_BIT_MASK: [u8; 8] = [
    255 - 1,
    255 - 2,
    255 - 4,
    255 - 8,
    255 - 16,
    255 - 32,
    255 - 64,
    255 - 128,
];

#[inline]
fn set_bit(byte: u8, i: usize, value: bool) -> u8 {
    if value {
        byte | BIT_MASK[i]
    } else {
        byte & UNSET_BIT_MASK[i]
    }
}

#[inline]
fn is_set(byte: u8, i: usize) -> bool {
    (byte & BIT_MASK[i]) != 0
}

#[inline]
fn get_bit(data: &[u8], i: usize, align: usize) -> bool {
    is_set(data[i / 8], i % 8 + align)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct BitmapRefMut<'a> {
    buffer: &'a mut [u8],
    length: usize,
    align: usize,
}

impl<'a> BitmapRefMut<'a> {
    #[inline]
    pub(crate) fn insert(&mut self, offset: usize, value: bool) {
        let byte = &mut self.buffer[offset / 8];
        *byte = set_bit(*byte, offset % 8, value);
    }

    #[inline]
    pub(crate) fn get_bit(&self, offset: usize) -> bool {
        get_bit(self.buffer, offset, self.align)
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.length
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct BitmapRef<'a> {
    buffer: &'a [u8],
    length: usize,
    align: usize,
}

impl<'a> BitmapRef<'a> {
    #[inline]
    pub(crate) fn get_bit(&self, offset: usize) -> bool {
        get_bit(self.buffer, offset, self.align)
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.length
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Bitmap {
    buffer: Vec<u8>,
    length: usize,
}

impl Bitmap {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub(crate) fn push(&mut self, value: bool) {
        if self.length % 8 == 0 {
            self.buffer.push(0);
        }
        let byte = self.buffer.as_mut_slice().last_mut().unwrap();
        *byte = set_bit(*byte, self.length % 8, value);
        self.length += 1;
    }

    #[inline]
    pub(crate) fn slice(&self, start: usize, end: usize) -> BitmapRef<'_> {
        BitmapRef {
            buffer: &self.buffer[(start / 8)..((end + 7) / 8)],
            length: end - start,
            align: start % 8,
        }
    }

    #[inline]
    pub(crate) fn slice_mut(&mut self, start: usize, end: usize) -> BitmapRefMut<'_> {
        BitmapRefMut {
            buffer: &mut self.buffer[(start / 8)..((end + 7) / 8)],
            length: end - start,
            align: start % 8,
        }
    }

    #[inline]
    pub(crate) fn add(&mut self, another: BitmapRef<'_>) {
        for i in 0..another.len() {
            self.push(another.get_bit(i));
        }
    }

    #[inline]
    pub(crate) fn as_ref(&self) -> BitmapRef<'_> {
        BitmapRef {
            buffer: &self.buffer,
            length: self.length,
            align: 0,
        }
    }

    #[inline]
    pub(crate) fn align(&mut self) {
        self.length = self.buffer.len() * 8;
    }
}

impl From<Vec<bool>> for Bitmap {
    #[inline]
    fn from(bits: Vec<bool>) -> Self {
        let mut bitmap = Bitmap::new();
        for bit in bits {
            bitmap.push(bit)
        }
        bitmap
    }
}

#[cfg(test)]
mod tests {
    use crate::array::bitmap::Bitmap;

    #[test]
    fn test_bitmap() {
        let mut bitmap = Bitmap::new();
        bitmap.push(true);
        bitmap.push(false);
        bitmap.push(false);
        bitmap.push(true);
        {
            let bitmap_ref = bitmap.slice(0, 2);
            assert!(bitmap_ref.get_bit(0) == true);
            assert!(bitmap_ref.get_bit(1) == false);
            let bitmap_ref_2 = bitmap.slice(2, 3);
            assert!(bitmap_ref_2.get_bit(0) == false);
            assert!(bitmap_ref_2.get_bit(1) == true);
        }
        let re_mut = bitmap.slice_mut(0, 2);
        assert!(re_mut.get_bit(0) == true);
        assert!(re_mut.get_bit(1) == false);
    }
}
