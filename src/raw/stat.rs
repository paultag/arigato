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

use super::{dehydrate, Dehydrate, Hydrate, Qid, SliceError, StringError};
use std::{
    io::{Cursor, Read},
    num::TryFromIntError,
};

/// Stat
#[derive(PartialEq, Debug, Clone)]
pub struct Stat {
    ///
    pub ty: u16,
    ///
    pub dev: u32,
    ///
    pub qid: Qid,
    ///
    pub mode: u32,
    ///
    pub atime: u32,
    ///
    pub mtime: u32,
    ///
    pub length: u64,
    ///
    pub name: String,
    ///
    pub uid: String,
    ///
    pub gid: String,
    ///
    pub muid: String,
    ///
    pub extension: String,
    ///
    pub nuid: u32,
    ///
    pub ngid: u32,
    ///
    pub nmuid: u32,
}

///
#[derive(Debug)]
pub enum StatError {
    ///
    TooLarge,

    ///
    IoError(std::io::Error),

    ///
    StringError(StringError),

    ///
    SliceError(SliceError<std::io::Error>),
}

impl From<SliceError<std::io::Error>> for StatError {
    fn from(se: SliceError<std::io::Error>) -> Self {
        match se {
            SliceError::Inner(e) => Self::IoError(e),
            SliceError::IoError(e) => Self::IoError(e),
            SliceError::TooLong => Self::TooLarge,
        }
    }
}

impl From<std::io::Error> for StatError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<StringError> for StatError {
    fn from(e: StringError) -> Self {
        Self::StringError(e)
    }
}

impl From<TryFromIntError> for StatError {
    fn from(_e: TryFromIntError) -> Self {
        Self::TooLarge
    }
}

///
#[derive(Debug, Clone)]
pub struct StatBuilder {
    ty: u16,
    dev: u32,
    qid: Qid,
    mode: u32,
    atime: u32,
    mtime: u32,
    length: u64,
    name: String,
    uid: String,
    gid: String,
    muid: String,
    extension: String,
    nuid: u32,
    ngid: u32,
    nmuid: u32,
}

impl StatBuilder {
    /// Create a new StatBuilder
    pub fn new(name: &str, qid: Qid) -> StatBuilder {
        StatBuilder {
            ty: 0,
            dev: 0,
            qid: qid,
            mode: 0,
            atime: 0,
            mtime: 0,
            length: 0,
            name: name.to_owned(),
            uid: "".to_owned(),
            gid: "".to_owned(),
            muid: "".to_owned(),
            extension: "".to_owned(),
            nuid: 0,
            ngid: 0,
            nmuid: 0,
        }
    }

    ///
    pub fn with_mode(mut self, mode: u32) -> Self {
        self.mode = mode;
        self
    }

    ///
    pub fn with_atime(mut self, atime: u32) -> Self {
        self.atime = atime;
        self
    }

    ///
    pub fn with_mtime(mut self, mtime: u32) -> Self {
        self.mtime = mtime;
        self
    }

    ///
    pub fn with_size(mut self, size: u64) -> Self {
        self.length = size;
        self
    }

    ///
    pub fn with_uid(mut self, uid: &str) -> Self {
        self.uid = uid.to_owned();
        self
    }

    ///
    pub fn with_gid(mut self, gid: &str) -> Self {
        self.gid = gid.to_owned();
        self
    }

    ///
    pub fn with_extension(mut self, ext: &str) -> Self {
        self.extension = ext.to_owned();
        self
    }

    ///
    pub fn with_muid(mut self, muid: &str) -> Self {
        self.muid = muid.to_owned();
        self
    }

    ///
    pub fn with_nuid(mut self, nuid: u32) -> Self {
        self.nuid = nuid;
        self
    }
    ///
    pub fn with_ngid(mut self, ngid: u32) -> Self {
        self.ngid = ngid;
        self
    }

    ///
    pub fn with_nmuid(mut self, nmuid: u32) -> Self {
        self.nmuid = nmuid;
        self
    }

    ///
    pub fn build(self) -> Stat {
        let Self {
            ty,
            dev,
            qid,
            mode,
            atime,
            mtime,
            length,
            name,
            uid,
            gid,
            muid,
            extension,
            nuid,
            ngid,
            nmuid,
        } = self;

        // override the provided mode.
        let qid_mode: u32 = qid.ty.into();
        let mode = mode & 0x00FFFFFF | qid_mode;

        Stat::new(
            ty, dev, qid, mode, atime, mtime, length, name, uid, gid, muid, extension, nuid, ngid,
            nmuid,
        )
    }
}

impl Stat {
    /// Create a new StatBuilder.
    pub fn builder(name: &str, qid: Qid) -> StatBuilder {
        StatBuilder::new(name, qid)
    }

    /// Create a new Stat object
    fn new(
        ty: u16,
        dev: u32,
        qid: Qid,
        mode: u32,
        atime: u32,
        mtime: u32,
        length: u64,
        name: String,
        uid: String,
        gid: String,
        muid: String,
        extension: String,
        nuid: u32,
        ngid: u32,
        nmuid: u32,
    ) -> Self {
        Self {
            ty,
            dev,
            qid,
            mode,
            atime,
            mtime,
            length,
            name,
            uid,
            gid,
            muid,
            extension,
            nuid,
            ngid,
            nmuid,
        }
    }
}

impl<T> Hydrate<T> for Stat
where
    Self: Sized,
    T: AsRef<[u8]>,
{
    type Error = StatError;

    fn hydrate(b: &mut Cursor<T>) -> Result<Self, Self::Error> {
        let size = u16::hydrate(b)? as usize;
        let mut buf = Vec::with_capacity(size);
        b.read_exact(&mut buf)?;

        Ok(Stat::new(
            // f
            u16::hydrate(b)?,
            u32::hydrate(b)?,
            Qid::hydrate(b)?,
            u32::hydrate(b)?,
            u32::hydrate(b)?,
            u32::hydrate(b)?,
            u64::hydrate(b)?,
            String::hydrate(b)?,
            String::hydrate(b)?,
            String::hydrate(b)?,
            String::hydrate(b)?,
            String::hydrate(b)?,
            u32::hydrate(b)?,
            u32::hydrate(b)?,
            u32::hydrate(b)?,
        ))
    }
}

impl Dehydrate for Stat
where
    Self: Sized,
{
    type Error = StatError;

    fn dehydrate(&self, b: &mut Cursor<Vec<u8>>) -> Result<(), Self::Error> {
        // first pass is to write the Stat into a buffer, we size it up
        // and then send it along.

        let mut out = Cursor::new(vec![]);
        dehydrate!(
            &mut out,
            self.ty,
            self.dev,
            self.qid,
            self.mode,
            self.atime,
            self.mtime,
            self.length,
            self.name.as_str(),
            self.uid.as_str(),
            self.gid.as_str(),
            self.muid.as_str(),
            self.extension.as_str(),
            self.nuid,
            self.ngid,
            self.nmuid
        );
        dehydrate!(b, out.into_inner().as_slice());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{test_round_trip, FileType},
        Dehydrate, Hydrate, Qid, Stat,
    };
    use std::io::Cursor;
    test_round_trip!(
        round_trip_qid,
        Stat,
        Stat,
        (Stat::builder("name", Qid::new(FileType::Unknown(3), 4, 5))
            .with_size(1024)
            .with_uid("uid")
            .with_gid("gid")
            .with_muid("muid")
            .with_atime(10)
            .with_mtime(20)
            .with_nuid(500)
            .with_ngid(501)
            .with_nmuid(502)
            .with_extension("something")
            .build())
    );
}

// vim: foldmethod=marker
