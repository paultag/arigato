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

use super::{
    dehydrate, Dehydrate, Fid, Hydrate, OpenMode, SliceError, StatError, StringError, Tag, Type,
    Version, VersionError,
};
use crate::raw::Stat;
use std::{
    io::{Cursor, Error, Read, Write},
    num::TryFromIntError,
};

///
#[derive(Debug)]
pub enum TError {
    ///
    TooLong,

    ///
    IoError(Error),

    ///
    StringError(StringError),

    ///
    VersionError(VersionError),

    ///
    StatError(StatError),
}

impl From<TryFromIntError> for TError {
    fn from(_: TryFromIntError) -> Self {
        Self::TooLong
    }
}

impl From<VersionError> for TError {
    fn from(ve: VersionError) -> Self {
        match ve {
            // pull the version error(s) out.
            VersionError::StringError(se) => se.into(),
            _ => Self::VersionError(ve),
        }
    }
}

impl From<StatError> for TError {
    fn from(se: StatError) -> Self {
        Self::StatError(se)
    }
}

impl From<StringError> for TError {
    fn from(se: StringError) -> Self {
        match se {
            StringError::IoError(ioe) => Self::IoError(ioe),
            _ => Self::StringError(se),
        }
    }
}

impl From<std::io::Error> for TError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<SliceError<StringError>> for TError {
    fn from(se: SliceError<StringError>) -> Self {
        match se {
            SliceError::TooLong => Self::TooLong,
            SliceError::IoError(e) => Self::IoError(e),
            SliceError::Inner(e) => e.into(),
        }
    }
}

impl From<SliceError<std::io::Error>> for TError {
    fn from(se: SliceError<std::io::Error>) -> Self {
        match se {
            SliceError::TooLong => Self::TooLong,
            SliceError::IoError(e) => Self::IoError(e),
            SliceError::Inner(e) => e.into(),
        }
    }
}

/// T messages are Client-to-Server messages. This is 9P2000.u, *not* 9P2000.
#[derive(Debug, PartialEq, Clone)]
pub enum T {
    /// Unknown is constructed when the Type is unknown or unexpected.
    Unknown(Type, Tag, Vec<u8>),

    ///
    Version(Tag, u32, Version),

    ///
    Auth(Tag, Fid, String, String, u32),

    ///
    Attach(Tag, Fid, Fid, String, String, u32),

    ///
    Flush(Tag, Tag),

    ///
    Walk(Tag, Fid, Fid, Vec<String>),

    ///
    Open(Tag, Fid, OpenMode),

    ///
    Create(Tag, Fid, String, u32, u8, String),

    ///
    Read(Tag, Fid, u64, u32),

    ///
    Write(Tag, Fid, u64, Vec<u8>),

    ///
    Clunk(Tag, Fid),

    ///
    Remove(Tag, Fid),

    ///
    Stat(Tag, Fid),

    ///
    WStat(Tag, Fid, Stat),
}

impl T {
    ///
    pub fn tag(&self) -> Tag {
        match self {
            T::Version(tag, _, _) => *tag,
            T::Attach(tag, _, _, _, _, _) => *tag,
            T::Flush(tag, _) => *tag,
            T::Auth(tag, _, _, _, _) => *tag,
            T::Walk(tag, _, _, _) => *tag,
            T::Open(tag, _, _) => *tag,
            T::Create(tag, _, _, _, _, _) => *tag,
            T::Read(tag, _, _, _) => *tag,
            T::Write(tag, _, _, _) => *tag,
            T::Clunk(tag, _) => *tag,
            T::Remove(tag, _) => *tag,
            T::Stat(tag, _) => *tag,
            T::WStat(tag, _, _) => *tag,
            T::Unknown(_, tag, _) => *tag,
        }
    }
}

const TYPE_TVERSION: Type = 100;
const TYPE_TAUTH: Type = 102;
const TYPE_TATTACH: Type = 104;
const TYPE_TFLUSH: Type = 108;
const TYPE_TWALK: Type = 110;
const TYPE_TOPEN: Type = 112;
const TYPE_TCREATE: Type = 114;
const TYPE_TREAD: Type = 116;
const TYPE_TWRITE: Type = 118;
const TYPE_TCLUNK: Type = 120;
const TYPE_TREMOVE: Type = 122;
const TYPE_TSTAT: Type = 124;
const TYPE_TWSTAT: Type = 126;

impl<ContainerT> Hydrate<ContainerT> for T
where
    ContainerT: AsRef<[u8]>,
{
    type Error = TError;

    fn hydrate(b: &mut Cursor<ContainerT>) -> Result<Self, TError> {
        let ty = Type::hydrate(b)?;
        let tag = Tag::hydrate(b)?;

        Ok(match ty {
            TYPE_TVERSION => Self::Version(tag, u32::hydrate(b)?, Version::hydrate(b)?),
            TYPE_TAUTH => Self::Auth(
                tag,
                Fid::hydrate(b)?,
                String::hydrate(b)?,
                String::hydrate(b)?,
                u32::hydrate(b)?,
            ),
            TYPE_TATTACH => Self::Attach(
                tag,
                Fid::hydrate(b)?,
                Fid::hydrate(b)?,
                String::hydrate(b)?,
                String::hydrate(b)?,
                u32::hydrate(b)?,
            ),
            TYPE_TFLUSH => Self::Flush(tag, Tag::hydrate(b)?),
            TYPE_TWALK => Self::Walk(
                tag,
                Fid::hydrate(b)?,
                Fid::hydrate(b)?,
                Vec::<String>::hydrate(b)?,
            ),
            TYPE_TOPEN => Self::Open(tag, Fid::hydrate(b)?, OpenMode::hydrate(b)?),
            TYPE_TCREATE => Self::Create(
                tag,
                Fid::hydrate(b)?,
                String::hydrate(b)?,
                u32::hydrate(b)?,
                u8::hydrate(b)?,
                String::hydrate(b)?,
            ),
            TYPE_TREAD => Self::Read(tag, Fid::hydrate(b)?, u64::hydrate(b)?, u32::hydrate(b)?),
            TYPE_TWRITE => {
                // We have to do this manually (not using a Vec<T>) since we're
                // using a u32, not a u16 here. I debated a special type that
                // we could use internally (LotsOfBytes / LotsOfBytesRef) for
                // Hydrate/Dehydrate, but since Read/Write is the only thing
                // that uses this, it seemed like a waste.

                let fid = Fid::hydrate(b)?;
                let offset = u64::hydrate(b)?;
                let size = u32::hydrate(b)? as usize;
                let mut buf = vec![0u8; size];
                b.read_exact(&mut buf)?;

                Self::Write(tag, fid, offset, buf)
            }
            TYPE_TCLUNK => Self::Clunk(tag, Fid::hydrate(b)?),
            TYPE_TREMOVE => Self::Remove(tag, Fid::hydrate(b)?),
            TYPE_TSTAT => Self::Stat(tag, Fid::hydrate(b)?),
            TYPE_TWSTAT => {
                // see bugs in stat(9P)

                let fid = Fid::hydrate(b)?;

                let size: u16 = u16::hydrate(b)?;
                let mut buf = vec![0u8; size as usize];
                b.read_exact(&mut buf)?;
                let mut b = Cursor::new(buf);
                Self::WStat(tag, fid, Stat::hydrate(&mut b)?)
            }
            _ => Self::Unknown(ty, tag, b.remaining_slice().into()),
        })
    }
}

impl Dehydrate for T {
    type Error = TError;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), TError> {
        match self {
            Self::Version(tag, msize, version) => dehydrate!(b, TYPE_TVERSION, tag, msize, version),
            Self::Auth(tag, fid, uname, aname, nuname) => {
                dehydrate!(
                    b,
                    TYPE_TAUTH,
                    tag,
                    fid,
                    uname.as_str(),
                    aname.as_str(),
                    nuname
                )
            }
            Self::Attach(tag, fid, afid, uname, aname, nuname) => dehydrate!(
                b,
                TYPE_TATTACH,
                tag,
                fid,
                afid,
                uname.as_str(),
                aname.as_str(),
                nuname
            ),
            Self::Flush(tag, oldtag) => dehydrate!(b, TYPE_TFLUSH, tag, oldtag),
            Self::Walk(tag, fid, newfid, paths) => {
                dehydrate!(b, TYPE_TWALK, tag, fid, newfid, paths.as_slice())
            }
            Self::Open(tag, fid, mode) => dehydrate!(b, TYPE_TOPEN, tag, fid, mode),
            Self::Create(tag, fid, name, perm, mode, ext) => {
                dehydrate!(b, TYPE_TCREATE, tag, fid, name, perm, mode, ext)
            }
            Self::Read(tag, fid, offset, len) => {
                dehydrate!(b, TYPE_TREAD, tag, fid, offset, len)
            }
            Self::Write(tag, fid, offset, buf) => {
                // We have to do this manually (not using a Vec<T>) since we're
                // using a u32, not a u16 here. I debated a special type that
                // we could use internally (LotsOfBytes / LotsOfBytesRef) for
                // Hydrate/Dehydrate, but since Read/Write is the only thing
                // that uses this, it seemed like a waste.

                let size: u32 = buf.len().try_into()?;
                dehydrate!(b, TYPE_TWRITE, tag, fid, offset, size);
                b.write_all(buf)?;
            }
            Self::Clunk(tag, fid) => {
                dehydrate!(b, TYPE_TCLUNK, tag, fid)
            }
            Self::Remove(tag, fid) => {
                dehydrate!(b, TYPE_TREMOVE, tag, fid)
            }
            Self::Stat(tag, fid) => {
                dehydrate!(b, TYPE_TSTAT, tag, fid)
            }
            Self::WStat(tag, fid, stat) => {
                let mut c = Cursor::new(vec![]);
                dehydrate!(&mut c, stat);
                let bytes = c.into_inner();
                let size: u16 = bytes.len().try_into()?;

                dehydrate!(b, TYPE_TWSTAT, tag, fid, size);
                b.write_all(&bytes)?;
            }
            Self::Unknown(ty, tag, buf) => {
                dehydrate!(b, ty, tag);
                b.write_all(buf)?;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Dehydrate, Hydrate, T};
    use crate::raw::{test_round_trips, FileType, Qid, Stat};
    use std::io::Cursor;

    test_round_trips!(
        T,
        T,
        (
            round_trip_unknown: T::Unknown(0xFF, 0xABCD, vec![1, 2, 3, 4]),
            round_trip_version: T::Version(0xABCD, 1024, "9P2000.L".parse().unwrap()),
            round_trip_auth: T::Auth(0x1234, 1, "foo".to_owned(), "bar".to_owned(), 0),
            round_trip_attach: T::Attach(0x1234, 1, 2, "foo".to_owned(), "bar".to_owned(), 0),
            round_trip_flush: T::Flush(0x1234, 0x4321),
            round_trip_walk: T::Walk(0x1234, 1, 2, vec!["bin".to_owned(), "bash".to_owned()]),
            round_trip_open: T::Open(0x1234, 1, 2.into()),
            round_trip_create: T::Create(0x1234, 1, "foo".to_owned(), 20, 21, "".to_owned()),
            round_trip_read: T::Read(0x1234, 1, 2, 3),
            round_trip_write: T::Write(0x1234, 1, 2, vec![1, 2, 3, 4, 5, 6]),
            round_trip_clunk: T::Clunk(0x1234, 1),
            round_trip_remove: T::Remove(0x1234, 20),
            round_trip_stat: T::Stat(0x1234, 2),
            round_trip_wstat: T::WStat(0x1234, 2, Stat::builder("name", Qid::new(FileType::File, 4, 5)).build())
        )
    );
}

// vim: foldmethod=marker
