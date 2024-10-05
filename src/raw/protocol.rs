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

use super::{dehydrate, Dehydrate, Hydrate};
use std::io::Cursor;

/// Type represents the underlying object type. This is usually abstracted
/// away by the interface enum, except when the message type is unknown
/// or unexpected.
pub type Type = u8;

/// Tag is the message request/response unique identifier.
pub type Tag = u16;

/// Client-defined file descriptor.
pub type Fid = u32;

///
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OpenMode(u8);

impl From<OpenMode> for u8 {
    fn from(v: OpenMode) -> Self {
        v.0
    }
}

impl From<u8> for OpenMode {
    fn from(v: u8) -> Self {
        OpenMode(v)
    }
}

impl<T> Hydrate<T> for OpenMode
where
    Self: Sized,
    T: AsRef<[u8]>,
{
    type Error = std::io::Error;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
        Ok(u8::hydrate(b)?.into())
    }
}

impl Dehydrate for OpenMode
where
    Self: Sized,
{
    type Error = std::io::Error;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        let raw: u8 = (*self).into();
        dehydrate!(b, raw);
        Ok(())
    }
}

///
pub enum IoDirection {
    ///
    Read,

    ///
    Write,

    ///
    ReadWrite,
}

impl OpenMode {
    ///
    pub const fn direction(&self) -> IoDirection {
        match self.0 % 0x04 {
            0 => IoDirection::Read,
            1 => IoDirection::Write,
            2 => IoDirection::ReadWrite,
            3 => IoDirection::Read,
            _ => unreachable!(),
        }
    }

    /// check for execute
    pub const fn execute(&self) -> bool {
        self.0 & 0x03 == 0x03
    }

    /// truncate file
    pub const fn truncate(&self) -> bool {
        self.0 & 0x10 == 0x10
    }

    /// remove on clunk
    pub const fn remove(&self) -> bool {
        self.0 & 0x40 == 0x40
    }
}

///
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    ///
    Dir,

    ///
    Append,

    ///
    Excl,

    ///
    Auth,

    ///
    Tmp,

    ///
    Link,

    ///
    Device,

    ///
    NamedPipe,

    ///
    Socket,

    ///
    File,

    ///
    Unknown(u8),
}

impl From<std::fs::Metadata> for FileType {
    fn from(v: std::fs::Metadata) -> Self {
        if v.is_file() {
            return Self::File;
        }

        if v.is_dir() {
            return Self::Dir;
        }

        if v.is_symlink() {
            return Self::Link;
        }

        // uhhh?
        Self::Unknown(0)
    }
}

impl From<u8> for FileType {
    fn from(v: u8) -> FileType {
        match v {
            0x80 => FileType::Dir,
            0x40 => FileType::Append,
            0x20 => FileType::Excl,
            0x08 => FileType::Auth,
            0x04 => FileType::Tmp,
            0x02 => FileType::Link,
            0x00 => FileType::File,

            // Special types are not represented in a uint8. If anything is in
            // uint8 range it's unknown.
            _ => FileType::Unknown(v),
        }
    }
}

impl From<FileType> for u32 {
    fn from(qt: FileType) -> u32 {
        let v: u8 = qt.into();
        let v: u32 = v.into();
        let v = v << (32 - 8);

        v | match qt {
            FileType::Device => 0x00800000,
            FileType::NamedPipe => 0x00200000,
            FileType::Socket => 0x00100000,
            _ => 0,
        }
    }
}

impl From<u32> for FileType {
    fn from(v: u32) -> FileType {
        // drop permission bits for now.
        let v = v & 0xfffffe00;

        match v {
            0x00800000 => FileType::Device,
            0x00200000 => FileType::NamedPipe,
            0x00100000 => FileType::Socket,
            _ => {
                let v: u8 = (v >> (32 - 8)).try_into().unwrap();
                v.into()
            }
        }
    }
}

impl From<FileType> for u8 {
    fn from(qt: FileType) -> u8 {
        match qt {
            FileType::Dir => 0x80,
            FileType::Append => 0x40,
            FileType::Excl => 0x20,
            FileType::Auth => 0x08,
            FileType::Tmp => 0x04,
            FileType::Link => 0x02,
            FileType::File => 0x00,
            FileType::Unknown(v) => v,

            // Special types are not represented in a uint8.
            _ => 0x00,
        }
    }
}

impl<T> Hydrate<T> for FileType
where
    Self: Sized,
    T: AsRef<[u8]>,
{
    type Error = std::io::Error;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
        Ok(u8::hydrate(b)?.into())
    }
}

impl Dehydrate for FileType
where
    Self: Sized,
{
    type Error = std::io::Error;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        let raw: u8 = (*self).into();
        dehydrate!(b, raw);
        Ok(())
    }
}

/// Qid is a unique file identifier. Two files are the same iff they have the
/// same qid.
#[derive(Debug, Clone, PartialEq)]
pub struct Qid {
    /// the type of the file (directory, etc.), represented as a bit vector corresponding to the
    /// high 8 bits of the file’s mode word.
    pub ty: FileType,

    /// version number for given path
    pub version: u32,

    /// the file server’s unique identification for the file
    pub path: u64,
}

impl Qid {
    /// Create a new Qid from parts. This is not something you want to do
    /// as a client; but fair play for the server.
    pub fn new(ty: FileType, version: u32, path: u64) -> Qid {
        Qid { ty, version, path }
    }
}

impl<T> Hydrate<T> for Qid
where
    Self: Sized,
    T: AsRef<[u8]>,
{
    type Error = std::io::Error;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
        Ok(Qid::new(
            FileType::hydrate(b)?,
            u32::hydrate(b)?,
            u64::hydrate(b)?,
        ))
    }
}

impl Dehydrate for Qid
where
    Self: Sized,
{
    type Error = std::io::Error;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        dehydrate!(b, self.ty, self.version, self.path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_round_trip, Dehydrate, FileType, Hydrate, Qid};
    use std::io::Cursor;

    test_round_trip!(
        round_trip_qid,
        Qid,
        Qid,
        (Qid::new(FileType::File, 10, 0xF00CAFE))
    );

    #[test]
    fn test_filetype() {
        for (ft, check) in [
            (FileType::File, 0u8),
            (FileType::Dir, 0x80),
            (FileType::Append, 0x40),
            (FileType::Excl, 0x20),
            (FileType::Auth, 0x08),
            (FileType::Tmp, 0x04),
            (FileType::Link, 0x02),
            //
            // special files
            (FileType::Device, 0x00),
            (FileType::NamedPipe, 0x00),
            (FileType::Socket, 0x00),
        ] {
            assert_eq!(check, ft.into());
        }

        for (ft, check) in [
            (FileType::File, 0u32),
            (FileType::Dir, 0x80000000),
            (FileType::Append, 0x40000000),
            (FileType::Excl, 0x20000000),
            (FileType::Auth, 0x08000000),
            (FileType::Tmp, 0x04000000),
            (FileType::Link, 0x02000000),
            (FileType::Device, 0x00800000),
            (FileType::NamedPipe, 0x00200000),
            (FileType::Socket, 0x00100000),
        ] {
            let ftu: u32 = ft.into();
            assert_eq!(check, ftu);
            assert_eq!(ft, ftu.into());
        }
    }
}

// vim: foldmethod=marker
