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

//! Async i/o

use crate::raw::{Dehydrate, Hydrate, R, RError, T, TError};
use std::{io::Cursor, pin::Pin};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Wrapper around tokio's AsyncRead, which is boxed and pinned for use by
/// futures.
pub type AsyncRead = Pin<Box<dyn tokio::io::AsyncRead + Send>>;

/// Wrapper around tokio's AsyncWrite, which is boxed and pinned for use by
/// futures.
pub type AsyncWrite = Pin<Box<dyn tokio::io::AsyncWrite + Send>>;

macro_rules! async_reader {
    ($name:ident -> <$ty:ty, $err:ty>, $overlong:expr) => {
        /// Read messages from the underlying [AsyncRead].
        pub struct $name(AsyncRead, u32);

        unsafe impl Send for $name {}

        impl $name {
            /// Create a new Reader, taking ownership of the [AsyncRead] object.
            pub fn new(r: AsyncRead, msize: u32) -> Self {
                Self(r, msize)
            }

            /// Set the limiting msize.
            pub fn set_msize(&mut self, msize: u32) {
                self.1 = msize;
            }

            /// Pull the next message from the underlying stream.
            pub async fn next(&mut self) -> Result<$ty, $err> {
                let mut size = [0, 0, 0, 0];
                self.0.read_exact(&mut size).await?;
                let size = u32::from_le_bytes(size);
                if size > self.1 {
                    return Err($overlong);
                }
                let size = size as usize;
                let mut buf = vec![0u8; size - 4];
                self.0.read_exact(&mut buf).await?;
                let mut c = Cursor::new(buf);
                <$ty>::hydrate(&mut c)
            }
        }
    };
}

macro_rules! async_writer {
    ($name:ident -> <$ty:ty, $err:ty>, $overlong:expr) => {
        /// Write messages to the underlying [AsyncWrite].
        pub struct $name(AsyncWrite, u32);

        unsafe impl Send for $name {}

        impl $name {
            /// Create a new Writer, taking ownership of the [AsyncWrite] object.
            pub fn new(w: AsyncWrite, msize: u32) -> Self {
                Self(w, msize)
            }

            /// Set the limiting msize.
            pub fn set_msize(&mut self, msize: u32) {
                self.1 = msize;
            }

            /// Write a message to the underlying stream.
            pub async fn send(&mut self, msg: $ty) -> Result<(), $err> {
                let mut buf = Cursor::new(vec![0; self.1 as usize]);
                msg.dehydrate(&mut buf)?;
                let pos = buf.position() as usize;
                let size = pos + 4;

                if size > (self.1 as usize) {
                    return Err($overlong);
                }

                self.0.write_all(&(size as u32).to_le_bytes()).await?;
                let buf = buf.into_inner();
                self.0.write_all(&buf[..pos]).await?;
                Ok(())
            }
        }
    };
}

async_reader!(RReader -> <R, RError>, RError::TooLong);
async_reader!(TReader -> <T, TError>, TError::TooLong);

async_writer!(RWriter -> <R, RError>, RError::TooLong);
async_writer!(TWriter -> <T, TError>, TError::TooLong);

// vim: foldmethod=marker
