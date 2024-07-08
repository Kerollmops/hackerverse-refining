use std::marker::PhantomData;
use std::mem;

use bytemuck::{AnyBitPattern, PodCastError};

pub struct MatLEView<'m, const DIM: usize, T> {
    bytes: &'m [u8],
    _marker: PhantomData<T>,
}

impl<const DIM: usize, T: AnyBitPattern> MatLEView<'_, DIM, T> {
    pub fn new(bytes: &[u8]) -> MatLEView<DIM, T> {
        assert!((bytes.len() / mem::size_of::<T>()) % DIM == 0);
        MatLEView { bytes, _marker: PhantomData }
    }

    pub fn get(&self, index: usize) -> Option<Result<&[T; DIM], PodCastError>> {
        let tsize = mem::size_of::<T>();
        if (index * DIM + DIM) * tsize < self.bytes.len() {
            let start = index * DIM;
            let bytes = &self.bytes[start * tsize..(start + DIM) * tsize];
            match bytemuck::try_cast_slice::<u8, T>(bytes) {
                Ok(slice) => Some(Ok(slice.try_into().unwrap())),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}
