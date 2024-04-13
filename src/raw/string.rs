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

use super::{Dehydrate, Hydrate, SliceError};
use std::{
    io::{Cursor, Error},
    num::TryFromIntError,
    str::Utf8Error,
};

///
#[derive(Debug)]
pub enum StringError {
    ///
    TooLarge,

    ///
    IoError(Error),

    ///
    UnicodeError(Utf8Error),
}

impl From<Utf8Error> for StringError {
    fn from(e: Utf8Error) -> Self {
        Self::UnicodeError(e)
    }
}

impl From<Error> for StringError {
    fn from(e: Error) -> Self {
        Self::IoError(e)
    }
}

impl From<TryFromIntError> for StringError {
    fn from(_e: TryFromIntError) -> Self {
        Self::TooLarge
    }
}

impl From<SliceError<std::io::Error>> for StringError {
    fn from(se: SliceError<std::io::Error>) -> Self {
        match se {
            SliceError::Inner(e) => Self::IoError(e),
            SliceError::IoError(e) => Self::IoError(e),
            SliceError::TooLong => Self::TooLarge,
        }
    }
}

impl<T> Hydrate<T> for String
where
    Self: Sized,
    T: AsRef<[u8]>,
{
    type Error = StringError;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
        let buf = Vec::<u8>::hydrate(b)?;
        Ok(std::str::from_utf8(&buf)?.to_owned())
    }
}

impl Dehydrate for &str
where
    Self: Sized,
{
    type Error = StringError;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        let buf = self.as_bytes();
        Ok(buf.dehydrate(b)?)
    }
}

impl Dehydrate for String
where
    Self: Sized,
{
    type Error = StringError;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        self.as_str().dehydrate(b)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_round_trip, Dehydrate, Hydrate};
    use std::io::Cursor;
    test_round_trip!(round_trip_string, &str, String, ("foo bar", "fnord", ""));
}

// vim: foldmethod=marker
