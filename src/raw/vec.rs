// {{{ Copyright (c) Paul R. Tagliamonte <paultag@gmail.com>, 2023-2024
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE. }}}

use super::{Dehydrate, Hydrate};
use std::{
    any::TypeId,
    io::{Cursor, Write},
    num::TryFromIntError,
};

/// Error decoding a Slice.
#[derive(Debug)]
pub enum SliceError<T> {
    /// Larger than the configured msize.
    TooLong,

    /// Underlying i/o error.
    IoError(std::io::Error),

    /// Some inner error T.
    Inner(T),
}

impl<T> From<std::io::Error> for SliceError<T> {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl<T> From<TryFromIntError> for SliceError<T> {
    fn from(_e: TryFromIntError) -> Self {
        Self::TooLong
    }
}

impl<CursorT, T> Hydrate<CursorT> for Vec<T>
where
    Self: Sized,
    CursorT: AsRef<[u8]>,
    T: Hydrate<CursorT>,
{
    type Error = SliceError<T::Error>;
    fn hydrate(b: &mut Cursor<CursorT>) -> Result<Self, Self::Error> {
        let len = u16::hydrate(b)? as usize;
        let mut buf: Self = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(T::hydrate(b).map_err(SliceError::Inner)?);
        }
        Ok(buf)
    }
}

impl<T> Dehydrate for &[T]
where
    Self: Sized,
    T: Dehydrate,
    T: 'static,
{
    type Error = SliceError<T::Error>;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        let size: u16 = self.len().try_into()?;
        size.dehydrate(b)?;

        if TypeId::of::<T>() == TypeId::of::<u8>() {
            let slf = unsafe { &*(*self as *const [T] as *const [u8]) };
            b.write_all(slf)?;
        } else {
            for d in self.iter() {
                d.dehydrate(b).map_err(SliceError::Inner)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_round_trip, Dehydrate, Hydrate};
    use crate::raw::{FileType, Qid};
    use std::io::Cursor;

    test_round_trip!(round_trip_vec_u16, &[u16], Vec<u16>, (&[0xABCD, 0xDEFA]));
    test_round_trip!(
        round_trip_vec_qid,
        &[Qid],
        Vec<Qid>,
        (
            &[Qid::new(FileType::Tmp, 0, 0)],
            &[Qid::new(FileType::Tmp, 2, 3), Qid::new(FileType::Dir, 5, 6)]
        )
    );
}

// vim: foldmethod=marker
