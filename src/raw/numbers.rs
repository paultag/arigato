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
use std::io::{Cursor, Error, Read, Write};

macro_rules! define_de_re_hydrate {
    ($b:expr, $ty:ty) => {
        impl<T> Hydrate<T> for $ty
        where
            Self: Sized,
            T: AsRef<[u8]>,
        {
            type Error = Error;

            fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
                let mut v = $b;
                b.read_exact(&mut v)?;
                Ok(<$ty>::from_le_bytes(v))
            }
        }

        impl Dehydrate for $ty
        where
            Self: Sized,
        {
            type Error = Error;

            fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
                b.write_all(&self.to_le_bytes())
            }
        }
    };
}

define_de_re_hydrate!([0u8; 1], u8);
define_de_re_hydrate!([0u8; 2], u16);
define_de_re_hydrate!([0u8; 4], u32);
define_de_re_hydrate!([0u8; 8], u64);

#[cfg(test)]
mod tests {
    use super::{super::test_round_trip, Dehydrate, Hydrate};
    use std::io::Cursor;

    test_round_trip!(round_trip_u8, u8, u8, (0, 1, 0xFF));
    test_round_trip!(round_trip_u16, u16, u16, (0, 1, 0xFFFF));
    test_round_trip!(round_trip_u32, u32, u32, (0, 1, 0xFFFFFFFF));
    test_round_trip!(round_trip_u64, u64, u64, (0, 1, 0xFFFFFFFFFFFFFFFF));
}

// vim: foldmethod=marker
