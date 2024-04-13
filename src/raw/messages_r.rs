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
    dehydrate, Dehydrate, Hydrate, Qid, SliceError, Stat, StatError, StringError, Tag, Type,
    Version, VersionError,
};
use std::{
    io::{Cursor, Error, Read, Write},
    num::TryFromIntError,
};

///
#[derive(Debug)]
pub enum RError {
    ///
    TooLong,

    ///
    IoError(Error),

    ///
    VersionError(VersionError),

    ///
    StringError(StringError),
}

impl From<Error> for RError {
    fn from(e: Error) -> Self {
        Self::IoError(e)
    }
}

impl From<VersionError> for RError {
    fn from(ve: VersionError) -> Self {
        match ve {
            // pull the version error(s) out.
            VersionError::StringError(se) => se.into(),
            _ => Self::VersionError(ve),
        }
    }
}

impl From<StringError> for RError {
    fn from(se: StringError) -> Self {
        match se {
            StringError::IoError(ioe) => Self::IoError(ioe),
            _ => Self::StringError(se),
        }
    }
}

impl From<TryFromIntError> for RError {
    fn from(_: TryFromIntError) -> Self {
        Self::TooLong
    }
}

impl From<StatError> for RError {
    fn from(se: StatError) -> Self {
        match se {
            StatError::IoError(ioe) => Self::IoError(ioe),
            StatError::TooLarge => Self::TooLong,
            StatError::StringError(se) => se.into(),
            StatError::SliceError(se) => se.into(),
        }
    }
}

// we only call Slice here in cases where we know the Error type, and we
// kinda want to pull it up rather than pass along an underlying slice error
// state.
impl From<SliceError<std::io::Error>> for RError {
    fn from(se: SliceError<std::io::Error>) -> Self {
        match se {
            SliceError::TooLong => Self::TooLong,
            SliceError::IoError(e) => Self::IoError(e),
            SliceError::Inner(e) => Self::IoError(e),
        }
    }
}

impl From<SliceError<StatError>> for RError {
    fn from(se: SliceError<StatError>) -> Self {
        match se {
            SliceError::TooLong => Self::TooLong,
            SliceError::IoError(e) => Self::IoError(e),
            SliceError::Inner(e) => e.into(),
        }
    }
}

/// R messages are Server-to-Client messages.
#[derive(Debug, PartialEq)]
pub enum R {
    /// Unknown is constructed when the Type is unknown or unexpected.
    Unknown(Type, Tag, Vec<u8>),

    /// RVersion is part of the negotiation of the connection.
    Version(Tag, u32, Version),

    /// Open authentication file.
    Auth(Tag, Qid),

    /// Attach to a filesystem tree.
    Attach(Tag, Qid),

    /// Something went wrong. What went wrong?
    Error(Tag, String, u32),

    /// The command by this Tag has been flushed from the queue.
    Flush(Tag),

    /// Walk the filesystem to a specific File, returning Qids for everything
    /// along the path.
    Walk(Tag, Vec<Qid>),

    /// Confirmation that a specific file has been Opened.
    Open(Tag, Qid, u32),

    /// Confirmation that a specific file has been Created.
    Create(Tag, Qid, u32),

    /// Data that was read in response to a Tag
    Read(Tag, Vec<u8>),

    /// Data was confirmed to have been written.
    Write(Tag, u32),

    /// File descriptor has been clunked (closed).
    Clunk(Tag),

    /// File was removed.
    Remove(Tag),

    /// Information about a File
    Stat(Tag, Stat),

    /// Information about a File
    WStat(Tag),
}

const TYPE_RVERSION: Type = 101;
const TYPE_RAUTH: Type = 103;
const TYPE_RATTACH: Type = 105;
const TYPE_RERROR: Type = 107;
const TYPE_RFLUSH: Type = 109;
const TYPE_RWALK: Type = 111;
const TYPE_ROPEN: Type = 113;
const TYPE_RCREATE: Type = 115;
const TYPE_RREAD: Type = 117;
const TYPE_RWRITE: Type = 119;
const TYPE_RCLUNK: Type = 121;
const TYPE_RREMOVE: Type = 123;
const TYPE_RSTAT: Type = 125;
const TYPE_RWSTAT: Type = 127;

impl<T> Hydrate<T> for R
where
    T: AsRef<[u8]>,
{
    type Error = RError;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, RError> {
        let ty = Type::hydrate(b)?;
        let tag = Tag::hydrate(b)?;

        Ok(match ty {
            TYPE_RVERSION => Self::Version(tag, u32::hydrate(b)?, Version::hydrate(b)?),
            TYPE_RAUTH => Self::Auth(tag, Qid::hydrate(b)?),
            TYPE_RATTACH => Self::Attach(tag, Qid::hydrate(b)?),
            TYPE_RERROR => Self::Error(tag, String::hydrate(b)?, u32::hydrate(b)?),
            TYPE_RFLUSH => Self::Flush(tag),
            TYPE_RWALK => Self::Walk(tag, Vec::<Qid>::hydrate(b)?),
            TYPE_ROPEN => Self::Open(tag, Qid::hydrate(b)?, u32::hydrate(b)?),
            TYPE_RCREATE => Self::Create(tag, Qid::hydrate(b)?, u32::hydrate(b)?),
            TYPE_RREAD => {
                // We have to do this manually (not using a Vec<T>) since we're
                // using a u32, not a u16 here. I debated a special type that
                // we could use internally (LotsOfBytes / LotsOfBytesRef) for
                // Hydrate/Dehydrate, but since Read/Write is the only thing
                // that uses this, it seemed like a waste.

                let size = u32::hydrate(b)? as usize;
                let mut buf = vec![0u8; size];
                b.read_exact(&mut buf)?;
                Self::Read(tag, buf)
            }
            TYPE_RWRITE => Self::Write(tag, u32::hydrate(b)?),
            TYPE_RCLUNK => Self::Clunk(tag),
            TYPE_RREMOVE => Self::Remove(tag),
            TYPE_RSTAT => {
                // see bugs in stat(9P)
                let size: u16 = u16::hydrate(b)?;
                let mut buf = vec![0u8; size as usize];
                b.read_exact(&mut buf)?;
                let mut b = Cursor::new(buf);
                Self::Stat(tag, Stat::hydrate(&mut b)?)
            }
            TYPE_RWSTAT => Self::WStat(tag),
            _ => Self::Unknown(ty, tag, b.remaining_slice().into()),
        })
    }
}

impl Dehydrate for R {
    type Error = RError;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), RError> {
        match self {
            Self::Version(tag, msize, version) => dehydrate!(b, TYPE_RVERSION, tag, msize, version),
            Self::Auth(tag, qid) => dehydrate!(b, TYPE_RAUTH, tag, qid),
            Self::Attach(tag, qid) => dehydrate!(b, TYPE_RATTACH, tag, qid),
            Self::Error(tag, err, errno) => dehydrate!(b, TYPE_RERROR, tag, err.as_str(), errno),
            Self::Flush(tag) => dehydrate!(b, TYPE_RFLUSH, tag),
            Self::Walk(tag, qids) => dehydrate!(b, TYPE_RWALK, tag, qids.as_slice()),
            Self::Open(tag, qid, iounit) => dehydrate!(b, TYPE_ROPEN, tag, qid, iounit),
            Self::Create(tag, qid, iounit) => dehydrate!(b, TYPE_RCREATE, tag, qid, iounit),
            Self::Read(tag, buf) => {
                // We have to do this manually (not using a Vec<T>) since we're
                // using a u32, not a u16 here. I debated a special type that
                // we could use internally (LotsOfBytes / LotsOfBytesRef) for
                // Hydrate/Dehydrate, but since Read/Write is the only thing
                // that uses this, it seemed like a waste.

                let size: u32 = buf.len().try_into()?;
                dehydrate!(b, TYPE_RREAD, tag, size);
                b.write_all(buf)?;
            }
            Self::Write(tag, n) => dehydrate!(b, TYPE_RWRITE, tag, n),
            Self::Clunk(tag) => dehydrate!(b, TYPE_RCLUNK, tag),
            Self::Remove(tag) => dehydrate!(b, TYPE_RREMOVE, tag),
            Self::Stat(tag, stat) => {
                // see bugs in stat(9P)

                let mut c = Cursor::new(vec![]);
                dehydrate!(&mut c, stat);
                let bytes = c.into_inner();
                let size: u16 = bytes.len().try_into()?;

                dehydrate!(b, TYPE_RSTAT, tag, size);
                b.write_all(&bytes)?;
            }
            Self::WStat(tag) => dehydrate!(b, TYPE_RWSTAT, tag),
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
    use super::{Dehydrate, Hydrate, Qid, Stat, R};
    use crate::raw::{test_round_trips, FileType};
    use std::io::Cursor;

    test_round_trips!(
        R,
        R,
        (
            round_trip_unknown: R::Unknown(0xFF, 0xABCD, vec![1, 2, 3, 4]),
            round_trip_version: R::Version(0xABCD, 1024, "9P2000.L".parse().unwrap()),
            round_trip_auth: R::Auth(0xDCBA, Qid::new(FileType::File, 1, 0xDEADBEEF)),
            round_trip_attach: R::Attach(0xDCBA, Qid::new(FileType::Dir, 10, 0xDEADBEEF)),
            round_trip_error: R::Error(0xDCBA, "oh shoot".to_owned(), 0),
            round_trip_flush: R::Flush(0xDCBA),
            round_trip_walk: R::Walk(0x1234, vec![Qid::new(FileType::Excl, 3, 2), Qid::new(FileType::Unknown(42), 1, 0)]),
            round_trip_open: R::Open(0x9876, Qid::new(FileType::File, 2, 3), 1024),
            round_trip_create: R::Create(0xA012, Qid::new(FileType::File, 2, 3), 1024),
            round_trip_read: R::Read(0xA012, vec![1, 2, 3, 4, 5]),
            round_trip_write: R::Write(0xA012, 42),
            round_trip_remove: R::Remove(0xA012),
            round_trip_stat: R::Stat(0xB012, Stat::builder("name", Qid::new(FileType::File, 4, 5)).build()),
            round_trip_wstat: R::WStat(0x0000)
        )
    );
}

// vim: foldmethod=marker
