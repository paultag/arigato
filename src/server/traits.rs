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

use crate::raw::{FileType, OpenMode, Qid, Stat};
use std::future::Future;

///
#[derive(Debug)]
pub struct FileError(pub u32, pub String);

impl From<std::io::Error> for FileError {
    fn from(e: std::io::Error) -> Self {
        match e.raw_os_error() {
            Some(ose) => FileError(ose as u32, format!("{:?}", e)),
            None => FileError(0, "".to_owned()),
        }
    }
}

///
pub trait OpenFile {
    ///
    fn iounit(&self) -> u32;

    /// Read the file at some particular offset.
    fn read_at(
        &mut self,
        buf: &mut [u8],
        offset: u64,
    ) -> impl Future<Output = FileResult<u32>> + Send;

    /// Write the file at some particular offset.
    fn write_at(
        &mut self,
        buf: &mut [u8],
        offset: u64,
    ) -> impl Future<Output = FileResult<u32>> + Send;
}

///
pub trait File
where
    Self: Sized,
{
    ///
    type OpenFile: OpenFile + Send;

    /// Get metadata about the file itself.
    fn stat(&self) -> impl Future<Output = FileResult<Stat>> + Send;

    /// Write stat back to the file
    fn wstat(&mut self, s: &Stat) -> impl Future<Output = FileResult<()>> + Send;

    /// Walk will navigate
    fn walk(
        &self,
        path: &[&str],
    ) -> impl Future<Output = FileResult<(Option<Self>, Vec<Self>)>> + Send;

    /// remove the file
    fn unlink(&mut self) -> impl Future<Output = FileResult<()>> + Send;

    /// create the file
    fn create(
        &mut self,
        name: &str,
        perm: u16,
        ty: FileType,
        mode: OpenMode,
        extension: &str,
    ) -> impl Future<Output = FileResult<Self>> + Send;

    /// Open
    fn open(&mut self, mode: OpenMode) -> impl Future<Output = FileResult<Self::OpenFile>> + Send;

    /// sync (not async)
    fn qid(&self) -> Qid;
}

///
pub type FileResult<RetT> = Result<RetT, FileError>;

///
pub type FilesystemResult<RetT> = Result<RetT, FileError>;

///
pub trait Filesystem {
    ///
    type File: File + Send + 'static;

    ///
    fn attach(
        &self,
        aname: &str,
        uname: &str,
        nuname: u32,
    ) -> impl Future<Output = FilesystemResult<Self::File>> + Send;
}

// vim: foldmethod=marker
