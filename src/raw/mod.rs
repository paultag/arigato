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

//! This module contains raw protocol level primitives. This is to be used by
//! something doing i/o between client and server.

mod messages_r;
mod messages_t;
mod numbers;
mod protocol;
mod stat;
mod string;
mod vec;
mod version;

pub use messages_r::{RError, R};
pub use messages_t::{TError, T};
pub use protocol::{Fid, FileType, IoDirection, OpenMode, Qid, Tag, Type};
pub use stat::{Stat, StatError};
pub use string::StringError;
pub use vec::SliceError;
pub use version::{Version, VersionError};

use std::io::Cursor;

/// Hydrate is used to take bytes and produce an object from.
pub trait Hydrate<T>
where
    Self: Sized,
    T: AsRef<[u8]>,
{
    /// Error to be returned during Hydrate routines.
    type Error;

    /// Read bytes from the Cursor to create a new object.
    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error>;
}

/// Dehydrate is used to take an object and turn it into bytes.
pub trait Dehydrate
where
    Self: Sized,
{
    /// Error to be returned during Dehydrate routines.
    type Error;

    /// Write bytes from the Cursor to create a new object.
    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error>;
}

#[allow(unused_macros)]
macro_rules! test_round_trip {
    ($name:ident, $dehy_ty:ty, $hyd_ty:ty, ($( $num:expr ),+)) => {
        #[test]
        fn $name() {
            $(

            let mut b = Cursor::new(vec![0u8; 1024]);
            let v: $dehy_ty = $num;
            v.dehydrate(&mut b).unwrap();

            let pos = b.position() as usize;
            let vec = b.into_inner();
            let mut b = Cursor::new(&vec[..pos]);

            let v1 = <$hyd_ty>::hydrate(&mut b).unwrap();
            assert_eq!(v, v1, "{:?} != {:?} after hydrate/dehydrate", v, v1);
            )+
        }
    };
}
#[allow(unused_imports)]
use test_round_trip;

#[allow(unused_macros)]
macro_rules! test_round_trips {
    ($dehy_ty:ty, $hyd_ty:ty, ($( $name:ident: $num:expr ),+)) => {
        $(

        crate::raw::test_round_trip!($name, $dehy_ty, $hyd_ty, ( $num ));

        )+
    };
}
#[allow(unused_imports)]
use test_round_trips;

macro_rules! dehydrate {
    ($buf:expr, $( $element:expr ),+) => {{
        $(
            $element.dehydrate($buf)?;
        )+
    }};
}
use dehydrate;

// vim: foldmethod=marker
