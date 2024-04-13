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

use arigato::{
    raw::{Dehydrate, FileType, IoDirection, OpenMode, Qid, Stat},
    server::{
        File as FileTrait, FileError, FileResult, Filesystem as FilesystemTrait,
        OpenFile as OpenFileTrait,
    },
};
use std::io::{Cursor, Read, Seek, SeekFrom};

///
pub struct Zero {}

impl Zero {
    pub fn new() -> Zero {
        Zero {}
    }
}

impl FilesystemTrait for Zero {
    // type File = File;
    type File = File;

    async fn attach(&self, _: &str, _: &str, _: u32) -> FileResult<File> {
        Ok(File::Directory)
    }
}

///
#[derive(Clone, Debug)]
pub enum File {
    ///
    Directory,

    ///
    Zero,

    ///
    Gig,

    ///
    TenGig,

    ///
    HundredGig,
}

impl File {
    fn name(&self) -> &str {
        match self {
            Self::Directory => "/",
            Self::Zero => "zero",
            Self::Gig => "1gig",
            Self::TenGig => "10gig",
            Self::HundredGig => "100gig",
        }
    }
}

impl FileTrait for File {
    type OpenFile = OpenFile;

    fn qid(&self) -> Qid {
        match self {
            Self::Directory => Qid::new(FileType::Dir, 0, 1u64),
            Self::Zero => Qid::new(FileType::File, 0, 2u64),
            Self::Gig => Qid::new(FileType::File, 0, 3u64),
            Self::TenGig => Qid::new(FileType::File, 0, 4u64),
            Self::HundredGig => Qid::new(FileType::File, 0, 5u64),
        }
    }

    async fn stat(&self) -> FileResult<Stat> {
        let qid = self.qid();

        let sb = Stat::builder(self.name(), qid)
            .with_nuid(0)
            .with_ngid(0)
            .with_nmuid(0);

        let sb = match self {
            Self::Directory => sb.with_mode(0o777),
            Self::Zero => sb.with_mode(0o666),
            Self::Gig => sb.with_mode(0o666).with_size(1_000_000_000),
            Self::TenGig => sb.with_mode(0o666).with_size(10_000_000_000),
            Self::HundredGig => sb.with_mode(0o666).with_size(100_000_000_000),
        };

        Ok(sb.build())
    }

    async fn wstat(&mut self, _: &Stat) -> FileResult<()> {
        Ok(())
    }

    async fn walk(&self, path: &[&str]) -> FileResult<(Option<Self>, Vec<Self>)> {
        if path.is_empty() {
            return Ok((Some(self.clone()), vec![]));
        }

        match self {
            Self::Directory => {
                if path.len() != 1 {
                    return Err(FileError(2, "ENOENT".to_owned()));
                }

                let path = path[0];
                match path {
                    "zero" => return Ok((Some(Self::Zero), vec![self.clone()])),
                    "1gig" => return Ok((Some(Self::Gig), vec![self.clone()])),
                    "10gig" => return Ok((Some(Self::TenGig), vec![self.clone()])),
                    "100gig" => return Ok((Some(Self::HundredGig), vec![self.clone()])),
                    _ => {}
                }
            }
            _ => {}
        };

        Err(FileError(2, "ENOENT".to_owned()))
    }

    async fn unlink(&mut self) -> FileResult<()> {
        Err(FileError(1, "EPERM".to_owned()))
    }

    async fn create(
        &mut self,
        _: &str,
        _: u16,
        _: arigato::raw::FileType,
        _: OpenMode,
        _: &str,
    ) -> FileResult<Self> {
        Err(FileError(1, "EPERM".to_owned()))
    }

    async fn open(&mut self, om: OpenMode) -> FileResult<OpenFile> {
        match self {
            Self::Directory => {
                match om.direction() {
                    IoDirection::Read => {}
                    _ => return Err(FileError(1, "EPERM".to_owned())),
                }

                let mut ent = Cursor::new(vec![]);

                Self::Zero.stat().await?.dehydrate(&mut ent).unwrap();
                Self::Gig.stat().await?.dehydrate(&mut ent).unwrap();
                Self::TenGig.stat().await?.dehydrate(&mut ent).unwrap();
                Self::HundredGig.stat().await?.dehydrate(&mut ent).unwrap();

                Ok(OpenFile::Cursor(ent))
            }
            Self::Zero => Ok(OpenFile::Zero),
            Self::Gig => Ok(OpenFile::Gig),
            Self::TenGig => Ok(OpenFile::TenGig),
            Self::HundredGig => Ok(OpenFile::HundredGig),
        }
    }
}

///
pub enum OpenFile {
    ///
    Cursor(Cursor<Vec<u8>>),

    ///
    Zero,

    ///
    Gig,

    ///
    TenGig,

    ///
    HundredGig,
}

impl OpenFileTrait for OpenFile {
    fn iounit(&self) -> u32 {
        0
    }

    async fn read_at(&mut self, buf: &mut [u8], off: u64) -> FileResult<u32> {
        match self {
            Self::Cursor(cur) => {
                cur.seek(SeekFrom::Start(off))?;
                Ok(cur.read(buf)? as u32)
            }
            Self::Zero => Ok(buf.len() as u32),
            Self::Gig => Ok(buf.len().min((1_000_000_000 - off) as usize) as u32),
            Self::TenGig => Ok(buf.len().min((10_000_000_000 - off) as usize) as u32),
            Self::HundredGig => Ok(buf.len().min((100_000_000_000 - off) as usize) as u32),
        }
    }

    async fn write_at(&mut self, buf: &mut [u8], _: u64) -> FileResult<u32> {
        match self {
            Self::Cursor(_) => Err(FileError(1, "EPERM".to_owned())),
            Self::Zero => Ok(buf.len() as u32),
            Self::Gig => Err(FileError(1, "EPERM".to_owned())),
            Self::TenGig => Err(FileError(1, "EPERM".to_owned())),
            Self::HundredGig => Err(FileError(1, "EPERM".to_owned())),
        }
    }
}

// vim: foldmethod=marker
